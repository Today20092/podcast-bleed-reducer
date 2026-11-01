//! Offline bridge to the maintained desktop analysis, used only by background workers.
use rust_app::{
    AudioSource, EditorAnalysisWorker, EffectiveEnvelopeCache, GainEnvelope, Project, Rounding,
    SourceTimebase, SpeakerId, TimeRange,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelRange {
    pub start: i64,
    pub end: i64,
    pub mask: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub reduction_db: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            reduction_db: -12.0,
            attack_ms: 15.0,
            release_ms: 100.0,
        }
    }
}

impl Settings {
    fn parameters(self) -> Result<rust_app::AutomixParameters, String> {
        if !self.reduction_db.is_finite()
            || !(-120.0..=0.0).contains(&self.reduction_db)
            || !self.attack_ms.is_finite()
            || !(0.0..=60000.0).contains(&self.attack_ms)
            || !self.release_ms.is_finite()
            || !(0.0..=60000.0).contains(&self.release_ms)
        {
            return Err("Session sound settings are invalid.".into());
        }
        Ok(rust_app::AutomixParameters {
            attenuation_db: self.reduction_db,
            open_ms: self.attack_ms,
            close_ms: self.release_ms,
            ..rust_app::AutomixParameters::default()
        })
    }
}

#[derive(Clone, Debug)]
pub struct Analysis {
    pub origin: i64,
    pub end: i64,
    pub rate: u32,
    pub envelopes: Vec<GainEnvelope>,
}

impl Analysis {
    /// Source-time coverage is half-open; unknown audio always retains unity.
    pub fn gain_at(&self, member: usize, source_sample: i64) -> f32 {
        if source_sample < self.origin || source_sample >= self.end {
            return 1.0;
        }
        let Some(local) = source_sample
            .checked_sub(self.origin)
            .and_then(|sample| usize::try_from(sample).ok())
        else {
            return 1.0;
        };
        let Some(envelope) = self.envelopes.get(member) else {
            return 1.0;
        };
        if envelope.sample_rate() != self.rate || local >= envelope.sample_count() {
            return 1.0;
        }
        let gain = envelope.gain_at(local);
        if gain.is_finite() && (0.0..=1.0).contains(&gain) {
            gain
        } else {
            1.0
        }
    }
}

fn cancelled(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Session analysis cancelled.".into())
    } else {
        Ok(())
    }
}

type TranslatedLabels = Vec<(TimeRange, Vec<SpeakerId>)>;

fn translate_labels(
    labels: &[LabelRange],
    members: usize,
    origin: i64,
    grid: SourceTimebase,
) -> Result<(i64, TranslatedLabels), String> {
    if !(1..=8).contains(&members)
        || !(8000..=192000).contains(&grid.sample_rate)
        || grid.frame_count == 0
    {
        return Err("Session analysis requires 1–8 tracks on a nonempty native audio grid.".into());
    }
    let length = i64::try_from(grid.frame_count).map_err(|_| "Captured session is too long.")?;
    let end = origin
        .checked_add(length)
        .ok_or("Captured source-time range overflows.")?;
    let allowed = ((1_u16 << members) - 1) as u8;
    let mut labels = labels.to_vec();
    labels.sort_by_key(|label| (label.start, label.end));
    let mut previous_end = origin;
    let mut translated = Vec::with_capacity(labels.len());
    for label in labels {
        if label.start < origin
            || label.end > end
            || label.start >= label.end
            || label.start < previous_end
            || label.mask & !allowed != 0
        {
            return Err(
                "Examples must be nonoverlapping, inside captured audio, and name existing microphones."
                    .into(),
            );
        }
        let local_start = label
            .start
            .checked_sub(origin)
            .ok_or("Example time overflow.")?;
        let local_end = label
            .end
            .checked_sub(origin)
            .ok_or("Example time overflow.")?;
        let range = TimeRange::new(
            grid.time_at_frame(local_start as u64, Rounding::Floor)
                .map_err(|error| error.to_string())?,
            grid.time_at_frame(local_end as u64, Rounding::Ceil)
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        if translated
            .last()
            .is_some_and(|(previous, _): &(TimeRange, Vec<SpeakerId>)| previous.end > range.start)
        {
            return Err("Examples overlap on the analysis time grid.".into());
        }
        let speakers = (0..members)
            .filter(|&index| label.mask & (1_u8 << index) != 0)
            .map(|index| SpeakerId(index as u64 + 1))
            .collect();
        translated.push((range, speakers));
        previous_end = label.end;
    }
    Ok((end, translated))
}

static NEXT_JOB: AtomicU64 = AtomicU64::new(0);

fn job_directory(cache: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(cache).map_err(|error| error.to_string())?;
    for _ in 0..1024 {
        let sequence = NEXT_JOB.fetch_add(1, Ordering::Relaxed);
        let path = cache.join(format!("analysis-{}-{sequence}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Could not allocate a fresh analysis job directory.".into())
}

/// Analyze immutable, synchronized WAV snapshots containing no capture gaps.
/// Channel zero supplies mono evidence; the caller applies each resulting gain to both channels.
/// The capture owner must validate coverage/revisions before publishing this result.
pub fn analyze(
    paths: &[PathBuf],
    labels: &[LabelRange],
    origin: i64,
    rate: u32,
    cache: &Path,
    cancel: &AtomicBool,
    settings: Settings,
) -> Result<Analysis, String> {
    cancelled(cancel)?;
    let parameters = settings.parameters()?;
    if !(1..=8).contains(&paths.len()) {
        return Err("Choose between one and eight microphone captures.".into());
    }
    let mut sources = Vec::with_capacity(paths.len());
    let mut grid: Option<SourceTimebase> = None;
    for (index, path) in paths.iter().enumerate() {
        cancelled(cancel)?;
        let source = AudioSource {
            path: path.clone(),
            channel: Some(0),
            name: Some(format!("Microphone {}", index + 1)),
        };
        let metadata = source
            .metadata(Some(cancel))
            .map_err(|error| error.to_string())?;
        if metadata.channels != 1
            || metadata.grid.sample_rate != rate
            || grid.is_some_and(|expected| expected != metadata.grid)
        {
            return Err("Captured microphones need matching native rates and complete equal-length coverage.".into());
        }
        grid = Some(metadata.grid);
        sources.push(source);
    }
    let grid = grid.ok_or("No captured microphones.")?;
    let (end, translated) = translate_labels(labels, sources.len(), origin, grid)?;
    cancelled(cancel)?;
    let job = job_directory(cache)?;
    let mut project = Project::create_from_sources(job.join("session.cleanslate"), sources)
        .map_err(|error| error.to_string())?;
    project
        .set_parameters(parameters)
        .map_err(|error| error.to_string())?;
    project
        .set_supplied_alignment(true)
        .map_err(|error| error.to_string())?;
    for (range, speakers) in translated {
        cancelled(cancel)?;
        project
            .add_reference_label(range, speakers, &[])
            .map_err(|error| error.to_string())?;
    }
    let progress = AtomicU8::new(0);
    let result = EditorAnalysisWorker::default()
        .analyze(&project, &cache.join("prepared"), &progress, cancel)
        .map_err(|error| error.to_string())?;
    cancelled(cancel)?;
    let effective = result
        .install(
            &mut project,
            &mut EffectiveEnvelopeCache::default(),
            Some(cancel),
        )
        .map_err(|error| error.to_string())?;
    cancelled(cancel)?;
    if !effective.analysis_current
        || effective.envelopes.len() != paths.len()
        || effective.envelopes.iter().any(|envelope| {
            envelope.sample_rate() != rate || envelope.sample_count() as u64 != grid.frame_count
        })
    {
        return Err("Analysis returned incompatible source-time envelopes.".into());
    }
    Ok(Analysis {
        origin,
        end,
        rate,
        envelopes: effective.envelopes.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_native_time_room_solo_overlap_and_unlabelled_gaps() {
        let grid = SourceTimebase::new(48000, 48000).unwrap();
        let labels = [
            LabelRange {
                start: 12000,
                end: 24000,
                mask: 3,
            },
            LabelRange {
                start: 0,
                end: 6000,
                mask: 0,
            },
            LabelRange {
                start: 24000,
                end: 48000,
                mask: 1,
            },
        ];
        let (end, translated) = translate_labels(&labels, 2, 0, grid).unwrap();
        assert_eq!(end, 48000);
        assert!(translated[0].1.is_empty());
        assert_eq!(translated[1].1, [SpeakerId(1), SpeakerId(2)]);
        assert_eq!(translated[2].1, [SpeakerId(1)]);
        assert!(translated[0].0.end < translated[1].0.start);
        assert_eq!(translated[2].0.end, rust_app::TICKS_PER_SECOND);
        let shifted: Vec<_> = labels
            .iter()
            .map(|label| LabelRange {
                start: label.start - 96000,
                end: label.end - 96000,
                mask: label.mask,
            })
            .collect();
        let (negative_end, negative) = translate_labels(&shifted, 2, -96000, grid).unwrap();
        assert_eq!(negative_end, -48000);
        assert_eq!(translated, negative);
    }

    #[test]
    fn rejects_stale_outside_overlapping_labels_and_overflow() {
        let grid = SourceTimebase::new(44100, 44100).unwrap();
        let valid = LabelRange {
            start: 100,
            end: 200,
            mask: 1,
        };
        for label in [
            LabelRange { start: -1, ..valid },
            LabelRange {
                end: 44101,
                ..valid
            },
            LabelRange { end: 100, ..valid },
            LabelRange { mask: 4, ..valid },
        ] {
            assert!(translate_labels(&[label], 2, 0, grid).is_err());
        }
        assert!(translate_labels(&[valid, valid], 2, 0, grid).is_err());
        assert!(translate_labels(&[], 2, i64::MAX, grid).is_err());
        assert!(translate_labels(&[], 0, 0, grid).is_err());
        assert!(translate_labels(&[], 9, 0, grid).is_err());
        assert!(
            translate_labels(
                &[],
                2,
                0,
                SourceTimebase {
                    sample_rate: 0,
                    frame_count: 10
                }
            )
            .is_err()
        );
        let all = LabelRange { mask: 255, ..valid };
        assert_eq!(
            translate_labels(&[all], 8, 0, grid).unwrap().1[0].1.len(),
            8
        );
    }

    #[test]
    fn missing_coverage_or_member_preserves_original() {
        let analysis = Analysis {
            origin: 48000,
            end: 96000,
            rate: 48000,
            envelopes: vec![GainEnvelope::constant_at_rate(48000, 48000, 0.25).unwrap()],
        };
        assert_eq!(analysis.gain_at(0, 47999), 1.0);
        assert_eq!(analysis.gain_at(0, 48000), 0.25);
        assert_eq!(analysis.gain_at(0, 95999), 0.25);
        assert_eq!(analysis.gain_at(0, 96000), 1.0);
        assert_eq!(analysis.gain_at(1, 50000), 1.0);
        assert_eq!(analysis.gain_at(0, i64::MIN), 1.0);
        assert_eq!(analysis.gain_at(0, i64::MAX), 1.0);
    }

    #[test]
    fn cancelled_job_stops_before_reading_or_creating_files() {
        let cancel = AtomicBool::new(true);
        assert_eq!(
            analyze(
                &[PathBuf::from("missing.wav")],
                &[],
                0,
                48000,
                Path::new("unused"),
                &cancel,
                Settings::default()
            )
            .unwrap_err(),
            "Session analysis cancelled."
        );
    }

    #[test]
    fn sound_settings_reach_the_maintained_parameters_and_reject_invalid_values() {
        let settings = Settings {
            reduction_db: -24.0,
            attack_ms: 20.0,
            release_ms: 250.0,
        };
        let parameters = settings.parameters().unwrap();
        assert_eq!(parameters.attenuation_db, -24.0);
        assert_eq!(parameters.open_ms, 20.0);
        assert_eq!(parameters.close_ms, 250.0);
        assert_eq!(
            parameters.minimum_gap_ms,
            rust_app::AutomixParameters::default().minimum_gap_ms
        );
        for invalid in [
            Settings {
                reduction_db: f32::NAN,
                ..settings
            },
            Settings {
                reduction_db: 1.0,
                ..settings
            },
            Settings {
                attack_ms: -1.0,
                ..settings
            },
            Settings {
                release_ms: f32::INFINITY,
                ..settings
            },
        ] {
            assert!(invalid.parameters().is_err());
        }
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Analysis>();
    }

    #[test]
    fn complete_stereo_capture_bridge_runs_maintained_analysis_and_preserves_truth() {
        use sha2::{Digest, Sha256};
        use std::io::Write;

        // Match the maintained supervised fixture's varied direct/bleed relationships.
        // This exercises the real decoder/resampler/Silero/model/envelope pipeline, not sound quality.
        const RATE: u32 = 48000;
        const FRAMES: usize = RATE as usize * 4;
        let root = std::env::temp_dir().join(format!(
            "cleanslate-plugin-analysis-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let mut random = 7654321_u32;
        let noise: Vec<f32> = (0..64000)
            .map(|_| {
                random = random.wrapping_mul(1664525).wrapping_add(1013904223);
                ((random >> 8) as f32 / 16777216.0 - 0.5) * 0.2
            })
            .collect();
        let mut paths = Vec::new();
        let mut identities = Vec::new();
        for member in 0..2 {
            let path = root.join(format!("microphone-{member}.wav"));
            let mut bytes = Vec::with_capacity(44 + FRAMES * 8);
            let length = (FRAMES * 8) as u32;
            bytes.extend_from_slice(b"RIFF");
            bytes.extend_from_slice(&(length + 36).to_le_bytes());
            bytes.extend_from_slice(b"WAVEfmt ");
            bytes.extend_from_slice(&16_u32.to_le_bytes());
            bytes.extend_from_slice(&3_u16.to_le_bytes());
            bytes.extend_from_slice(&2_u16.to_le_bytes());
            bytes.extend_from_slice(&RATE.to_le_bytes());
            bytes.extend_from_slice(&(RATE * 8).to_le_bytes());
            bytes.extend_from_slice(&8_u16.to_le_bytes());
            bytes.extend_from_slice(&32_u16.to_le_bytes());
            bytes.extend_from_slice(b"data");
            bytes.extend_from_slice(&length.to_le_bytes());
            for frame in 0..FRAMES {
                let index = frame / 3;
                let active_a = (8000..24000).contains(&index) || (48000..60000).contains(&index);
                let active_b = (32000..44000).contains(&index) || (52000..60000).contains(&index);
                let a = noise[index] * if active_a { 1.0 } else { 0.0001 };
                let b = noise[(index + 700) % noise.len()] * if active_b { 1.0 } else { 0.0001 };
                let sample = if member == 0 {
                    a + b * 0.1
                } else {
                    b + a * 0.2
                };
                bytes.extend_from_slice(&sample.to_le_bytes());
                // An accidental stereo average would erase the evidence and fail readiness.
                bytes.extend_from_slice(&(-sample).to_le_bytes());
            }
            let mut file = fs::File::create_new(&path).unwrap();
            file.write_all(&bytes).unwrap();
            file.sync_all().unwrap();
            identities.push(<[u8; 32]>::from(Sha256::digest(&bytes)));
            paths.push(path);
        }
        let origin = i64::from(RATE) * 10 + 137;
        let labels: Vec<_> = [
            (0, RATE / 4, 0),
            (RATE / 2, RATE * 3 / 2, 1),
            (RATE * 2, RATE * 11 / 4, 2),
            (RATE * 13 / 4, RATE * 29 / 8, 3),
        ]
        .into_iter()
        .map(|(start, end, mask)| LabelRange {
            start: origin + i64::from(start),
            end: origin + i64::from(end),
            mask,
        })
        .collect();
        let analysis = analyze(
            &paths,
            &labels,
            origin,
            RATE,
            &root.join("analysis"),
            &AtomicBool::new(false),
            Settings::default(),
        )
        .unwrap();
        assert_eq!(analysis.origin, origin);
        assert_eq!(analysis.end, origin + FRAMES as i64);
        assert_eq!(analysis.rate, RATE);
        assert_eq!(analysis.envelopes.len(), 2);
        for (member, path) in paths.iter().enumerate() {
            assert_eq!(analysis.envelopes[member].sample_count(), FRAMES);
            for label in labels
                .iter()
                .filter(|label| label.mask & (1 << member) != 0)
            {
                for sample in label.start..label.end {
                    assert_eq!(
                        analysis.gain_at(member, sample),
                        1.0,
                        "Explicit positive truth must retain exact unity: mic {member}, source sample {sample}"
                    );
                }
            }
            for local in (0..FRAMES).step_by(317) {
                let gain = analysis.gain_at(member, origin + local as i64);
                assert!(gain.is_finite() && (0.0..=1.0).contains(&gain));
            }
            assert_eq!(analysis.gain_at(member, origin - 1), 1.0);
            assert_eq!(analysis.gain_at(member, analysis.end), 1.0);
            let after: [u8; 32] = Sha256::digest(fs::read(path).unwrap()).into();
            assert_eq!(after, identities[member], "Captured source bytes changed");
        }
        // A fresh test directory owns every generated file; never clean an unchecked path.
        let cleanup = root.canonicalize().unwrap();
        assert!(cleanup.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        assert!(
            cleanup
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("cleanslate-plugin-analysis-")
        );
        fs::remove_dir_all(cleanup).unwrap();
    }
}
