//! Read-only production waveform/gain probe; unused file-loading entry points never execute.
pub use rust_app::*;
#[path = "../../src/appearance.rs"]
mod appearance;
#[path = "../../src/shortcuts.rs"]
mod shortcuts;
mod performance {
    pub use rust_app::performance::Metric;
    // The software probe does not start the native editor's optional recorder.
    pub fn measure(_: Metric) {}
}
#[path = "../../src/editor_drawing.rs"]
mod editor_drawing;
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
    fn fingerprint(&self, _: Option<&std::sync::atomic::AtomicBool>) -> Result<String, std::io::Error> {
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
    let project = Project::open(std::env::args().nth(1).expect("saved project path")).unwrap();
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
    let automatic: Vec<_> = project
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
    let effective: Vec<_> = project
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
    let constraints = project.gain_constraints();
    let preview = AnalysisPreview {
        stems: project.source_paths(),
        regions: vec![],
        envelopes: effective.clone(),
        protections: vec![],
        parameters: project.parameters(),
    };
    let waveforms: Vec<waveform::WaveformCache> = project.sources.iter().map(|source| {
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
    for _ in 0..8 {
        viewport.zoom_at(950.0, 1900.0, 1.0);
    }
    let mut times = vec![];
    for frame in 0..40 {
        viewport.pan_by_pixels(if frame < 20 { -40.0 } else { 40.0 }, 1900.0);
        let start = std::time::Instant::now();
        for (i, source) in project.sources.iter().enumerate() {
            let waveform = waveform::TimelineCanvas::waveform(
                &waveforms[i],
                &viewport,
                |_| (),
                Some((&preview, i)),
                false,
            )
            .with_revision(frame);
            std::hint::black_box(waveform.draw(
                &waveform_states[i],
                &renderer,
                &iced::Theme::Dark,
                iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1900.0, 180.0)),
                iced::mouse::Cursor::Unavailable,
            ));
            let canvas = gain_editor::GainCanvas::<()> {
                revision: frame,
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
                iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1900.0, 20.0)),
                iced::mouse::Cursor::Unavailable,
            ));
        }
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(f64::total_cmp);
    println!(
        "Production waveform + gain continuous-scroll preparation, {} tracks/1900 columns/40 frames, software geometry: median {:.2} ms, max {:.2} ms",
        project.sources.len(),
        times[20],
        times[39]
    );
    assert!(
        times[20] < 16.7,
        "waveform + gain preparation exceeds 60 Hz frame budget"
    );
}
