use rust_app::{Project, ResultState, TICKS_PER_SECOND, TimeRange};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn session() -> (PathBuf, Project) {
    session_frames(32_000)
}

fn session_frames(length: usize) -> (PathBuf, Project) {
    static NEXT_SESSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "cleanslate-references-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT_SESSION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
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
