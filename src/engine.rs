use crate::{
    audio_io::{AtomicOutput, Stem},
    vad::{self, ENERGY_MARGIN_DB, SPEECH_PROBABILITY},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt, io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

const ATTENUATION: f32 = 0.125_892_53;
const TRANSITION_SAMPLES: f32 = 480.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewParameters {
    pub speech_sensitivity: f32,
    pub bleed_discrimination_db: f32,
    pub attenuation_db: f32,
    pub speech_padding_ms: f32,
    pub transition_ms: f32,
}

#[cfg(test)]
mod preview_tests {
    use super::*;

    #[test]
    fn cached_preview_envelope_matches_engine_transition_and_protection_logic() {
        let regions = vec![Region {
            start: 0,
            end: 16,
            decision: Decision::NoSpeech,
        }];
        let protections = vec![Protection {
            track: Some(0),
            start: 6,
            end: 9,
        }];
        let envelope = gain_envelope(0, 16, &regions, &protections, 0.25, 4.0);
        assert_eq!(envelope.gain_at(0), 0.8125);
        assert_eq!(envelope.gain_at(3), 0.25);
        assert!(envelope.gain_at(8) > envelope.gain_at(5));
        assert!(envelope.gain_at(12) < envelope.gain_at(9));
        assert!(envelope.points.len() < 16);
    }
}

impl Default for PreviewParameters {
    fn default() -> Self {
        Self {
            speech_sensitivity: SPEECH_PROBABILITY,
            bleed_discrimination_db: ENERGY_MARGIN_DB,
            attenuation_db: -18.0,
            speech_padding_ms: 512.0,
            transition_ms: 10.0,
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
    step: f32,
    target: f32,
}

impl GainEnvelope {
    pub fn gain_at(&self, sample: usize) -> f32 {
        let point = self.points[self
            .points
            .partition_point(|point| point.sample <= sample)
            .saturating_sub(1)];
        let elapsed = sample.saturating_sub(point.sample) as f32;
        let gain = point.gain + point.step * elapsed;
        if (point.gain - point.target).abs() > point.step.abs() * elapsed {
            gain
        } else {
            point.target
        }
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisPreview {
    pub stems: Vec<PathBuf>,
    pub regions: Vec<Region>,
    pub envelopes: Vec<GainEnvelope>,
    pub protections: Vec<Protection>,
    pub parameters: PreviewParameters,
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
        vad::analyze(
            &stems
                .iter()
                .map(|stem| stem.samples.as_slice())
                .collect::<Vec<_>>(),
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
                ATTENUATION,
                TRANSITION_SAMPLES,
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
            "{{\n  \"detector\": \"silero-vad-v5\",\n  \"model\": {{\"sha256\": \"1A153A22F4509E292A94E67D6F9B85E8DEB25B4988682B7E174C65279D8788E3\", \"license\": \"MIT\"}},\n  \"thresholds\": {{\"speech_probability\": {SPEECH_PROBABILITY}, \"energy_margin_db\": {ENERGY_MARGIN_DB}, \"attenuation_db\": -18, \"transition_samples\": 480}},\n  \"input\": {{\"sample_rate\": 48000, \"sample_count\": {sample_count}, \"channel_count\": 1, \"bext_time_reference\": {}, \"stems\": [{}]}},\n  \"output\": {{\"sample_rate\": 48000, \"sample_count\": {sample_count}, \"channel_count\": 1, \"stems\": [{}], \"invalid_samples\": 0, \"clipping_introduced\": false}},\n  \"regions\": [{}],\n  \"warnings\": [{}],\n  \"protections\": [{}]\n}}\n",
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
    parameters: PreviewParameters,
    progress: Arc<AtomicU8>,
) -> Result<AnalysisPreview, RenderBlocker> {
    progress.store(5, Ordering::Relaxed);
    let (stems, _) = load_and_validate(stems)?;
    let samples = stems
        .iter()
        .map(|stem| stem.samples.as_slice())
        .collect::<Vec<_>>();
    progress.store(20, Ordering::Relaxed);
    let regions = vad::analyze_with_parameters(
        &samples,
        parameters.speech_sensitivity,
        parameters.bleed_discrimination_db,
        (parameters.speech_padding_ms * 48_000.0 / 1000.0 / 1_536.0).round() as usize,
    )?;
    progress.store(80, Ordering::Relaxed);
    let attenuation = 10.0_f32.powf(parameters.attenuation_db / 20.0);
    let transition = (parameters.transition_ms * 48_000.0 / 1000.0).max(1.0);
    let envelopes = (0..stems.len())
        .map(|track| {
            gain_envelope(
                track,
                samples[0].len(),
                &regions,
                protections,
                attenuation,
                transition,
            )
        })
        .collect();
    progress.store(100, Ordering::Relaxed);
    Ok(AnalysisPreview {
        stems: stems.iter().map(|stem| stem.path.clone()).collect(),
        regions,
        envelopes,
        protections: protections.to_vec(),
        parameters,
    })
}

fn gain_envelope(
    track: usize,
    sample_count: usize,
    regions: &[Region],
    protections: &[Protection],
    attenuation: f32,
    transition_samples: f32,
) -> GainEnvelope {
    let mut points = vec![GainPoint {
        sample: 0,
        gain: 1.0,
        step: 0.0,
        target: 1.0,
    }];
    let (mut gain, mut target, mut step) = (1.0, 1.0, 0.0);
    for sample in 0..sample_count {
        let next_target =
            target_gain_with_attenuation(track, sample, regions, protections, attenuation);
        if next_target != target {
            target = next_target;
            step = (target - gain) / transition_samples;
        }
        gain = if (gain - target).abs() > step.abs() {
            gain + step
        } else {
            target
        };
        if points
            .last()
            .is_none_or(|point| point.step != step || point.target != target)
        {
            points.push(GainPoint {
                sample,
                gain,
                step,
                target,
            });
        }
    }
    GainEnvelope {
        sample_count,
        points,
    }
}

fn load_and_validate(paths: &[PathBuf]) -> Result<(Vec<Stem>, Validation), RenderBlocker> {
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
