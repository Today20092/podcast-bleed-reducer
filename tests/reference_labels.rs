use rust_app::{Project, ResultState, TICKS_PER_SECOND, TimeRange};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn session() -> (PathBuf, Project) {
    session_frames(32_000)
}

fn session_frames(length: usize) -> (PathBuf, Project) {
    let root = std::env::temp_dir().join(format!(
        "cleanslate-references-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut paths = vec![];
    for track in 0..2 {
        let path = root.join(format!("speaker-{track}.wav"));
        let mut format = bwavfile::WaveFmt::new_pcm_mono(16_000, 32);
        format.tag = 3;
        let mut writer = bwavfile::WaveWriter::create(&path, format)
            .unwrap()
            .audio_frame_writer()
            .unwrap();
        let samples: Vec<_> = (0..length)
            .map(|frame| {
                let solo = if track == 0 {
                    8_000..16_000
                } else {
                    20_000..28_000
                };
                let gain = if solo.contains(&frame)
                    || (track == 0 && (480_000..488_000).contains(&frame))
                {
                    0.1
                } else {
                    0.0001
                };
                gain * (frame as f32 * 0.17).sin()
            })
            .collect();
        writer.write_frames(&samples).unwrap();
        writer.end().unwrap();
        paths.push(path);
    }
    let project = Project::create(root.join("session.cleanslate"), paths).unwrap();
    (root, project)
}

#[test]
fn reference_replacement_is_explicit_undoable_and_persisted() {
    let (root, mut project) = session();
    let speaker = project.sources[0].speaker_id;
    let range = TimeRange::new(TICKS_PER_SECOND / 2, TICKS_PER_SECOND).unwrap();
    let solo = project
        .add_reference_label(range, vec![speaker], &[])
        .unwrap();
    assert!(project.add_reference_label(range, vec![], &[]).is_err());
    assert_eq!(project.reference_labels().len(), 1);
    let room = project.add_reference_label(range, vec![], &[solo]).unwrap();
    assert_ne!(room, solo);
    assert!(project.reference_labels()[0].active_speakers.is_empty());
    assert!(project.undo_reference_labels().unwrap());
    assert_eq!(project.reference_labels()[0].id, solo);
    assert!(project.redo_reference_labels().unwrap());
    assert_eq!(project.reference_labels()[0].id, room);
    let reopened = Project::open(project.path()).unwrap();
    assert_eq!(reopened.reference_labels(), project.reference_labels());
    assert!(matches!(reopened.result(), ResultState::None));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn references_require_current_usable_solos_and_room_tone() {
    let (root, mut project) = session();
    let copies: Vec<_> = project
        .source_paths()
        .iter()
        .map(|path| {
            rust_app::prepare_analysis_audio_in(path, &root.join("audio-cache"), None).unwrap()
        })
        .collect();
    let missing = rust_app::reference_readiness(&project, &copies, None).unwrap();
    assert_eq!(missing.missing_speakers.len(), 2);
    assert!(!missing.room_tone);
    assert!(!missing.ready());
    let speakers: Vec<_> = project
        .sources
        .iter()
        .map(|source| source.speaker_id)
        .collect();
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND / 2, TICKS_PER_SECOND).unwrap(),
            vec![speakers[0]],
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 5 / 4, TICKS_PER_SECOND * 7 / 4).unwrap(),
            vec![speakers[1]],
            &[],
        )
        .unwrap();
    assert!(
        rust_app::reference_readiness(&project, &copies, None)
            .unwrap()
            .ready()
    );
    let bad = project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND / 4, TICKS_PER_SECOND / 2).unwrap(),
            vec![speakers[0]],
            &[],
        )
        .unwrap();
    let unusable = rust_app::reference_readiness(&project, &copies, None).unwrap();
    assert!(!unusable.ready());
    assert!(
        unusable
            .issues
            .iter()
            .any(|issue| issue.contains(&format!("Reference {bad}")))
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn evidence(
    root: &std::path::Path,
    copies: &[rust_app::PreparedAnalysisAudio],
    columns: [usize; 2],
) -> rust_app::model::DiarizationEvidence {
    use rust_app::model::{
        DiarizationEvidence, ProbabilityFrames, ProbabilityHistoryWriter, RuntimeProvenance,
    };
    let path = root.join(format!("history-{}-{}.f32", columns[0], columns[1]));
    let mut writer = ProbabilityHistoryWriter::create(&path).unwrap();
    let mut probabilities = vec![0.0; 201 * 8];
    for frame in 50..100 {
        probabilities[frame * 8 + columns[0]] = 0.9;
    }
    for frame in 125..175 {
        probabilities[frame * 8 + columns[1]] = 0.9;
    }
    writer
        .append(&ProbabilityFrames {
            start: 0,
            frames: 201,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities,
        })
        .unwrap();
    DiarizationEvidence {
        schema: 2,
        sources: copies.iter().map(|audio| audio.identity.clone()).collect(),
        input_sha256: copies
            .iter()
            .map(|audio| audio.audio_sha256().to_string())
            .collect(),
        history: writer.finish().unwrap(),
        runtime: RuntimeProvenance {
            revision: rust_app::model::RUNTIME_REVISION.into(),
            recipe_sha256: "1".repeat(64),
            binaries: Default::default(),
        },
        model_revision: rust_app::model::MODEL_REVISION.into(),
        model_sha256: rust_app::model::NEMOTRON.sha256.into(),
        preset: "v3-offline".into(),
        mix_recipe: "mean-global-peak-floor-one-v1".into(),
        mix_peak: 1.0,
        mix_gains: vec![0.5; 2],
        sample_count: 32_000,
        reported_seconds_per_frame: 0.01,
        compactions: 0,
        maximum_retained_frames: 201,
    }
}

#[test]
fn trusted_solos_resolve_swapped_arrival_channels_without_duplicate_identity() {
    let (root, mut project) = session();
    let speakers: Vec<_> = project
        .sources
        .iter()
        .map(|source| source.speaker_id)
        .collect();
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND / 2, TICKS_PER_SECOND).unwrap(),
            vec![speakers[0]],
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 5 / 4, TICKS_PER_SECOND * 7 / 4).unwrap(),
            vec![speakers[1]],
            &[],
        )
        .unwrap();
    let copies: Vec<_> = project
        .source_paths()
        .iter()
        .map(|path| {
            rust_app::prepare_analysis_audio_in(path, &root.join("audio-cache"), None).unwrap()
        })
        .collect();
    let swapped = evidence(&root, &copies, [6, 2]);
    let mapping = rust_app::resolve_speaker_mapping(&project, &copies, &swapped, None).unwrap();
    assert!(mapping.readiness.ready());
    assert_eq!(mapping.channels[&speakers[0]], Some(6));
    assert_eq!(mapping.channels[&speakers[1]], Some(2));
    let duplicate = evidence(&root, &copies, [6, 6]);
    let unresolved =
        rust_app::resolve_speaker_mapping(&project, &copies, &duplicate, None).unwrap();
    assert_eq!(unresolved.channels[&speakers[0]], None);
    assert_eq!(unresolved.channels[&speakers[1]], None);
    assert!(
        unresolved
            .issues
            .iter()
            .any(|issue| issue.reason.contains("same anonymous channel"))
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn long_gap_and_ambiguous_identity_require_review_without_reinference() {
    use rust_app::model::{ProbabilityFrames, ProbabilityHistoryWriter};
    let (root, mut project) = session_frames(512_000);
    let speakers: Vec<_> = project
        .sources
        .iter()
        .map(|source| source.speaker_id)
        .collect();
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND / 2, TICKS_PER_SECOND).unwrap(),
            vec![speakers[0]],
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 5 / 4, TICKS_PER_SECOND * 7 / 4).unwrap(),
            vec![speakers[1]],
            &[],
        )
        .unwrap();
    let copies: Vec<_> = project
        .source_paths()
        .iter()
        .map(|path| {
            rust_app::prepare_analysis_audio_in(path, &root.join("audio-cache"), None).unwrap()
        })
        .collect();
    let mut fixed = evidence(&root, &copies, [6, 2]);
    let mut rows = vec![0.0; 3201 * 8];
    for frame in 50..100 {
        rows[frame * 8 + 6] = 0.9;
    }
    for frame in 125..175 {
        rows[frame * 8 + 2] = 0.9;
    }
    for frame in 3000..3050 {
        rows[frame * 8 + 4] = 0.9;
    }
    let mut writer = ProbabilityHistoryWriter::create(&root.join("long-gap.f32")).unwrap();
    writer
        .append(&ProbabilityFrames {
            start: 0,
            frames: 3201,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities: rows,
        })
        .unwrap();
    fixed.history = writer.finish().unwrap();
    fixed.sample_count = 512_000;
    let before = rust_app::resolve_speaker_mapping(&project, &copies, &fixed, None).unwrap();
    let fixed_hash = fixed.history.sha256.clone();
    let copy_hashes: Vec<_> = copies
        .iter()
        .map(|copy| copy.audio_sha256().to_owned())
        .collect();
    project
        .add_reference_label(
            TimeRange::new(
                30 * TICKS_PER_SECOND,
                30 * TICKS_PER_SECOND + TICKS_PER_SECOND / 2,
            )
            .unwrap(),
            vec![speakers[0]],
            &[],
        )
        .unwrap();
    let after = rust_app::resolve_speaker_mapping(&project, &copies, &fixed, None).unwrap();
    assert_ne!(before.cache_key, after.cache_key);
    assert_eq!(after.channels[&speakers[0]], None);
    assert_eq!(after.channels[&speakers[1]], Some(2));
    assert!(
        after
            .issues
            .iter()
            .any(|issue| issue.reason.contains("recurrence"))
    );
    assert_eq!(fixed.history.sha256, fixed_hash);
    assert_eq!(
        copies
            .iter()
            .map(|copy| copy.audio_sha256().to_owned())
            .collect::<Vec<_>>(),
        copy_hashes
    );
    let cancelled = std::sync::atomic::AtomicBool::new(true);
    assert!(
        rust_app::resolve_speaker_mapping(&project, &copies, &fixed, Some(&cancelled)).is_err()
    );
    assert_eq!(project.reference_labels().len(), 4);
    let mut ambiguous = vec![0.0; 3201 * 8];
    for frame in 50..100 {
        ambiguous[frame * 8 + 6] = 0.9;
        ambiguous[frame * 8 + 7] = 0.9;
    }
    for frame in 125..175 {
        ambiguous[frame * 8 + 2] = 0.9;
    }
    let mut writer = ProbabilityHistoryWriter::create(&root.join("ambiguous.f32")).unwrap();
    writer
        .append(&ProbabilityFrames {
            start: 0,
            frames: 3201,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities: ambiguous,
        })
        .unwrap();
    fixed.history = writer.finish().unwrap();
    let ambiguity = rust_app::resolve_speaker_mapping(&project, &copies, &fixed, None).unwrap();
    assert_eq!(ambiguity.channels[&speakers[0]], None);
    assert!(
        ambiguity
            .issues
            .iter()
            .any(|issue| issue.reason.contains("ambiguous"))
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn label_membership_and_source_replacement_preserve_truth_but_stale_results() {
    let (root, mut project) = session();
    let speakers: Vec<_> = project
        .sources
        .iter()
        .map(|source| source.speaker_id)
        .collect();
    let range = TimeRange::new(TICKS_PER_SECOND / 2, TICKS_PER_SECOND).unwrap();
    assert!(
        project
            .add_reference_label(range, vec![rust_app::SpeakerId(999)], &[])
            .is_err()
    );
    assert!(
        project
            .add_reference_label(range, vec![speakers[0], speakers[0]], &[])
            .is_err()
    );
    let overlap = project
        .add_reference_label(range, speakers.clone(), &[])
        .unwrap();
    project
        .set_result(
            ResultState::Current {
                identity: "finished-analysis".into(),
            },
            None,
        )
        .unwrap();
    let truth = project.reference_labels().to_vec();
    assert!(project.undo_reference_labels().unwrap());
    assert_eq!(
        project.result(),
        &ResultState::Stale {
            identity: "finished-analysis".into()
        }
    );
    assert!(project.redo_reference_labels().unwrap());
    assert_eq!(project.reference_labels(), truth);
    assert_eq!(project.reference_labels()[0].id, overlap);
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    let truth = project.reference_labels().to_vec();
    let replacement = root.join("replacement.wav");
    std::fs::copy(&project.sources[1].path, &replacement).unwrap();
    project.relink(0, replacement).unwrap();
    assert_eq!(project.reference_labels(), truth);
    assert!(!project.reference_label_is_current(&truth[0]));
    assert!(
        project.undo_reference_labels().is_err(),
        "Old source-bound truth must not be silently restored onto changed audio"
    );
    let reopened = Project::open(project.path()).unwrap();
    assert_eq!(reopened.reference_labels(), truth);
    assert!(matches!(reopened.result(), ResultState::Stale { .. }));
    std::fs::remove_dir_all(root).unwrap();
}
