use crate::{
    audio_io::{AtomicOutput, Stem},
    vad::{self, ENERGY_MARGIN_DB, SPEECH_PROBABILITY},
};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::HashSet,
    fmt, io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AutomixParameters {
    pub speech_sensitivity: f32,
    pub bleed_discrimination_db: f32,
    pub attenuation_db: f32,
    pub speech_padding_ms: f32,
    pub open_ms: f32,
    pub close_ms: f32,
}

impl<'de> Deserialize<'de> for AutomixParameters {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Stored {
            #[serde(default = "default_speech_sensitivity")]
            speech_sensitivity: f32,
            #[serde(default = "default_bleed_discrimination")]
            bleed_discrimination_db: f32,
            #[serde(default = "default_attenuation")]
            attenuation_db: f32,
            #[serde(default = "default_speech_padding")]
            speech_padding_ms: f32,
            open_ms: Option<f32>,
            close_ms: Option<f32>,
            transition_ms: Option<f32>,
        }
        let stored = Stored::deserialize(deserializer)?;
        Ok(Self {
            speech_sensitivity: stored.speech_sensitivity,
            bleed_discrimination_db: stored.bleed_discrimination_db,
            attenuation_db: stored.attenuation_db,
            speech_padding_ms: stored.speech_padding_ms,
            open_ms: stored.open_ms.or(stored.transition_ms).unwrap_or(50.0),
            close_ms: stored.close_ms.or(stored.transition_ms).unwrap_or(500.0),
        })
    }
}

fn default_speech_sensitivity() -> f32 {
    SPEECH_PROBABILITY
}
fn default_bleed_discrimination() -> f32 {
    ENERGY_MARGIN_DB
}
fn default_attenuation() -> f32 {
    -18.0
}
fn default_speech_padding() -> f32 {
    512.0
}

#[cfg(test)]
mod preview_tests {
    use super::*;

    // Keep the sample-scanning implementation as an independent compatibility oracle.
    fn scanned_envelope(
        track: usize,
        sample_count: usize,
        regions: &[Region],
        protections: &[Protection],
        attenuation: f32,
        open_samples: f32,
        close_samples: f32,
    ) -> GainEnvelope {
        let mut envelope = GainEnvelope {
            sample_count,
            points: vec![],
        };
        if sample_count == 0 {
            return envelope;
        }
        let mut target = target_gain_with_attenuation(track, 0, regions, protections, attenuation);
        envelope.points.push(GainPoint {
            sample: 0,
            gain: target,
            alpha: 0.0,
            target,
        });
        for sample in 1..sample_count {
            let next =
                target_gain_with_attenuation(track, sample, regions, protections, attenuation);
            if next == target {
                continue;
            }
            let gain = envelope.gain_at(sample - 1);
            target = next;
            let alpha = 1.0
                - (-1.0
                    / if target > gain {
                        open_samples
                    } else {
                        close_samples
                    })
                .exp();
            envelope.points.push(GainPoint {
                sample,
                gain: gain + alpha * (target - gain),
                alpha,
                target,
            });
        }
        envelope
    }

    #[test]
    fn envelope_matches_sample_scan_at_region_and_protection_edges() {
        // Sorted starts, including ties, overlaps, gaps and empty intervals.
        let regions = vec![
            Region {
                start: 3,
                end: 18,
                decision: Decision::NoSpeech,
            },
            Region {
                start: 8,
                end: 12,
                decision: Decision::Owned(vec![0, 1]),
            },
            Region {
                start: 8,
                end: 10,
                decision: Decision::Owned(vec![1]),
            },
            Region {
                start: 15,
                end: 15,
                decision: Decision::NoSpeech,
            },
            Region {
                start: 20,
                end: 28,
                decision: Decision::Uncertain,
            },
            Region {
                start: 28,
                end: 64,
                decision: Decision::NoSpeech,
            },
        ];
        let protections = vec![
            Protection {
                track: Some(0),
                start: 35,
                end: usize::MAX,
            },
            Protection {
                track: None,
                start: 5,
                end: 9,
            },
            Protection {
                track: Some(1),
                start: 8,
                end: 14,
            },
            Protection {
                track: None,
                start: 9,
                end: 11,
            },
            Protection {
                track: None,
                start: 30,
                end: 30,
            },
            Protection {
                track: Some(2),
                start: 0,
                end: 2,
            },
        ];
        for track in 0..3 {
            for count in [0, 1, 8, 64] {
                for floor in [0.0, 0.25, 1.0] {
                    for (open, close) in [(1.0, 1.0), (10.0, 20.0), (2400.0, 24000.0)] {
                        for (regions, protections) in [
                            (regions.as_slice(), protections.as_slice()),
                            (regions.as_slice(), &[][..]),
                            (&[][..], protections.as_slice()),
                            (&[][..], &[][..]),
                        ] {
                            let actual = gain_envelope(
                                track,
                                count,
                                regions,
                                protections,
                                floor,
                                open,
                                close,
                            );
                            let expected = scanned_envelope(
                                track,
                                count,
                                regions,
                                protections,
                                floor,
                                open,
                                close,
                            );
                            assert_eq!(actual.sample_count(), count);
                            for sample in 0..count {
                                assert_eq!(
                                    actual.gain_at(sample),
                                    expected.gain_at(sample),
                                    "track {track}, count {count}, sample {sample}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "long-session construction benchmark; run with --ignored --nocapture"]
    fn long_session_gain_construction_benchmark() {
        use std::{hint::black_box, time::Instant};
        let count = 239_660_221;
        let regions: Vec<_> = (0..count)
            .step_by(48_000)
            .enumerate()
            .map(|(index, start)| Region {
                start,
                end: (start + 48_000).min(count),
                decision: if index % 2 == 0 {
                    Decision::NoSpeech
                } else {
                    Decision::Owned(vec![index % 3])
                },
            })
            .collect();
        let start = Instant::now();
        let expected: Vec<_> = (0..3)
            .map(|track| scanned_envelope(track, count, &regions, &[], 0.25, 2400.0, 24000.0))
            .collect();
        let scanned = start.elapsed();
        let start = Instant::now();
        let actual: Vec<_> = (0..3)
            .map(|track| {
                black_box(gain_envelope(
                    track,
                    count,
                    &regions,
                    &[],
                    0.25,
                    2400.0,
                    24000.0,
                ))
            })
            .collect();
        let swept = start.elapsed();
        assert_eq!(actual, expected);
        println!(
            "samples={count}, tracks=3, regions={}, sample_scan={scanned:?}, edge_sweep={swept:?}, speedup={:.1}x",
            regions.len(),
            scanned.as_secs_f64() / swept.as_secs_f64()
        );
        assert!(
            swept < scanned,
            "edge construction must beat the sample scan"
        );
    }

    #[test]
    fn gain_bounds_include_internal_open_close_and_protection_edges() {
        let regions = vec![
            Region {
                start: 0,
                end: 20,
                decision: Decision::NoSpeech,
            },
            Region {
                start: 20,
                end: 40,
                decision: Decision::Owned(vec![0]),
            },
            Region {
                start: 40,
                end: 80,
                decision: Decision::NoSpeech,
            },
        ];
        let protections = [Protection {
            track: Some(0),
            start: 55,
            end: 65,
        }];
        let envelope = gain_envelope(0, 80, &regions, &protections, 0.25, 4.0, 6.0);
        for (start, end) in [
            (0, 80),
            (20, 40),
            (40, 55),
            (55, 65),
            (65, 80),
            (38, 70),
            (79, 99),
        ] {
            let actual: Vec<_> = (start..end.min(80))
                .map(|sample| envelope.gain_at(sample))
                .collect();
            let expected = actual
                .iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), &gain| {
                    (low.min(gain), high.max(gain))
                });
            assert_eq!(envelope.gain_bounds(start, end), Some(expected));
        }
        assert_eq!(envelope.gain_bounds(80, 90), None);
        assert_eq!(envelope.gain_bounds(20, 20), None);
        assert_eq!(GainEnvelope::constant(0, 1.0).gain_bounds(0, 10), None);
        assert!(envelope.gain_bounds(0, 80).unwrap().1 > envelope.gain_at(0));
    }

    #[test]
    fn one_pole_matches_closed_form_and_retargets_from_current_gain() {
        let floor = 0.25;
        let regions = vec![
            Region {
                start: 0,
                end: 10,
                decision: Decision::NoSpeech,
            },
            Region {
                start: 10,
                end: 30,
                decision: Decision::Owned(vec![0]),
            },
            Region {
                start: 30,
                end: 40,
                decision: Decision::NoSpeech,
            },
            Region {
                start: 40,
                end: 60,
                decision: Decision::Owned(vec![0]),
            },
        ];
        let envelope = gain_envelope(0, 60, &regions, &[], floor, 10.0, 20.0);

        assert_eq!(envelope.gain_at(0), floor);
        for elapsed in [1, 5, 10] {
            let expected = 1.0 - (1.0 - floor) * (-(elapsed as f32) / 10.0).exp();
            assert!((envelope.gain_at(10 + elapsed - 1) - expected).abs() < 1e-6);
        }
        let before_close = envelope.gain_at(29);
        assert!(envelope.gain_at(30) < before_close);
        let expected_close = floor + (before_close - floor) * (-10.0_f32 / 20.0).exp();
        assert!((envelope.gain_at(39) - expected_close).abs() < 1e-6);
        assert!(envelope.gain_at(40) > envelope.gain_at(39));
        assert!(envelope.gain_at(40) < 1.0);
        assert_eq!(envelope.points.len(), 4);
    }

    #[test]
    fn legacy_transition_time_migrates_to_both_time_constants() {
        let parameters: AutomixParameters = serde_json::from_str(
            r#"{"speech_sensitivity":0.6,"bleed_discrimination_db":4.0,"attenuation_db":-12.0,"speech_padding_ms":300.0,"transition_ms":75.0}"#,
        ).unwrap();
        assert_eq!(parameters.open_ms, 75.0);
        assert_eq!(parameters.close_ms, 75.0);
    }
}

impl Default for AutomixParameters {
    fn default() -> Self {
        Self {
            speech_sensitivity: SPEECH_PROBABILITY,
            bleed_discrimination_db: ENERGY_MARGIN_DB,
            attenuation_db: -18.0,
            speech_padding_ms: 512.0,
            open_ms: 50.0,
            close_ms: 500.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GainEnvelope {
    sample_count: usize,
    points: Vec<GainPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GainPoint {
    sample: usize,
    gain: f32,
    alpha: f32,
    target: f32,
}

impl GainEnvelope {
    #[cfg(test)]
    pub(crate) fn constant(sample_count: usize, gain: f32) -> Self {
        Self {
            sample_count,
            points: vec![GainPoint {
                sample: 0,
                gain,
                alpha: 0.0,
                target: gain,
            }],
        }
    }

    pub fn gain_at(&self, sample: usize) -> f32 {
        let point = self.points[self
            .points
            .partition_point(|point| point.sample <= sample)
            .saturating_sub(1)];
        let elapsed = sample.saturating_sub(point.sample) as f32;
        point.target + (point.gain - point.target) * (1.0 - point.alpha).powf(elapsed)
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// Exact gain extrema in the half-open sample range, clipped to this envelope.
    pub(crate) fn gain_bounds(&self, start: usize, end: usize) -> Option<(f32, f32)> {
        let end = end.min(self.sample_count);
        if start >= end {
            return None;
        }
        let first = self.points.partition_point(|point| point.sample <= start);
        let last = self.points.partition_point(|point| point.sample < end);
        let mut bounds = (self.gain_at(start), self.gain_at(start));
        // One-pole segments are monotonic. Include both sides of every retarget.
        for sample in std::iter::once(end - 1).chain(
            self.points[first..last]
                .iter()
                .flat_map(|point| [point.sample - 1, point.sample]),
        ) {
            let gain = self.gain_at(sample);
            bounds = (bounds.0.min(gain), bounds.1.max(gain));
        }
        Some(bounds)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisPreview {
    pub stems: Vec<PathBuf>,
    pub regions: Vec<Region>,
    pub envelopes: Vec<GainEnvelope>,
    pub protections: Vec<Protection>,
    pub parameters: AutomixParameters,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Decision {
    Owned(Vec<usize>),
    Uncertain,
    NoSpeech,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub start: usize,
    pub end: usize,
    pub decision: Decision,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Protection {
    pub track: Option<usize>,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Validation {
    pub warnings: Vec<String>,
}

pub struct RunRequest {
    pub stems: Vec<PathBuf>,
    pub output_dir: PathBuf,
    pub regions: Vec<Region>,
    pub protections: Vec<Protection>,
    pub cancelled: Option<Arc<AtomicBool>>,
}

#[derive(Debug, PartialEq)]
pub struct RunResult {
    pub stems: Vec<PathBuf>,
    pub report: PathBuf,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct RenderBlocker(pub String);
impl fmt::Display for RenderBlocker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for RenderBlocker {}
impl From<io::Error> for RenderBlocker {
    fn from(value: io::Error) -> Self {
        Self(value.to_string())
    }
}
impl From<bwavfile::Error> for RenderBlocker {
    fn from(value: bwavfile::Error) -> Self {
        Self(value.to_string())
    }
}

pub fn run(request: RunRequest) -> Result<RunResult, RenderBlocker> {
    run_with_progress(request, None)
}

pub fn run_with_progress(
    request: RunRequest,
    progress: Option<Arc<AtomicU8>>,
) -> Result<RunResult, RenderBlocker> {
    run_with_parameters(request, progress, AutomixParameters::default())
}

pub fn run_with_parameters(
    request: RunRequest,
    progress: Option<Arc<AtomicU8>>,
    parameters: AutomixParameters,
) -> Result<RunResult, RenderBlocker> {
    if request.output_dir.exists() {
        return Err(RenderBlocker("output directory already exists".into()));
    }
    let (stems, validation) = load_and_validate(&request.stems)?;
    let sample_count = stems[0].samples.len();
    let references: Vec<Option<u64>> = stems
        .iter()
        .map(|stem| stem.bext.as_ref().map(|bext| bext.time_reference))
        .collect();
    let mut warnings = validation.warnings;
    if request
        .regions
        .iter()
        .any(|r| r.start > r.end || r.end > sample_count)
    {
        return Err(RenderBlocker(
            "ownership region is outside the session".into(),
        ));
    }
    if request.regions.iter().any(|region| {
        let Decision::Owned(owners) = &region.decision else {
            return false;
        };
        owners.is_empty()
            || owners.iter().any(|owner| *owner >= stems.len())
            || owners.iter().collect::<HashSet<_>>().len() != owners.len()
    }) {
        return Err(RenderBlocker(
            "ownership region has an invalid owner set".into(),
        ));
    }
    if request.protections.iter().any(|p| {
        p.start > p.end
            || (p.end != usize::MAX && p.end > sample_count)
            || p.track.is_some_and(|track| track >= stems.len())
    }) {
        return Err(RenderBlocker("protection is outside the session".into()));
    }
    let automated = request.regions.is_empty();
    let regions = if automated {
        vad::analyze_with_parameters(
            &stems
                .iter()
                .map(|stem| stem.samples.as_slice())
                .collect::<Vec<_>>(),
            parameters.speech_sensitivity,
            parameters.bleed_discrimination_db,
            (parameters.speech_padding_ms * 48_000.0 / 1000.0 / 1_536.0).round() as usize,
        )?
    } else {
        request.regions.clone()
    };
    if automated
        && regions.iter().any(|region| {
            region.end - region.start >= 10 * 48_000 && region.decision != Decision::Uncertain
        })
    {
        warnings.push("review unusually broad attenuation".into());
    }
    let uncertain_samples: usize = regions
        .iter()
        .filter(|region| region.decision == Decision::Uncertain)
        .map(|region| region.end - region.start)
        .sum();
    if automated && uncertain_samples > sample_count / 4 {
        warnings.push("review substantial conservative fallback".into());
    }

    let output = AtomicOutput::new(&request.output_dir)?;
    (|| {
        let mut names = Vec::new();
        for (track, stem) in stems.iter().enumerate() {
            check_cancelled(&request)?;
            let envelope = gain_envelope(
                track,
                sample_count,
                &regions,
                &request.protections,
                10.0_f32.powf(parameters.attenuation_db / 20.0),
                (parameters.open_ms * 48_000.0 / 1000.0).max(1.0),
                (parameters.close_ms * 48_000.0 / 1000.0).max(1.0),
            );
            let mut rendered = Vec::with_capacity(sample_count);
            for (sample, source) in stem.samples.iter().enumerate() {
                check_cancelled(&request)?;
                rendered.push(source * envelope.gain_at(sample));
            }
            let output_name = format!(
                "{:02}-{}",
                track + 1,
                stem.path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("stem.wav")
            );
            stem.write(&output.path(&output_name), &rendered)?;
            names.push(output_name);
            if let Some(progress) = &progress {
                progress.store(((track + 1) * 100 / stems.len()) as u8, Ordering::Relaxed);
            }
        }
        check_cancelled(&request)?;
        let report = format!(
            "{{\n  \"detector\": \"silero-vad-v5\",\n  \"model\": {{\"sha256\": \"1A153A22F4509E292A94E67D6F9B85E8DEB25B4988682B7E174C65279D8788E3\", \"license\": \"MIT\"}},\n  \"thresholds\": {{\"speech_probability\": {}, \"energy_margin_db\": {}, \"attenuation_db\": {}, \"speech_padding_ms\": {}, \"open_ms\": {}, \"close_ms\": {}}},\n  \"input\": {{\"sample_rate\": 48000, \"sample_count\": {sample_count}, \"channel_count\": 1, \"bext_time_reference\": {}, \"stems\": [{}]}},\n  \"output\": {{\"sample_rate\": 48000, \"sample_count\": {sample_count}, \"channel_count\": 1, \"stems\": [{}], \"invalid_samples\": 0, \"clipping_introduced\": false}},\n  \"regions\": [{}],\n  \"warnings\": [{}],\n  \"protections\": [{}]\n}}\n",
            parameters.speech_sensitivity,
            parameters.bleed_discrimination_db,
            parameters.attenuation_db,
            parameters.speech_padding_ms,
            parameters.open_ms,
            parameters.close_ms,
            references[0].map_or("null".into(), |reference| reference.to_string()),
            request
                .stems
                .iter()
                .map(|path| format!("\"{}\"", json_escape(&path.display().to_string())))
                .collect::<Vec<_>>()
                .join(", "),
            names
                .iter()
                .map(|name| format!("\"{}\"", json_escape(name)))
                .collect::<Vec<_>>()
                .join(", "),
            regions
                .iter()
                .flat_map(|region| report_regions(region, stems.len()))
                .collect::<Vec<_>>()
                .join(", "),
            warnings
                .iter()
                .map(|warning| format!("\"{}\"", json_escape(warning)))
                .collect::<Vec<_>>()
                .join(", "),
            request
                .protections
                .iter()
                .map(|protection| format!(
                    "{{\"track\": {}, \"start\": {}, \"end\": {}}}",
                    protection
                        .track
                        .map_or("null".into(), |track| track.to_string()),
                    protection.start,
                    protection.end
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
        output.write_report(&report)?;
        output.commit()?;
        Ok(RunResult {
            stems: names
                .into_iter()
                .map(|name| request.output_dir.join(name))
                .collect(),
            report: request.output_dir.join("report.json"),
            warnings,
        })
    })()
}

pub fn validate(stems: &[PathBuf]) -> Result<Validation, RenderBlocker> {
    load_and_validate(stems).map(|(_, validation)| validation)
}

pub fn analyze_preview(
    stems: &[PathBuf],
    protections: &[Protection],
    parameters: AutomixParameters,
    progress: Arc<AtomicU8>,
) -> Result<AnalysisPreview, RenderBlocker> {
    crate::PreviewCache::default().analyze(stems, protections, parameters, &progress, None)
}

pub(crate) fn gain_envelope(
    track: usize,
    sample_count: usize,
    regions: &[Region],
    protections: &[Protection],
    attenuation: f32,
    open_samples: f32,
    close_samples: f32,
) -> GainEnvelope {
    if sample_count == 0 {
        return GainEnvelope {
            sample_count,
            points: vec![],
        };
    }
    let mut target = target_gain_with_attenuation(track, 0, regions, protections, attenuation);
    let mut points = vec![GainPoint {
        sample: 0,
        gain: target,
        alpha: 0.0,
        target,
    }];
    // Targets are constant between edges. Count active protections so overlapping
    // ranges remain additive without scanning every protection at every edge.
    let mut edges: Vec<(usize, isize)> = regions
        .iter()
        .flat_map(|region| [(region.start, 0), (region.end, 0)])
        .filter(|(sample, _)| *sample < sample_count)
        .collect();
    for protection in protections.iter().filter(|p| {
        p.start < p.end && p.start < sample_count && p.track.is_none_or(|owner| owner == track)
    }) {
        edges.push((protection.start, 1));
        if protection.end < sample_count {
            edges.push((protection.end, -1));
        }
    }
    edges.sort_unstable_by_key(|&(sample, _)| sample);
    let mut protected = 0;
    let mut edge = 0;
    while edge < edges.len() {
        let sample = edges[edge].0;
        // Apply all starts and ends together to preserve half-open intervals.
        while edge < edges.len() && edges[edge].0 == sample {
            protected += edges[edge].1;
            edge += 1;
        }
        if sample == 0 {
            continue;
        }
        let next_target = if protected > 0 {
            1.0
        } else {
            target_gain_with_attenuation(track, sample, regions, &[], attenuation)
        };
        if next_target != target {
            let previous = points.last().unwrap();
            let elapsed = (sample - 1 - previous.sample) as f32;
            let gain = previous.target
                + (previous.gain - previous.target) * (1.0 - previous.alpha).powf(elapsed);
            target = next_target;
            let alpha = 1.0
                - (-1.0
                    / if target > gain {
                        open_samples
                    } else {
                        close_samples
                    })
                .exp();
            points.push(GainPoint {
                sample,
                gain: gain + alpha * (target - gain),
                alpha,
                target,
            });
        }
    }
    GainEnvelope {
        sample_count,
        points,
    }
}

pub(crate) fn load_and_validate(
    paths: &[PathBuf],
) -> Result<(Vec<Stem>, Validation), RenderBlocker> {
    if paths.is_empty() {
        return Err(RenderBlocker("session requires at least one stem".into()));
    }
    let stems: Vec<Stem> = paths
        .iter()
        .map(|path| Stem::read(path))
        .collect::<Result<_, _>>()?;
    let sample_count = stems[0].samples.len();
    if sample_count == 0 {
        return Err(RenderBlocker("stems contain no audio samples".into()));
    }
    if stems.iter().any(|stem| stem.samples.len() != sample_count) {
        return Err(RenderBlocker("stems have unequal sample counts".into()));
    }
    let references: Vec<Option<u64>> = stems
        .iter()
        .map(|stem| stem.bext.as_ref().map(|bext| bext.time_reference))
        .collect();
    let warnings = if references.iter().all(Option::is_none) {
        vec!["alignment could not be verified: no stems contain BEXT".into()]
    } else if references.iter().any(Option::is_none) {
        return Err(RenderBlocker(
            "mixed BEXT presence prevents alignment verification".into(),
        ));
    } else if references.iter().collect::<HashSet<_>>().len() != 1 {
        return Err(RenderBlocker("conflicting BEXT time references".into()));
    } else {
        vec![]
    };
    Ok((stems, Validation { warnings }))
}

fn report_regions(region: &Region, track_count: usize) -> Vec<String> {
    let (decision, owners, reasons, affected): (&str, String, &str, Vec<Option<usize>>) =
        match &region.decision {
            Decision::Owned(owners) => (
                "Owned",
                owners
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                "Silero speech and calibrated energy agree",
                (0..track_count)
                    .filter(|track| !owners.contains(track))
                    .map(Some)
                    .collect(),
            ),
            Decision::Uncertain => (
                "Uncertain",
                String::new(),
                "detector evidence is ambiguous; preserved at unity",
                vec![None],
            ),
            Decision::NoSpeech => (
                "NoSpeech",
                String::new(),
                "Silero and calibrated energy agree on non-speech",
                (0..track_count).map(Some).collect(),
            ),
        };
    affected
        .into_iter()
        .map(|track| {
            let warning = if region.end - region.start >= 10 * 48_000
                && region.decision != Decision::Uncertain
            {
                "\"unusually broad attenuation\""
            } else {
                "null"
            };
            format!(
                "{{\"start\": {}, \"end\": {}, \"decision\": \"{decision}\", \"owners\": [{owners}], \"affected_track\": {}, \"applied_attenuation_db\": {}, \"evidence_reasons\": [\"{reasons}\"], \"review_warning\": {warning}}}",
                region.start,
                region.end,
                track.map_or("null".into(), |track| track.to_string()),
                if matches!(region.decision, Decision::Uncertain) { 0 } else { -18 }
            )
        })
        .collect()
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| match character {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            character if character.is_control() => {
                format!("\\u{:04x}", character as u32).chars().collect()
            }
            character => vec![character],
        })
        .collect()
}

fn check_cancelled(request: &RunRequest) -> Result<(), RenderBlocker> {
    if request
        .cancelled
        .as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
    {
        Err(RenderBlocker("render cancelled".into()))
    } else {
        Ok(())
    }
}

fn target_gain_with_attenuation(
    track: usize,
    sample: usize,
    regions: &[Region],
    protections: &[Protection],
    attenuation: f32,
) -> f32 {
    if protections.iter().any(|p| {
        p.start <= sample && sample < p.end && p.track.is_none_or(|protected| protected == track)
    }) {
        return 1.0;
    }
    match regions
        .get(
            regions
                .partition_point(|region| region.start <= sample)
                .saturating_sub(1),
        )
        .filter(|region| sample < region.end)
        .map(|region| &region.decision)
    {
        Some(Decision::Owned(owners)) if !owners.contains(&track) => attenuation,
        Some(Decision::NoSpeech) => attenuation,
        _ => 1.0,
    }
}
