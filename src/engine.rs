use crate::audio_io::{AtomicOutput, Stem};
use std::{
    collections::HashSet,
    fmt, io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

const ATTENUATION: f32 = 0.125_892_53;
const TRANSITION_SAMPLES: f32 = 480.0;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Protection {
    pub track: Option<usize>,
    pub start: usize,
    pub end: usize,
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
    if request.stems.is_empty() {
        return Err(RenderBlocker("session requires at least one stem".into()));
    }
    if request.output_dir.exists() {
        return Err(RenderBlocker("output directory already exists".into()));
    }
    let stems: Vec<Stem> = request
        .stems
        .iter()
        .map(|path| Stem::read(path))
        .collect::<Result<_, _>>()?;
    let sample_count = stems[0].samples.len();
    if stems.iter().any(|stem| stem.samples.len() != sample_count) {
        return Err(RenderBlocker("stems have unequal sample counts".into()));
    }
    let references: Vec<Option<u64>> = stems
        .iter()
        .map(|stem| stem.bext.as_ref().map(|bext| bext.time_reference))
        .collect();
    let warnings: Vec<String> = if references.iter().all(Option::is_none) {
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
    if request
        .regions
        .iter()
        .any(|r| r.start > r.end || r.end > sample_count)
    {
        return Err(RenderBlocker(
            "ownership region is outside the session".into(),
        ));
    }
    if request.protections.iter().any(|p| {
        p.start > p.end || p.end > sample_count || p.track.is_some_and(|track| track >= stems.len())
    }) {
        return Err(RenderBlocker("protection is outside the session".into()));
    }

    let output = AtomicOutput::new(&request.output_dir)?;
    (|| {
        let mut names = Vec::new();
        for (track, stem) in stems.iter().enumerate() {
            check_cancelled(&request)?;
            let (mut gain, mut target, mut step) = (1.0, 1.0, 0.0);
            let mut rendered = Vec::with_capacity(sample_count);
            for (sample, source) in stem.samples.iter().enumerate() {
                check_cancelled(&request)?;
                let next_target =
                    target_gain(track, sample, &request.regions, &request.protections);
                if next_target != target {
                    target = next_target;
                    step = (target - gain) / TRANSITION_SAMPLES;
                }
                if (gain - target).abs() > step.abs() {
                    gain += step;
                } else {
                    gain = target;
                }
                rendered.push(source * gain);
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
        }
        check_cancelled(&request)?;
        let report = format!(
            "{{\n  \"attenuation_db\": -18,\n  \"sample_rate\": 48000,\n  \"sample_count\": {sample_count},\n  \"stems\": [{}],\n  \"warnings\": [{}]\n}}\n",
            names
                .iter()
                .map(|name| format!("\"{}\"", json_escape(name)))
                .collect::<Vec<_>>()
                .join(", "),
            warnings
                .iter()
                .map(|warning| format!("\"{}\"", json_escape(warning)))
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

fn target_gain(track: usize, sample: usize, regions: &[Region], protections: &[Protection]) -> f32 {
    if protections.iter().any(|p| {
        p.start <= sample && sample < p.end && p.track.is_none_or(|protected| protected == track)
    }) {
        return 1.0;
    }
    match regions
        .iter()
        .find(|region| region.start <= sample && sample < region.end)
        .map(|region| &region.decision)
    {
        Some(Decision::Owned(owners)) if !owners.contains(&track) => ATTENUATION,
        Some(Decision::NoSpeech) => ATTENUATION,
        _ => 1.0,
    }
}
