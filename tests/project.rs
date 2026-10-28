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
    static NEXT_PROJECT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "cleanslate-project-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT_PROJECT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
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

#[test]
fn preset_sound_changes_are_atomic_undoable_and_preserve_mute() {
    let dir = temp_dir();
    let mut project = session(&dir);
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![project.sources[0].speaker_id],
            &[],
        )
        .unwrap();
    project
        .protect_range(TICKS_PER_SECOND / 2, TICKS_PER_SECOND * 3 / 4)
        .unwrap();
    let references = project.reference_labels().to_vec();
    let constraints = project.gain_constraints();
    project
        .set_parameters(AutomixParameters {
            mute: true,
            ..project.parameters()
        })
        .unwrap();
    let before = project.parameters();
    let mut sound = rust_app::SoundSettings::from(before);
    sound.attenuation_db = -12.0;
    sound.open_ms = 25.0;
    assert!(project.apply_sound(sound).unwrap());
    assert!(project.parameters().mute);
    assert_eq!(project.reference_labels(), references);
    assert_eq!(project.gain_constraints(), constraints);
    assert_eq!(project.parameters().attenuation_db, -12.0);
    assert!(!project.apply_sound(sound).unwrap());
    let mut invalid = sound;
    invalid.close_ms = f32::NAN;
    assert!(project.apply_sound(invalid).is_err());
    assert_eq!(project.parameters().close_ms, before.close_ms);
    project.undo_sound().unwrap();
    assert_eq!(project.parameters(), before);
    project.redo_sound().unwrap();
    assert_eq!(
        Project::open(project.path()).unwrap().parameters(),
        project.parameters()
    );
    project
        .set_parameters(AutomixParameters {
            mute: false,
            ..project.parameters()
        })
        .unwrap();
    let mut unmuted = rust_app::SoundSettings::from(project.parameters());
    unmuted.attenuation_db = -24.0;
    assert!(project.apply_sound(unmuted).unwrap());
    assert!(!project.parameters().mute);
    project.undo_sound().unwrap();
    assert!(!project.parameters().mute);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn named_save_is_independent_of_recovery_and_failed_save_as_keeps_destination() {
    let dir = temp_dir();
    let mut project = session(&dir);
    assert!(project.save_destination().is_none());
    let recovery = project.path().to_owned();
    let named = dir.join("named.cleanslate");
    project.save_as(&named).unwrap();
    let named = fs::canonicalize(named).unwrap();
    let saved = fs::read(&named).unwrap();
    assert_eq!(project.path(), recovery);
    project
        .set_parameters(AutomixParameters {
            attenuation_db: -12.0,
            ..project.parameters()
        })
        .unwrap();
    assert_eq!(fs::read(&named).unwrap(), saved);
    assert!(project.has_unsaved_changes());
    assert!(
        project
            .save_as(&dir.join("missing/copy.cleanslate"))
            .is_err()
    );
    assert_eq!(project.save_destination(), Some(named.as_path()));
    project.save_named().unwrap();
    assert!(!project.has_unsaved_changes());
    let reopened = Project::open(&named).unwrap();
    assert_eq!(reopened.save_destination(), Some(named.as_path()));
    assert_eq!(reopened.parameters().attenuation_db, -12.0);
    assert_eq!(reopened.sources[0].path, project.sources[0].path);
    assert_eq!(
        Project::open(&recovery).unwrap().save_destination(),
        Some(fs::canonicalize(&recovery).unwrap().as_path())
    );
    assert!(project.save_as(&dir.join("voice.wav")).is_err());
    let saved = fs::read(&named).unwrap();
    assert!(project.save_as(&dir.join("named")).is_err());
    assert_eq!(fs::read(&named).unwrap(), saved);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(windows)]
fn named_save_reports_success_with_warning_if_only_recovery_association_fails() {
    let dir = temp_dir();
    let mut project = session(&dir);
    let recovery = project.path().to_owned();
    let original_permissions = fs::metadata(&recovery).unwrap().permissions();
    let mut readonly = original_permissions.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&recovery, readonly).unwrap();
    let destination = dir.join("named.cleanslate");
    project.save_as(&destination).unwrap();
    assert!(project.save_warning().is_some());
    assert_eq!(
        project.save_destination(),
        Some(fs::canonicalize(&destination).unwrap().as_path())
    );
    assert!(!project.has_unsaved_changes());
    assert_eq!(
        Project::open(&destination).unwrap().parameters(),
        project.parameters()
    );
    fs::set_permissions(recovery, original_permissions).unwrap();
    project.save_named().unwrap();
    assert!(project.save_warning().is_none());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn sound_values_validate_persist_and_legacy_settings_keep_finite_attenuation() {
    let dir = temp_dir();
    let mut project = session(&dir);
    for db in [-120.0, 0.0, -12.125] {
        for duration in [0.0, 60000.0, 12.125] {
            project
                .set_parameters(AutomixParameters {
                    attenuation_db: db,
                    open_ms: duration,
                    close_ms: duration,
                    minimum_gap_ms: duration,
                    ..project.parameters()
                })
                .unwrap();
            assert_eq!(
                Project::open(project.path()).unwrap().parameters(),
                project.parameters()
            );
        }
    }
    for invalid in [f32::NAN, f32::INFINITY, -120.01, 0.01] {
        assert!(
            project
                .set_parameters(AutomixParameters {
                    attenuation_db: invalid,
                    ..project.parameters()
                })
                .is_err()
        );
    }
    for invalid in [f32::NAN, f32::INFINITY, -0.01, 60000.1] {
        for field in 0..3 {
            let mut p = project.parameters();
            match field {
                0 => p.open_ms = invalid,
                1 => p.close_ms = invalid,
                _ => p.minimum_gap_ms = invalid,
            }
            assert!(project.set_parameters(p).is_err());
        }
    }
    let finite = project.parameters();
    assert!(finite.attenuation_gain() > 0.0);
    project
        .set_parameters(AutomixParameters {
            mute: true,
            ..finite
        })
        .unwrap();
    assert_eq!(
        Project::open(project.path())
            .unwrap()
            .parameters()
            .attenuation_gain(),
        0.0
    );
    let mut saved: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path()).unwrap()).unwrap();
    saved["parameters"].as_object_mut().unwrap().remove("mute");
    fs::write(project.path(), serde_json::to_vec(&saved).unwrap()).unwrap();
    assert_eq!(Project::open(project.path()).unwrap().parameters(), finite);
    saved["parameters"]["mute"] = serde_json::json!("yes");
    fs::write(project.path(), serde_json::to_vec(&saved).unwrap()).unwrap();
    assert!(Project::open(project.path()).is_err());
    saved["parameters"]["mute"] = serde_json::json!(false);
    saved["parameters"]["open_ms"] = serde_json::json!(60001);
    fs::write(project.path(), serde_json::to_vec(&saved).unwrap()).unwrap();
    assert!(Project::open(project.path()).is_err());
    fs::remove_dir_all(dir).unwrap();
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
fn retired_native_project_preserves_human_decisions_and_requires_reanalysis() {
    let dir = temp_dir();
    let mut project = session(&dir);
    project
        .replace_reference_labels(vec![label(&project)])
        .unwrap();
    project
        .protect_region(&[0], TICKS_PER_SECOND / 2, TICKS_PER_SECOND)
        .unwrap();
    let draft = project
        .region_gain_edit(
            project.sources[1].track_id,
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            rust_app::GainRegionAction::Attenuate(0.5),
        )
        .unwrap();
    project.commit_gain_edit(draft).unwrap();
    project
        .set_result(
            rust_app::ResultState::Current {
                identity: "legacy-analysis".into(),
            },
            None,
        )
        .unwrap();
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path()).unwrap()).unwrap();
    legacy["nemotron_enabled"] = serde_json::json!(true);
    fs::write(project.path(), serde_json::to_vec(&legacy).unwrap()).unwrap();
    let reopened = Project::open(project.path()).unwrap();
    assert_eq!(reopened.reference_labels(), project.reference_labels());
    assert_eq!(reopened.manual_gain_edits(), project.manual_gain_edits());
    assert_eq!(
        reopened.recorded_protections(),
        project.recorded_protections()
    );
    assert_eq!(reopened.sources.len(), project.sources.len());
    assert!(reopened.calibration_notice().unwrap().contains("removed"));
    assert_eq!(
        reopened.result(),
        &rust_app::ResultState::Stale {
            identity: "legacy-analysis".into()
        }
    );
    reopened.save().unwrap();
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path()).unwrap()).unwrap();
    assert!(saved.get("nemotron_enabled").is_none());
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
