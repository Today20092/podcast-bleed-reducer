use rust_app::{AutomixParameters, Project, RenderState, ReviewState, WorkflowStep};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cleanslate-project-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn project_reopens_with_workflow_and_review_state() {
    let dir = temp_dir();
    let source = dir.join("voice.wav");
    fs::write(&source, b"source audio identity").unwrap();
    let project_path = dir.join("episode.cleanslate");
    let render_report = dir.join("render-1/report.json");
    fs::create_dir(dir.join("render-1")).unwrap();
    fs::write(&render_report, b"{}").unwrap();

    let mut project = Project::create(&project_path, vec![source]).unwrap();
    project
        .replace_protections(vec![rust_app::Protection {
            track: Some(0),
            start: 10,
            end: 20,
        }])
        .unwrap();
    project
        .set_review_state("broad-attenuation", ReviewState::Reviewed)
        .unwrap();
    let parameters = AutomixParameters {
        attenuation_db: -12.0,
        ..AutomixParameters::default()
    };
    project.set_parameters(parameters).unwrap();
    project.mark_render_current(&render_report).unwrap();

    let reopened = Project::open(&project_path).unwrap();
    assert_eq!(reopened.step(), WorkflowStep::Review);
    assert_eq!(reopened.protections().len(), 1);
    assert_eq!(reopened.parameters(), parameters);
    assert_eq!(
        reopened.review_states()["broad-attenuation"],
        ReviewState::Reviewed
    );
    assert_eq!(
        reopened.render(),
        &RenderState::Current {
            identity: render_report
        }
    );
    assert!(reopened.sources[0].available);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn changed_source_requires_relink_and_full_revalidation() {
    let dir = temp_dir();
    let source = dir.join("voice.wav");
    let unchanged = dir.join("guest.wav");
    fs::write(&source, b"original").unwrap();
    fs::write(&unchanged, b"unchanged").unwrap();
    let project_path = dir.join("episode.cleanslate");
    let mut project = Project::create(&project_path, vec![source.clone(), unchanged]).unwrap();
    project
        .mark_render_current(dir.join("missing-report.json"))
        .unwrap();

    fs::write(&source, b"changed source").unwrap();
    let reopened = Project::open(&project_path).unwrap();

    assert!(!reopened.sources[0].available);
    assert!(reopened.sources[1].available);
    assert_eq!(reopened.step(), WorkflowStep::Import);
    assert_eq!(reopened.render(), &RenderState::Stale);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_committed_render_reopens_stale() {
    let dir = temp_dir();
    let source = dir.join("voice.wav");
    fs::write(&source, b"original").unwrap();
    let project_path = dir.join("episode.cleanslate");
    let mut project = Project::create(&project_path, vec![source]).unwrap();
    project
        .mark_render_current(dir.join("removed-render/report.json"))
        .unwrap();

    let reopened = Project::open(&project_path).unwrap();
    assert_eq!(reopened.step(), WorkflowStep::Process);
    assert_eq!(reopened.render(), &RenderState::Stale);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ownership_mode_is_joint_for_new_projects_and_legacy_when_absent() {
    let dir = temp_dir();
    let source = dir.join("voice.wav");
    fs::write(&source, b"source").unwrap();
    let path = dir.join("episode.cleanslate");
    let project = Project::create(&path, vec![source]).unwrap();
    assert!(project.ownership_settings().enabled);
    assert!(Project::open(&path).unwrap().ownership_settings().enabled);
    let mut json: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("ownership");
    fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
    let mut legacy = Project::open(&path).unwrap();
    assert!(!legacy.ownership_settings().enabled);
    legacy
        .set_ownership_settings(rust_app::OwnershipSettings {
            enabled: true,
            ..Default::default()
        })
        .unwrap();
    assert!(Project::open(&path).unwrap().ownership_settings().enabled);
    fs::remove_dir_all(dir).unwrap();
}

fn calibration_project(dir: &std::path::Path) -> (Project, rust_app::OwnershipSettings) {
    let source = dir.join("voice.wav");
    let writer =
        bwavfile::WaveWriter::create(&source, bwavfile::WaveFmt::new_pcm_mono(48_000, 16)).unwrap();
    let mut frames = writer.audio_frame_writer().unwrap();
    frames.write_frames(&vec![0.0_f32; 48_000]).unwrap();
    frames.end().unwrap();
    let project = Project::create(dir.join("episode.cleanslate"), vec![source.clone()]).unwrap();
    let settings = rust_app::OwnershipSettings {
        enabled: true,
        sources: vec![rust_app::SourceStamp::read(&source).unwrap()],
        examples: vec![rust_app::CalibrationExample {
            kind: rust_app::CalibrationKind::RoomTone,
            track: 0,
            start: 0,
            end: 12_000,
        }],
    };
    (project, settings)
}

#[test]
fn calibration_round_trips_removes_and_rejects_invalid_examples_without_mutation() {
    let dir = temp_dir();
    let (mut project, settings) = calibration_project(&dir);
    project.set_ownership_settings(settings.clone()).unwrap();
    project
        .mark_render_current(dir.join("report.json"))
        .unwrap();
    for (track, start, end) in [
        (1, 0, 12_000),
        (0, 0, 11_999),
        (0, 24_000, 12_000),
        (0, 40_000, 52_000),
    ] {
        let mut invalid = settings.clone();
        invalid.examples[0].track = track;
        invalid.examples[0].start = start;
        invalid.examples[0].end = end;
        assert!(project.set_ownership_settings(invalid).is_err());
        assert_eq!(project.ownership_settings(), &settings);
        assert!(matches!(project.render(), RenderState::Current { .. }));
    }
    let mut invalid = settings.clone();
    invalid.sources.clear();
    assert!(project.set_ownership_settings(invalid).is_err());
    assert_eq!(
        Project::open(project.path()).unwrap().ownership_settings(),
        &settings
    );
    let mut removed = settings.clone();
    removed.examples.clear();
    removed.sources.clear();
    project.set_ownership_settings(removed.clone()).unwrap();
    assert_eq!(project.render(), &RenderState::Stale);
    assert_eq!(
        Project::open(project.path()).unwrap().ownership_settings(),
        &removed
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn source_changes_relink_and_additions_invalidate_examples() {
    let dir = temp_dir();
    let (mut project, settings) = calibration_project(&dir);
    project.set_ownership_settings(settings.clone()).unwrap();
    let source = project.source_paths()[0].clone();
    fs::write(&source, b"changed").unwrap();
    assert!(project.set_ownership_settings(settings).is_err());
    let reopened = Project::open(project.path()).unwrap();
    assert!(reopened.ownership_settings().examples.is_empty());
    assert!(reopened.calibration_notice().is_some());
    assert_eq!(reopened.render(), &RenderState::Stale);

    let (mut project, settings) = calibration_project(&dir);
    project.set_ownership_settings(settings.clone()).unwrap();
    project.relink(0, source.clone()).unwrap();
    assert!(project.ownership_settings().examples.is_empty());
    assert!(project.calibration_notice().is_some());
    project.set_ownership_settings(settings).unwrap();
    project.add_sources(vec![source]).unwrap();
    assert!(project.ownership_settings().examples.is_empty());
    assert!(
        Project::open(project.path())
            .unwrap()
            .ownership_settings()
            .examples
            .is_empty()
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn malformed_saved_calibration_is_cleared_and_failed_save_is_transactional() {
    let dir = temp_dir();
    let (mut project, settings) = calibration_project(&dir);
    project.set_ownership_settings(settings.clone()).unwrap();
    let mut json: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path()).unwrap()).unwrap();
    json["ownership"]["examples"][0]["end"] = 99_000.into();
    fs::write(project.path(), serde_json::to_vec(&json).unwrap()).unwrap();
    let reopened = Project::open(project.path()).unwrap();
    assert!(reopened.ownership_settings().examples.is_empty());
    assert!(reopened.calibration_notice().is_some());
    fs::remove_file(project.path()).unwrap();
    fs::create_dir(project.path()).unwrap();
    assert!(project.set_ownership_settings(Default::default()).is_err());
    assert_eq!(project.ownership_settings(), &settings);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn calibration_is_bound_to_ordered_sources_even_when_mode_is_disabled() {
    let dir = temp_dir();
    let (project, mut settings) = calibration_project(&dir);
    let other = dir.join("guest.wav");
    fs::write(&other, b"other source").unwrap();
    settings
        .sources
        .push(rust_app::SourceStamp::read(&other).unwrap());
    settings.enabled = false;
    assert!(
        settings
            .validate(&settings.sources, &[48_000, 48_000])
            .is_ok()
    );
    let mut reversed = settings.sources.clone();
    reversed.reverse();
    assert!(settings.validate(&reversed, &[48_000, 48_000]).is_err());
    settings.examples[0].kind = rust_app::CalibrationKind::SoloSpeech;
    assert!(
        settings
            .validate(&settings.sources, &[48_000, 8_000])
            .is_err()
    );
    drop(project);
    fs::remove_dir_all(dir).unwrap();
}
