use rust_app::{
    AutomixParameters, EnvelopePoint, ManualGainEdit, Project, ReferenceLabel, RenderState,
    ReviewState, SourceBinding, TICKS_PER_SECOND, TimeRange, WorkflowStep,
};
use std::{
    fs,
    path::{Path, PathBuf},
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
fn wav(path: &Path, rate: u32, value: f32) {
    let writer =
        bwavfile::WaveWriter::create(path, bwavfile::WaveFmt::new_pcm_mono(rate, 16)).unwrap();
    let mut frames = writer.audio_frame_writer().unwrap();
    frames.write_frames(&vec![value; rate as usize]).unwrap();
    frames.end().unwrap();
}
fn session(dir: &Path) -> Project {
    wav(&dir.join("voice.wav"), 44_100, 0.1);
    wav(&dir.join("guest.wav"), 96_000, 0.2);
    Project::create(
        dir.join("episode.cleanslate"),
        vec![dir.join("voice.wav"), dir.join("guest.wav")],
    )
    .unwrap()
}
fn bindings(project: &Project) -> Vec<SourceBinding> {
    project
        .sources
        .iter()
        .map(|s| SourceBinding {
            track: s.track_id,
            fingerprint: s.fingerprint.clone(),
        })
        .collect()
}
fn label(project: &Project) -> ReferenceLabel {
    ReferenceLabel {
        id: 1,
        range: TimeRange {
            start: 0,
            end: TICKS_PER_SECOND / 2,
        },
        active_speakers: vec![project.sources[0].speaker_id],
        sources: bindings(project),
        reference_track: None,
    }
}

#[test]
fn saved_project_retains_confirmed_decisions_without_derived_caches() {
    let dir = temp_dir();
    let mut project = session(&dir);
    project
        .replace_reference_labels(vec![label(&project)])
        .unwrap();
    project
        .protect_region(&[0], TICKS_PER_SECOND / 2, 3 * TICKS_PER_SECOND / 4)
        .unwrap();
    let edit = ManualGainEdit {
        interpolation: rust_app::ManualInterpolation::Linear,
        id: 1,
        source: bindings(&project)[1].clone(),
        points: vec![
            EnvelopePoint { time: 0, gain: 0.2 },
            EnvelopePoint {
                time: TICKS_PER_SECOND / 2,
                gain: 0.4,
            },
        ],
    };
    project
        .replace_manual_gain_edits(vec![edit.clone()])
        .unwrap();
    project
        .set_review_state("uncertain:one", ReviewState::Reviewed)
        .unwrap();
    let mut parameters = project.parameters();
    parameters.attenuation_db = -12.0;
    project.set_parameters(parameters).unwrap();
    let reopened = Project::open(project.path()).unwrap();
    assert_eq!(reopened.reference_labels(), project.reference_labels());
    assert_eq!(reopened.manual_gain_edits(), &[edit]);
    assert_eq!(
        reopened.recorded_protections(),
        project.recorded_protections()
    );
    assert_eq!(
        reopened.review_states()["uncertain:one"],
        ReviewState::Reviewed
    );
    assert_eq!(reopened.parameters(), parameters);
    assert_eq!(reopened.sources[0].metadata.grid.sample_rate, 44_100);
    assert_eq!(reopened.sources[1].metadata.grid.sample_rate, 96_000);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn reordering_and_identical_relink_preserve_track_and_speaker_bindings() {
    let dir = temp_dir();
    let mut project = session(&dir);
    let original = project.sources[0].clone();
    project
        .replace_reference_labels(vec![label(&project)])
        .unwrap();
    project
        .protect_region(&[0], TICKS_PER_SECOND / 2, TICKS_PER_SECOND)
        .unwrap();
    let moved = dir.join("moved.wav");
    fs::rename(&original.path, &moved).unwrap();
    let mut project = Project::open(project.path()).unwrap();
    assert!(!project.sources[0].available);
    project.relink(0, moved).unwrap();
    assert!(project.pending_source_reviews().is_empty());
    assert_eq!(project.ownership_settings().examples.len(), 1);
    let other = project.sources[1].track_id;
    project.reorder_tracks(&[other, original.track_id]).unwrap();
    let reopened = Project::open(project.path()).unwrap();
    assert_eq!(reopened.sources[1].speaker_id, original.speaker_id);
    assert_eq!(reopened.protections()[0].track, Some(1));
    assert_eq!(reopened.ownership_settings().examples[0].track, 1);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn changed_audio_keeps_decisions_dormant_until_explicit_rebind() {
    let dir = temp_dir();
    let mut project = session(&dir);
    let label = label(&project);
    project
        .replace_reference_labels(vec![label.clone()])
        .unwrap();
    project.protect_track(0).unwrap();
    let source = project.sources[0].clone();
    wav(&source.path, 44_100, 0.3);
    let mut project = Project::open(project.path()).unwrap();
    assert_eq!(project.reference_labels(), std::slice::from_ref(&label));
    assert!(project.ownership_settings().examples.is_empty());
    assert!(project.protections().is_empty());
    assert_eq!(project.step(), WorkflowStep::Import);
    project.relink(0, source.path).unwrap();
    assert_eq!(project.pending_source_reviews(), &[source.track_id]);
    assert_eq!(project.reference_labels(), &[label]);
    assert!(project.protections().is_empty());
    project.review_and_rebind_source(source.track_id).unwrap();
    assert_eq!(project.protections().len(), 1);
    assert_eq!(project.ownership_settings().examples.len(), 1);
    assert_eq!(
        Project::open(project.path())
            .unwrap()
            .recorded_protections(),
        project.recorded_protections()
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_source_changed_while_open_cannot_receive_old_protections_or_new_edits() {
    let dir = temp_dir();
    let mut project = session(&dir);
    project.protect_track(0).unwrap();
    wav(&project.sources[0].path, 44_100, 0.8);
    assert!(project.protections().is_empty());
    assert!(project.protect_range(0, TICKS_PER_SECOND / 2).is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn invalid_decisions_and_parameters_do_not_mutate_saved_project() {
    let dir = temp_dir();
    let mut project = session(&dir);
    let good = label(&project);
    project
        .replace_reference_labels(vec![good.clone()])
        .unwrap();
    let before = fs::read(project.path()).unwrap();
    let mut contrary = good.clone();
    contrary.id = 2;
    contrary.active_speakers = vec![project.sources[1].speaker_id];
    assert!(
        project
            .replace_reference_labels(vec![good, contrary])
            .is_err()
    );
    assert!(project.protect_range(0, 2 * TICKS_PER_SECOND).is_err());
    assert!(
        project
            .set_parameters(AutomixParameters {
                attenuation_db: f32::NAN,
                ..Default::default()
            })
            .is_err()
    );
    assert_eq!(fs::read(project.path()).unwrap(), before);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn malformed_versions_ids_bounds_and_nonfinite_gain_are_rejected() {
    let dir = temp_dir();
    let project = session(&dir);
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path()).unwrap()).unwrap();
    for pointer in [
        "/schema_version",
        "/ticks_per_second",
        "/sources/0/track_id",
        "/sources/0/metadata/grid/sample_rate",
    ] {
        let mut json = original.clone();
        *json.pointer_mut(pointer).unwrap() = 0.into();
        fs::write(project.path(), serde_json::to_vec(&json).unwrap()).unwrap();
        assert!(Project::open(project.path()).is_err(), "{pointer}");
    }
    let mut json = original.clone();
    json["sources"][1]["speaker_id"] = json["sources"][0]["speaker_id"].clone();
    fs::write(project.path(), serde_json::to_vec(&json).unwrap()).unwrap();
    assert!(Project::open(project.path()).is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(windows)]
#[test]
fn failed_atomic_replace_retains_prior_data_and_memory() {
    use std::os::windows::fs::OpenOptionsExt;
    let dir = temp_dir();
    let mut project = session(&dir);
    let before = fs::read(project.path()).unwrap();
    let parameters = project.parameters();
    let lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(project.path())
        .unwrap();
    assert!(
        project
            .set_parameters(AutomixParameters {
                attenuation_db: -10.0,
                ..parameters
            })
            .is_err()
    );
    assert_eq!(project.parameters(), parameters);
    assert_eq!(fs::read(project.path()).unwrap(), before);
    drop(lock);
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 3);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_render_marks_only_deliverable_stale_and_project_stays_editable() {
    let dir = temp_dir();
    let mut project = session(&dir);
    project
        .mark_render_current(dir.join("missing/report.json"))
        .unwrap();
    let reopened = Project::open(project.path()).unwrap();
    assert_eq!(reopened.render(), &RenderState::Stale);
    assert_eq!(reopened.step(), WorkflowStep::Process);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn manual_curve_validation_rejects_unordered_boosts_and_constraint_conflicts() {
    let dir = temp_dir();
    let mut project = session(&dir);
    let binding = bindings(&project)[0].clone();
    for points in [
        vec![
            EnvelopePoint {
                time: TICKS_PER_SECOND / 2,
                gain: 0.5,
            },
            EnvelopePoint { time: 0, gain: 0.2 },
        ],
        vec![
            EnvelopePoint {
                time: 0,
                gain: f32::INFINITY,
            },
            EnvelopePoint {
                time: TICKS_PER_SECOND / 2,
                gain: 1.0,
            },
        ],
        vec![
            EnvelopePoint { time: 0, gain: 1.1 },
            EnvelopePoint {
                time: TICKS_PER_SECOND / 2,
                gain: 1.0,
            },
        ],
    ] {
        assert!(
            project
                .replace_manual_gain_edits(vec![ManualGainEdit {
                    interpolation: rust_app::ManualInterpolation::Linear,
                    id: 1,
                    source: binding.clone(),
                    points
                }])
                .is_err()
        );
    }
    project.protect_track(0).unwrap();
    assert!(
        project
            .replace_manual_gain_edits(vec![ManualGainEdit {
                interpolation: rust_app::ManualInterpolation::Linear,
                id: 1,
                source: binding,
                points: vec![
                    EnvelopePoint { time: 0, gain: 0.5 },
                    EnvelopePoint {
                        time: TICKS_PER_SECOND / 2,
                        gain: 0.5
                    }
                ]
            }])
            .is_err()
    );
    assert_eq!(project.recorded_protections().len(), 1);
    fs::remove_dir_all(dir).unwrap();
}
