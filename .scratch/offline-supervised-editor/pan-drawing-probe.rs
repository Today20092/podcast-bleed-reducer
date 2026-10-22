//! Read-only production waveform/gain probe; unused file-loading entry points never execute.
// The example reuses production modules; their unit tests run in the library target.
#![cfg(not(test))]
#![allow(dead_code, unused_imports)]
use rust_app::*;
#[path = "../../src/appearance.rs"]
mod appearance;
#[path = "../../src/shortcuts.rs"]
mod shortcuts;
mod performance {
    pub use rust_app::performance::Metric;
    // The software probe does not start the native editor's optional recorder.
    pub fn measure(_: Metric) -> Option<()> {
        None
    }
}
#[path = "../../src/editor_drawing.rs"]
mod production_editor_drawing;
mod editor_drawing {
    pub use super::production_editor_drawing::*;
    pub static DENSE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    pub(crate) fn sample_visible_curve(
        viewport: &crate::timeline::TimelineViewport,
        width: f32,
        gain: impl Fn(u64) -> f32,
    ) -> Vec<crate::EnvelopePoint> {
        if !DENSE.load(std::sync::atomic::Ordering::Relaxed) {
            return super::production_editor_drawing::sample_visible_curve(viewport, width, gain);
        }
        // Frozen e824482 sampling baseline, interleaved in the same process/renderer.
        let columns = (width.ceil() as usize).clamp(1, 4096);
        (0..=columns)
            .map(|column| {
                let time =
                    visible_boundary(viewport, width * column as f32 / columns as f32, width);
                crate::EnvelopePoint {
                    time,
                    gain: gain(time),
                }
            })
            .collect()
    }
}
#[path = "../../src/gain_editor.rs"]
mod gain_editor;
#[path = "../../src/waveform.rs"]
mod waveform;
mod timebase {
    pub use rust_app::{Rounding, SourceTimebase, TICKS_PER_SECOND, TimePosition};
}
#[path = "../../src/timeline.rs"]
mod timeline;

struct AudioSource(rust_app::AudioSource);
impl std::ops::Deref for AudioSource {
    type Target = rust_app::AudioSource;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AudioSource {
    fn mono(path: std::path::PathBuf) -> Self {
        Self(rust_app::AudioSource::mono(path))
    }
    fn fingerprint(
        &self,
        _: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<String, std::io::Error> {
        unreachable!("synthetic probe never fingerprints audio")
    }
}
mod project {
    pub fn fingerprint_cancellable(
        _: &std::path::Path,
        _: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<String, crate::RenderBlocker> {
        unreachable!("probe never loads files")
    }
    pub fn atomic_save(_: &std::path::Path, _: &[u8]) -> Result<(), crate::RenderBlocker> {
        unreachable!("probe never saves files")
    }
}
mod analysis_audio {
    pub const AUDIO_DECODER_RECIPE: &str = "unused";
}
mod audio_io {
    pub struct Decoded {
        pub metadata: crate::SourceMetadata,
    }
    pub fn decode_source_chunks(
        _: &crate::AudioSource,
        _: Option<&std::sync::atomic::AtomicBool>,
        _: impl FnMut(u32, &[f32]) -> Result<(), crate::RenderBlocker>,
    ) -> Result<Decoded, crate::RenderBlocker> {
        unreachable!("synthetic waveform only")
    }
}
pub use editor_drawing::{visible_automatic_buckets, visible_gain_buckets};
mod prepared_analysis {
    pub fn check_cancelled(
        cancel: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<(), crate::RenderBlocker> {
        if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            Err(crate::RenderBlocker("Cancelled".into()))
        } else {
            Ok(())
        }
    }
}

fn main() {
    use iced::widget::canvas::Program;
    let project_path = std::env::args().nth(1).expect("saved project path");
    let project_bytes = std::fs::read(&project_path).unwrap();
    let project = Project::open(&project_path).unwrap();
    let follow = std::env::args().nth(2).as_deref() == Some("--follow");
    let real_cache = std::env::args().nth(3);
    let width = std::env::args()
        .nth(4)
        .map_or(1900.0, |value| value.parse().unwrap());
    let cached = std::env::args().nth(5).as_deref() == Some("--cached");
    let compare = std::env::args().any(|argument| argument == "--compare");
    assert!((1.0..=4096.0).contains(&width));
    let duration = project
        .sources
        .iter()
        .map(|s| s.metadata.grid.duration().unwrap())
        .max()
        .unwrap();
    let count = duration.div_ceil(OWNERSHIP_FRAME_TICKS) as usize;
    let row: Vec<_> = (0..count)
        .map(|i| {
            let p = if i % 100 < 50 { 0.95 } else { 0.01 };
            TemporalEvidence {
                probability: Some(p),
                raw_speech: Some(p),
                assigned_activity: Some(p),
                preservation_reason: None,
            }
        })
        .collect();
    let grid = TemporalEvidenceGrid::new(
        project.sources.iter().map(|s| s.track_id).collect(),
        duration,
        project.sources.iter().map(|_| row.clone()).collect(),
        vec![0.95; count],
    )
    .unwrap();
    let ownership = TemporalOwnershipCache::default()
        .optimize_evidence(&grid, TemporalSettings::default(), None)
        .unwrap();
    let mut automatic: Vec<_> = project
        .sources
        .iter()
        .map(|s| {
            AutomaticEnvelope::from_ownership(
                &ownership,
                s.track_id,
                s.metadata.grid,
                GainFadeSettings::default(),
                None,
            )
            .unwrap()
        })
        .collect();
    let mut effective: Vec<_> = project
        .sources
        .iter()
        .map(|s| {
            GainEnvelope::constant_at_rate(
                s.metadata.grid.frame_count as usize,
                s.metadata.grid.sample_rate,
                0.25,
            )
            .unwrap()
        })
        .collect();
    if let Some(cache) = &real_cache {
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let progress = std::sync::atomic::AtomicU8::new(0);
        let analysis = EditorAnalysisWorker::default()
            .analyze(&project, std::path::Path::new(cache), &progress, &cancel)
            .unwrap();
        let envelopes = EffectiveEnvelopeCache::default()
            .update(
                &project,
                &analysis.temporal,
                project_fade_settings(&project).unwrap(),
                Some(&cancel),
            )
            .unwrap();
        automatic = envelopes
            .automatic
            .iter()
            .map(|envelope| (**envelope).clone())
            .collect();
        effective = envelopes.envelopes.clone();
        println!(
            "Workload: actual analyzed envelopes, {} reference labels, {} ownership regions; analysis excluded from drawing timing",
            project.reference_labels().len(),
            analysis.temporal.regions().len()
        );
    }
    let constraints = project.gain_constraints();
    let preview = AnalysisPreview {
        stems: project.source_paths(),
        regions: vec![],
        envelopes: effective.clone(),
        protections: vec![],
        parameters: project.parameters(),
    };
    let waveforms: Vec<waveform::WaveformCache> = project.sources.iter().map(|source| {
        if real_cache.is_some() {
            let path = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap())
                .join("CleanSlate/waveforms-v3").join(format!("{}.json", source.fingerprint));
            let persisted: serde_json::Value = serde_json::from_slice(&std::fs::read(path).expect("actual follow probe requires retained waveform cache")).unwrap();
            assert_eq!(persisted["fingerprint"], source.fingerprint);
            assert_eq!(persisted["source"], serde_json::to_value(source.metadata).unwrap());
            let waveform: waveform::WaveformCache = serde_json::from_value(persisted["waveform"].clone()).unwrap();
            assert_eq!(waveform.sample_count as u64, source.metadata.grid.frame_count);
            assert_eq!(waveform.sample_rate, source.metadata.grid.sample_rate);
            return waveform;
        }
        let mut bucket = 256usize;
        let mut count = (source.metadata.grid.frame_count as usize).div_ceil(bucket);
        let mut levels = vec![];
        loop {
            levels.push(serde_json::json!({"samples_per_bucket": bucket, "peaks": vec![(-0.8, 0.9); count]}));
            if count <= 1 { break; }
            count = count.div_ceil(2);
            bucket *= 2;
        }
        serde_json::from_value(serde_json::json!({"sample_count": source.metadata.grid.frame_count, "sample_rate": source.metadata.grid.sample_rate, "levels": levels})).unwrap()
    }).collect();
    let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        iced::Font::default(),
        iced::Pixels(16.0),
    ));
    let states: Vec<_> = project
        .sources
        .iter()
        .map(|_| gain_editor::GainCanvasState::default())
        .collect();
    let waveform_states: Vec<_> = project
        .sources
        .iter()
        .map(|_| waveform::TimelineCanvasState::default())
        .collect();
    let mut viewport = timeline::TimelineViewport::new(duration);
    for _ in 0..if follow { 1 } else { 8 } {
        viewport.zoom_at(width / 2.0, width, 1.0);
    }
    if follow {
        viewport.seek(duration / 2);
        viewport.center_playhead();
        viewport.lock_playhead();
    }
    let mut times = vec![];
    let mut waveform_times = vec![];
    let mut gain_times = vec![];
    let mut dense_times = vec![];
    let mut dense_waveform_times = vec![];
    let mut dense_gain_times = vec![];
    let frames = if follow { 240 } else { 40 };
    for frame in 0..frames {
        if follow && !cached {
            viewport.seek(duration / 2 + frame * TICKS_PER_SECOND / 60);
            viewport.follow_playhead(timeline::PlaybackScroll::Locked);
        } else if !follow {
            viewport.pan_by_pixels(if frame < 20 { -40.0 } else { 40.0 }, width);
        }
        for dense in if compare {
            vec![frame % 2 == 0, frame % 2 != 0]
        } else {
            vec![false]
        } {
            editor_drawing::DENSE.store(dense, std::sync::atomic::Ordering::Relaxed);
            let start = std::time::Instant::now();
            let mut waveform_elapsed = std::time::Duration::ZERO;
            let mut gain_elapsed = std::time::Duration::ZERO;
            for (i, source) in project.sources.iter().enumerate() {
                let waveform_start = std::time::Instant::now();
                let waveform = waveform::TimelineCanvas::waveform(
                    &waveforms[i],
                    &viewport,
                    |_| (),
                    Some((&preview, i)),
                    false,
                )
                .with_examples(project.reference_labels(), None, None, |_| ())
                .with_revision(if cached {
                    0
                } else {
                    frame * 2 + u64::from(dense)
                });
                std::hint::black_box(waveform.draw(
                    &waveform_states[i],
                    &renderer,
                    &iced::Theme::Dark,
                    iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(width, 180.0)),
                    iced::mouse::Cursor::Unavailable,
                ));
                waveform_elapsed += waveform_start.elapsed();
                let gain_start = std::time::Instant::now();
                let canvas = gain_editor::GainCanvas::<()> {
                    revision: if cached {
                        0
                    } else {
                        frame * 2 + u64::from(dense)
                    },
                    project: &project,
                    constraints: &constraints,
                    track: source.track_id,
                    viewport: &viewport,
                    automatic: Some(&automatic[i]),
                    effective: Some(&effective[i]),
                    editing: false,
                    selected_point: None,
                    on_event: Box::new(|_| ()),
                };
                std::hint::black_box(canvas.draw(
                    &states[i],
                    &renderer,
                    &iced::Theme::Dark,
                    iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(width, 20.0)),
                    iced::mouse::Cursor::Unavailable,
                ));
                gain_elapsed += gain_start.elapsed();
            }
            let (total, wave, gain) = if dense {
                (
                    &mut dense_times,
                    &mut dense_waveform_times,
                    &mut dense_gain_times,
                )
            } else {
                (&mut times, &mut waveform_times, &mut gain_times)
            };
            total.push(start.elapsed().as_secs_f64() * 1000.0);
            wave.push(waveform_elapsed.as_secs_f64() * 1000.0);
            gain.push(gain_elapsed.as_secs_f64() * 1000.0);
        }
    }
    println!(
        "Production waveform + gain {} preparation, {} tracks/{width} columns/{frames} observations, {:.3} visible seconds; software geometry only, excludes raster/GPU/presentation",
        if cached {
            "unchanged cached control"
        } else if follow {
            "locked-follow"
        } else {
            "continuous-pan"
        },
        project.sources.len(),
        viewport.visible_samples() / TICKS_PER_SECOND as f64,
    );
    for (label, values) in [
        ("complete waveform + gain", &mut times),
        ("all waveform canvases", &mut waveform_times),
        ("all gain canvases", &mut gain_times),
        ("dense baseline complete waveform + gain", &mut dense_times),
        (
            "dense baseline all waveform canvases",
            &mut dense_waveform_times,
        ),
        ("dense baseline all gain canvases", &mut dense_gain_times),
    ] {
        if values.is_empty() {
            continue;
        }
        if compare {
            println!("raw samples ms in acquisition order, {label}: {values:?}");
        }
        values.sort_by(f64::total_cmp);
        println!(
            "{label}: count {} median {:.3} ms p95 {:.3} ms p99 {:.3} ms max {:.3} ms",
            values.len(),
            values[(values.len() as f64 * 0.50).ceil() as usize - 1],
            values[(values.len() as f64 * 0.95).ceil() as usize - 1],
            values[(values.len() as f64 * 0.99).ceil() as usize - 1],
            values[values.len() - 1],
        );
    }
    assert!(
        times[times.len() / 2] < 16.7,
        "waveform + gain preparation exceeds 60 Hz frame budget"
    );
    assert_eq!(
        std::fs::read(project_path).unwrap(),
        project_bytes,
        "probe must preserve saved project"
    );
}
