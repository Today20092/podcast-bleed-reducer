use crate::audio_io::waveform;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewWarning {
    pub start: crate::TimePosition,
    pub end: crate::TimePosition,
    pub label: String,
    pub affected_tracks: Vec<usize>,
    region: usize,
    analysis_identity: Option<String>,
}

impl ReviewWarning {
    pub fn key(&self) -> String {
        match &self.analysis_identity {
            Some(identity) => format!(
                "warning:{identity}:{}:{}:{:?}:{}",
                self.start, self.end, self.affected_tracks, self.label
            ),
            None => format!("warning:{}:{}:{}", self.start, self.end, self.label),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Comparison {
    #[default]
    Original,
    Processed,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ReviewRegion {
    pub start: crate::TimePosition,
    pub end: crate::TimePosition,
    pub decision: ReportedDecision,
    pub owners: Vec<usize>,
    pub affected_track: Option<usize>,
    pub applied_attenuation_db: i32,
    pub evidence_reasons: Vec<String>,
    pub review_warning: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum ReportedDecision {
    Owned,
    Uncertain,
    NoSpeech,
}

impl ReviewRegion {
    pub fn key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.start,
            self.end,
            self.affected_track
                .map_or_else(|| "all".into(), |track| track.to_string())
        )
    }
}

#[derive(Deserialize)]
struct Report {
    regions: Vec<ReviewRegion>,
    output: Output,
}

#[derive(Deserialize)]
struct Output {
    stems: Vec<String>,
}

#[derive(Default)]
pub struct ReviewWorkspace {
    pub regions: Vec<ReviewRegion>,
    pub warnings: Vec<ReviewWarning>,
    pub output_stems: Vec<String>,
    pub waveforms: Vec<Vec<(f32, f32)>>,
    selected: Option<usize>,
    selected_warning: Option<usize>,
    playhead: crate::TimePosition,
    selection: Option<(crate::TimePosition, crate::TimePosition)>,
    highlighted_tracks: Vec<usize>,
    comparison: Comparison,
}

impl ReviewWorkspace {
    pub fn from_ownership(ownership: &crate::TemporalOwnership) -> Self {
        let regions: Vec<_> = ownership
            .regions()
            .iter()
            .map(|region| {
                let owners: Vec<_> = region
                    .states
                    .iter()
                    .enumerate()
                    .filter_map(|(i, s)| (*s == crate::TemporalTrackState::Owned).then_some(i))
                    .collect();
                let decision = if region
                    .states
                    .contains(&crate::TemporalTrackState::UncertainUnity)
                {
                    ReportedDecision::Uncertain
                } else if owners.is_empty() {
                    ReportedDecision::NoSpeech
                } else {
                    ReportedDecision::Owned
                };
                ReviewRegion {
                    start: region.start,
                    end: region.end,
                    decision,
                    owners,
                    affected_track: None,
                    applied_attenuation_db: 0,
                    evidence_reasons: vec![],
                    review_warning: None,
                }
            })
            .collect();
        let mut grouped = BTreeMap::<(u64, u64, String), Vec<usize>>::new();
        for warning in &ownership.warnings {
            let tracks = grouped
                .entry((warning.start, warning.end, warning.reason.clone()))
                .or_default();
            tracks.extend(
                warning
                    .tracks
                    .iter()
                    .filter_map(|id| ownership.tracks.iter().position(|track| track == id)),
            );
        }
        let warnings = grouped
            .into_iter()
            .map(|((start, end, label), mut affected_tracks)| {
                affected_tracks.sort_unstable();
                affected_tracks.dedup();
                let region = regions.partition_point(|region| region.end <= start);
                ReviewWarning {
                    start,
                    end,
                    label,
                    affected_tracks,
                    region,
                    analysis_identity: Some(ownership.cache_key.clone()),
                }
            })
            .collect();
        Self {
            regions,
            warnings,
            ..Self::default()
        }
    }
    pub fn open(path: &Path, sources: &[PathBuf]) -> io::Result<Self> {
        let report: Report = serde_json::from_slice(&fs::read(path)?).map_err(io::Error::other)?;
        let mut warnings = BTreeMap::<
            (crate::TimePosition, crate::TimePosition, String),
            (usize, Vec<usize>),
        >::new();
        for (index, region) in report.regions.iter().enumerate() {
            if let Some(label) = &region.review_warning {
                let (_, tracks) = warnings
                    .entry((region.start, region.end, label.clone()))
                    .or_insert((index, vec![]));
                if let Some(track) = region.affected_track {
                    tracks.push(track);
                }
            }
        }
        let warnings = warnings
            .into_iter()
            .map(|((start, end, label), (region, mut affected_tracks))| {
                affected_tracks.sort_unstable();
                affected_tracks.dedup();
                ReviewWarning {
                    start,
                    end,
                    label,
                    affected_tracks,
                    region,
                    analysis_identity: None,
                }
            })
            .collect();
        Ok(Self {
            regions: report.regions,
            warnings,
            output_stems: report.output.stems,
            waveforms: sources
                .iter()
                .map(|path| waveform(path, 600).map_err(io::Error::other))
                .collect::<io::Result<_>>()?,
            ..Self::default()
        })
    }

    pub fn select(&mut self, index: usize) {
        let Some(region) = self.regions.get(index) else {
            return;
        };
        self.selected = Some(index);
        self.selected_warning = None;
        self.playhead = region.start;
        self.selection = Some((region.start, region.end));
        self.highlighted_tracks = region.affected_track.into_iter().collect();
    }

    pub fn selected_region(&self) -> Option<&ReviewRegion> {
        self.selected.and_then(|index| self.regions.get(index))
    }
    pub fn playhead(&self) -> crate::TimePosition {
        self.playhead
    }

    pub fn select_warning(&mut self, index: usize) {
        let Some(warning) = self.warnings.get(index) else {
            return;
        };
        self.selected = Some(warning.region);
        self.selected_warning = Some(index);
        self.playhead = warning.start;
        self.selection = Some((warning.start, warning.end));
        self.highlighted_tracks.clone_from(&warning.affected_tracks);
    }

    pub fn selected_warning(&self) -> Option<&ReviewWarning> {
        self.selected_warning
            .and_then(|index| self.warnings.get(index))
    }
    pub fn selected_key(&self) -> Option<String> {
        self.selected_warning()
            .map(ReviewWarning::key)
            .or_else(|| self.selected_region().map(ReviewRegion::key))
    }
    pub fn seek(&mut self, sample: crate::TimePosition) {
        self.playhead = sample;
    }
    pub fn selection(&self) -> Option<(crate::TimePosition, crate::TimePosition)> {
        self.selection
    }
    pub fn highlighted_tracks(&self) -> &[usize] {
        &self.highlighted_tracks
    }
    pub fn comparison(&self) -> Comparison {
        self.comparison
    }
    pub fn comparison_mut(&mut self) -> &mut Comparison {
        &mut self.comparison
    }
    pub fn set_selection(&mut self, start: crate::TimePosition, end: crate::TimePosition) {
        self.selection = Some((start.min(end), end.max(start)));
    }
    pub fn set_comparison(&mut self, comparison: Comparison) {
        self.comparison = comparison;
    }
}
