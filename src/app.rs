use crate::{
    AnalysisPreview, AutomixParameters, CalibrationExample, CalibrationKind, Comparison,
    OwnershipSettings, PreviewCache, Project, RenderState, ReviewState, ReviewWorkspace,
    RunRequest, WorkflowStep,
    audition::{AuditionMode, AuditionPreparation, LiveAudition},
    export,
    prepared_analysis::source_stamps,
    preview_stage, run_with_ownership,
    timeline::{TimelineEvent, TimelineViewport},
    validate, validate_export,
    waveform::{self, TimelineCanvas, WaveformCache},
};
use iced::{
    Element, Fill, Length, Subscription, Task, Theme, keyboard,
    widget::{
        Space, button, canvas, column, container, progress_bar, responsive, row, scrollable,
        slider, text, text_input,
    },
    window,
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

// Match the detector's rounding of 1536-sample frames at 48 kHz.
fn effective_padding_ms(value: f32) -> f32 {
    (value / 32.0).round() * 32.0
}

fn dialog_start_directory() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|path| path.join("Documents"))
        .filter(|path| path.is_dir())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
        })
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn available_project_path(directory: &std::path::Path) -> PathBuf {
    let mut path = directory.join("CleanSlate.cleanslate");
    for suffix in 2.. {
        if !path.exists() {
            return path;
        }
        path = directory.join(format!("CleanSlate-{suffix}.cleanslate"));
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn completed_render_cannot_replace_changed_session() {
        for change in 0..8 {
            let directory = std::env::temp_dir().join(format!(
                "cleanslate-render-snapshot-{}-{change}",
                std::process::id()
            ));
            std::fs::create_dir_all(&directory).unwrap();
            let source = directory.join("voice.wav");
            write_wav(&source, &vec![0.0; 48_000]);
            let project =
                crate::Project::create(directory.join("episode.cleanslate"), vec![source.clone()])
                    .unwrap();
            let snapshot = super::RenderSnapshot::read(&project).unwrap();
            assert!(super::RenderSnapshot::read(&project).unwrap() == snapshot);
            let (sender, result) = super::mpsc::channel();
            let mut app = super::CleanSlateApp {
                project: Some(project),
                processing: Some(super::Processing {
                    cancelled: super::Arc::new(super::AtomicBool::new(false)),
                    progress: super::Arc::new(super::AtomicU8::new(100)),
                    result,
                    snapshot,
                }),
                ..Default::default()
            };
            match change {
                0 => {
                    let project = app.project.as_mut().unwrap();
                    let mut parameters = project.parameters();
                    parameters.attenuation_db = -12.0;
                    project.set_parameters(parameters).unwrap();
                }
                1 => app.project.as_mut().unwrap().protect_range(0, 100).unwrap(),
                2 => write_wav(&source, &[0.0; 3073]),
                3 => app.project = None,
                4 => {
                    let mut other = crate::Project::create(
                        directory.join("other.cleanslate"),
                        vec![source.clone()],
                    )
                    .unwrap();
                    other
                        .mark_render_current(directory.join("other-report.json"))
                        .unwrap();
                    app.project = Some(other);
                }
                7 => {
                    let _ = app.update(super::Message::Attenuation(-12.0));
                }
                _ => {
                    let project = app.project.as_mut().unwrap();
                    let mut settings = project.ownership_settings().clone();
                    if change == 5 {
                        settings.enabled = false;
                    } else {
                        settings.sources = super::source_stamps(&project.source_paths()).unwrap();
                        settings.examples.push(crate::CalibrationExample {
                            kind: crate::CalibrationKind::RoomTone,
                            track: 0,
                            start: 0,
                            end: 24_000,
                        });
                    }
                    project.set_ownership_settings(settings).unwrap();
                }
            }
            sender
                .send(Ok(directory.join("old-report.json").display().to_string()))
                .unwrap();
            app.poll_processing();
            assert!(app.processing.is_none());
            assert!(app.messages[0].contains("earlier settings or sources"));
            assert!(!app.export_ready());
            if let Some(project) = app.project {
                if change == 4 {
                    assert_eq!(
                        project.render(),
                        &crate::RenderState::Current {
                            identity: directory.join("other-report.json")
                        }
                    );
                } else {
                    assert_eq!(project.render(), &crate::RenderState::Stale);
                    assert!(project.review_identity().is_none());
                }
            }
            std::fs::remove_dir_all(directory).unwrap();
        }
    }

    #[test]
    fn calibration_time_controls_reject_invalid_input_and_capture_playhead() {
        let directory =
            std::env::temp_dir().join(format!("calibration-times-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("voice.wav");
        write_wav(&source, &vec![0.0; 48_000]);
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("episode.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        app.timeline.set_total_samples(48_000);
        for input in ["NaN", "inf", "-0.01", "1e99", "", "not a time"] {
            let _ = app.update(super::Message::CalibrationStart(input.into()));
            let _ = app.update(super::Message::CalibrationEnd("1".into()));
            let _ = app.update(super::Message::SaveCalibration(
                crate::CalibrationKind::RoomTone,
                0,
            ));
            assert!(app.calibration_error.is_some(), "{input}");
            assert!(
                app.project
                    .as_ref()
                    .unwrap()
                    .ownership_settings()
                    .examples
                    .is_empty()
            );
            assert!(app.preview_processing.is_none());
        }
        app.timeline.seek(17);
        let _ = app.update(super::Message::CalibrationStartAtPlayhead);
        app.timeline.seek(24_017);
        let _ = app.update(super::Message::CalibrationEndAtPlayhead);
        let start = app.calibration_start.clone();
        let end = app.calibration_end.clone();
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Select(100, 30_000),
        ));
        assert_eq!(app.calibration_start, start);
        assert_eq!(app.calibration_end, end);
        let _ = app.update(super::Message::CalibrationFromSelection);
        assert_eq!(app.calibration_start, "0.00208333");
        assert_eq!(app.calibration_end, "0.62500000");
        assert!(app.project.as_ref().unwrap().protections().is_empty());
        let source = app.project.as_ref().unwrap().source_paths()[0].clone();
        write_wav(&source, &vec![0.0; 96_000]);
        let _ = app.update(super::Message::SaveCalibration(
            crate::CalibrationKind::RoomTone,
            0,
        ));
        assert!(
            app.calibration_error
                .as_deref()
                .unwrap()
                .contains("changed")
        );
        assert!(
            app.project
                .as_ref()
                .unwrap()
                .ownership_settings()
                .examples
                .is_empty()
        );
        assert!(app.preview_processing.is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn calibration_drag_saves_exact_examples_without_protection_and_refreshes_preview() {
        let directory =
            std::env::temp_dir().join(format!("calibration-editor-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("voice.wav");
        write_wav(&source, &vec![0.0; 48_000]);
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("episode.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        app.timeline.set_total_samples(48_000);
        let _ = app.update(super::Message::CalibrationMode(true));
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Select(24_001, 1),
        ));
        let _ = app.update(super::Message::SaveCalibration(
            crate::CalibrationKind::RoomTone,
            0,
        ));
        assert!(
            app.calibration_error.is_none(),
            "{:?}",
            app.calibration_error
        );
        assert_eq!(
            app.project.as_ref().unwrap().ownership_settings().examples,
            vec![crate::CalibrationExample {
                kind: crate::CalibrationKind::RoomTone,
                track: 0,
                start: 1,
                end: 24_001,
            }]
        );
        assert!(app.project.as_ref().unwrap().protections().is_empty());
        assert!(app.preview_processing.is_some() || app.preview_pending.is_some());
        let _ = app.update(super::Message::SaveCalibration(
            crate::CalibrationKind::SoloSpeech,
            0,
        ));
        assert_eq!(
            app.project
                .as_ref()
                .unwrap()
                .ownership_settings()
                .examples
                .len(),
            2
        );
        let _ = app.update(super::Message::RemoveCalibration(0));
        assert_eq!(
            app.project.as_ref().unwrap().ownership_settings().examples[0].kind,
            crate::CalibrationKind::SoloSpeech
        );
        let _ = app.update(super::Message::OwnershipMode(false));
        assert!(
            !crate::Project::open(app.project.as_ref().unwrap().path())
                .unwrap()
                .ownership_settings()
                .enabled
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while app.preview_processing.is_some() || app.preview_pending.is_some() {
            app.poll_processing();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn failed_derivation_keeps_completed_preview_source_identity() {
        let directory =
            std::env::temp_dir().join(format!("preview-identity-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("voice.wav");
        write_wav(&path, &[0.0; 3073]);
        let mut project =
            crate::Project::create(directory.join("episode.cleanslate"), vec![path.clone()])
                .unwrap();
        project.protect_range(1600, 3000).unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        let drain = |app: &mut super::CleanSlateApp| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while app.preview_processing.is_some() || app.preview_pending.is_some() {
                app.poll_processing();
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        };
        app.start_preview();
        drain(&mut app);
        let original = vec![super::waveform::SourceStamp::read(&path).unwrap()];
        assert_eq!(app.preview_sources, original);
        write_wav(&path, &[0.0; 1537]);
        app.start_preview();
        drain(&mut app);
        assert!(app.preview_error.as_deref().unwrap().contains("protection"));
        assert_eq!(app.preview_cache.stats().preparations, 2);
        assert_eq!(
            app.preview.as_ref().unwrap().envelopes[0].sample_count(),
            3073
        );
        assert_eq!(app.preview_sources, original);
        assert_ne!(
            app.preview_sources,
            vec![super::waveform::SourceStamp::read(&path).unwrap()]
        );
        app.timeline.set_total_samples(3073);
        app.audition(false);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while app.audition_preparation.is_busy() {
            app.poll_audition();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(app.audition.is_none());
        assert!(
            app.messages
                .iter()
                .any(|message| message.contains("Sources changed"))
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn dragging_coalesces_and_finishes_with_latest_parameters() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-drag-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("voice.wav");
        write_wav(&path, &[0.0; 1537]);
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("episode.cleanslate"), vec![path]).unwrap(),
            ),
            ..Default::default()
        };
        app.start_preview();
        let active = app.preview_processing.as_ref().unwrap().cancelled.clone();
        for value in [-6.0, -12.0, -17.0] {
            let _ = app.update(super::Message::Attenuation(value));
        }
        assert!(std::sync::Arc::ptr_eq(
            &active,
            &app.preview_processing.as_ref().unwrap().cancelled
        ));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            app.poll_processing();
            if app
                .preview
                .as_ref()
                .is_some_and(|p| p.parameters.attenuation_db == -17.0)
            {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "latest drag value never installed"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(app.preview_cache.stats().preparations, 1);
        // Control worker delivery and the clock boundary: two drag inputs occur
        // before completion (e.g. 0/16 ms input, 30 ms result, 40 ms boundary).
        // Repeating this proves intermediate snapshots survive sustained input.
        for snapshot in [-11.0, -10.0, -9.0] {
            app.preview_parameters.attenuation_db = snapshot;
            app.start_preview();
            let processing = app.preview_processing.as_mut().unwrap();
            let completed = processing
                .result
                .recv_timeout(std::time::Duration::from_secs(20))
                .unwrap();
            let (sender, receiver) = super::mpsc::channel();
            processing.result = receiver;
            for value in [snapshot - 1.0, -17.0] {
                let _ = app.update(super::Message::Attenuation(value));
                // Freeze the coalescing deadline ahead of the simulated inputs.
                app.preview_pending =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
                app.poll_processing();
                assert!(
                    !app.preview_processing
                        .as_ref()
                        .unwrap()
                        .cancelled
                        .load(super::Ordering::Relaxed)
                );
            }
            sender.send(completed).unwrap();
            app.poll_processing();
            assert_eq!(
                app.preview.as_ref().unwrap().parameters.attenuation_db,
                snapshot
            );
        }
        let _ = app.update(super::Message::RefreshPreview);
        assert_eq!(
            app.project.as_ref().unwrap().parameters().attenuation_db,
            -17.0
        );
        let _ = app.update(super::Message::ProtectTrack(0));
        while app.preview_processing.is_some() || app.preview_pending.is_some() {
            app.poll_processing();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(
            app.preview.as_ref().unwrap().protections,
            app.project.as_ref().unwrap().protections()
        );
        assert_eq!(app.preview_cache.stats().preparations, 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn completed_obsolete_work_cannot_replace_preview_after_source_change() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-obsolete-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("voice.wav");
        write_wav(&path, &[0.0; 1537]);
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("episode.cleanslate"), vec![path.clone()])
                    .unwrap(),
            ),
            ..Default::default()
        };
        app.start_preview();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while app
            .preview_processing
            .as_ref()
            .unwrap()
            .progress
            .load(super::Ordering::Relaxed)
            != 100
        {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        write_wav(&path, &[0.0; 3073]);
        app.poll_processing();
        assert!(
            app.preview.is_none(),
            "changed source accepted a completed obsolete preview"
        );
        app.install_waveforms(vec![]);
        app.start_preview();
        while app.preview_processing.is_some() || app.preview_pending.is_some() {
            app.poll_processing();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(
            app.preview.as_ref().unwrap().envelopes[0].sample_count(),
            3073
        );
        app.start_preview();
        let obsolete = app.preview_processing.as_ref().unwrap().cancelled.clone();
        app.install_waveforms(vec![]);
        app.start_preview();
        assert!(obsolete.load(super::Ordering::Relaxed));
        assert!(std::sync::Arc::ptr_eq(
            &obsolete,
            &app.preview_processing.as_ref().unwrap().cancelled
        ));
        while app.preview_processing.is_some() || app.preview_pending.is_some() {
            app.poll_processing();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(app.preview_error.is_none());
        std::fs::write(&path, b"invalid WAV").unwrap();
        app.start_preview();
        while app.preview_processing.is_some() {
            app.poll_processing();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(app.preview_error.is_some());
        assert_eq!(
            app.preview.as_ref().unwrap().envelopes[0].sample_count(),
            3073
        );
        std::fs::remove_file(&path).unwrap();
        app.start_preview();
        assert!(app.preview_error.is_some());
        assert_eq!(
            app.preview.as_ref().unwrap().envelopes[0].sample_count(),
            3073
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn audition_stop_relink_and_error_do_not_start_output() {
        let directory =
            std::env::temp_dir().join(format!("audition-editor-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("voice.wav");
        write_wav(&path, &[0.0; 1537]);
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("episode.cleanslate"), vec![path.clone()])
                    .unwrap(),
            ),
            ..Default::default()
        };
        let preview = crate::AnalysisPreview {
            stems: vec![path.clone()],
            regions: vec![],
            envelopes: vec![crate::GainEnvelope::constant(1537, 1.0)],
            protections: vec![],
            parameters: Default::default(),
        };
        app.preview = Some(preview.clone());
        app.timeline.set_total_samples(1537);
        app.timeline.set_selection(Some((17, 600)));
        let drain = |app: &mut super::CleanSlateApp| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while app.audition_preparation.is_busy() {
                app.poll_audition();
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
            assert!(app.audition.is_none());
        };
        app.audition(true);
        assert_eq!(app.audition_range, Some((17, 600)));
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Seek(40),
        ));
        assert_eq!(app.audition_range, Some((40, 600)));
        let _ = app.update(super::Message::Stop);
        drain(&mut app);
        assert!(app.audition_range.is_none());
        app.audition(false);
        app.install_waveforms(vec![]);
        drain(&mut app);
        assert!(app.audition_range.is_none());
        app.preview = Some(preview);
        std::fs::remove_file(&path).unwrap();
        app.audition(false);
        drain(&mut app);
        assert!(!app.messages.is_empty());
        assert!(app.audition_range.is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn padding_control_shows_effective_steps_and_keeps_legacy_value_until_edited() {
        let mut app = super::CleanSlateApp::default();
        app.preview_parameters.speech_padding_ms = 300.0;
        assert_eq!(super::effective_padding_ms(300.0), 288.0);
        assert_eq!(app.preview_parameters.speech_padding_ms, 300.0);
        for value in [0.0, 32.0, 320.0, 992.0] {
            let _ = app.update(super::Message::SpeechPadding(value));
            assert_eq!(app.preview_parameters.speech_padding_ms, value);
            let saved = serde_json::to_string(&app.preview_parameters).unwrap();
            assert_eq!(
                serde_json::from_str::<crate::AutomixParameters>(&saved).unwrap(),
                app.preview_parameters
            );
        }
        let _ = app.update(super::Message::SpeechPadding(300.0));
        assert_eq!(app.preview_parameters.speech_padding_ms, 288.0);
    }

    #[test]
    fn sensitivity_control_inverts_threshold_without_changing_legacy_storage() {
        let legacy: crate::AutomixParameters =
            serde_json::from_str(r#"{"speech_sensitivity":0.6,"speech_padding_ms":300.0}"#)
                .unwrap();
        let mut app = super::CleanSlateApp {
            preview_parameters: legacy,
            ..Default::default()
        };
        let saved = serde_json::to_string(&app.preview_parameters).unwrap();
        assert_eq!(
            serde_json::from_str::<crate::AutomixParameters>(&saved).unwrap(),
            legacy
        );
        let _ = app.update(super::Message::SpeechSensitivity(0.8));
        assert!((app.preview_parameters.speech_sensitivity - 0.2).abs() < 1e-6);
        assert!(app.preview_parameters.speech_sensitivity < legacy.speech_sensitivity);
        assert_eq!(app.preview_parameters.speech_padding_ms, 300.0);
        let saved = serde_json::to_string(&app.preview_parameters).unwrap();
        let reopened: crate::AutomixParameters = serde_json::from_str(&saved).unwrap();
        assert!((reopened.speech_sensitivity - 0.2).abs() < 1e-6);
    }

    #[test]
    fn editor_retains_preparation_for_parameter_edits_and_rechecks_same_path() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-prepared-editor-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("voice.wav");
        write_wav(&path, &vec![0.0; 48_000]);
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("episode.cleanslate"), vec![path.clone()])
                    .unwrap(),
            ),
            ..Default::default()
        };
        app.timeline.set_total_samples(48_000);
        let wait = |app: &mut super::CleanSlateApp| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while app.preview_processing.is_some() {
                app.poll_processing();
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            assert!(app.preview_error.is_none(), "{:?}", app.preview_error);
        };
        app.start_preview();
        wait(&mut app);
        app.preview_parameters.attenuation_db = -9.0;
        app.start_preview();
        wait(&mut app);
        assert_eq!(app.preview_cache.stats().preparations, 1);
        assert_eq!(
            app.preview.as_ref().unwrap().parameters.attenuation_db,
            -9.0
        );
        for enabled in [false, true] {
            let before = app.preview_cache.stats().ownership_derivations;
            let _ = app.update(super::Message::OwnershipMode(enabled));
            wait(&mut app);
            assert_eq!(app.preview_cache.stats().ownership_derivations, before + 1);
        }
        app.calibration_start = "0".into();
        app.calibration_end = "0.5".into();
        let before = app.preview_cache.stats().ownership_derivations;
        let _ = app.update(super::Message::SaveCalibration(
            crate::CalibrationKind::RoomTone,
            0,
        ));
        wait(&mut app);
        assert_eq!(app.preview_cache.stats().ownership_derivations, before + 1);
        let _ = app.update(super::Message::RemoveCalibration(0));
        wait(&mut app);
        assert_eq!(app.preview_cache.stats().ownership_derivations, before + 2);
        assert_eq!(app.preview_cache.stats().preparations, 1);
        write_wav(&path, &[0.0; 3073]);
        app.start_preview();
        wait(&mut app);
        assert_eq!(app.preview_cache.stats().preparations, 2);
        assert_eq!(
            app.preview.as_ref().unwrap().envelopes[0].sample_count(),
            3073
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn write_wav(path: &std::path::Path, samples: &[f32]) {
        let writer =
            bwavfile::WaveWriter::create(path, bwavfile::WaveFmt::new_pcm_mono(48_000, 16))
                .unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(samples).unwrap();
        frames.end().unwrap();
    }

    #[test]
    fn drop_defers_source_loading_until_background_work() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-deferred-import-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("track.wav");
        write_wav(&source, &[0.0; 16]);
        let mut app = super::CleanSlateApp::default();
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (_sender, result) = super::mpsc::channel();
        app.preview_processing = Some(super::PreviewProcessing {
            progress: Default::default(),
            cancelled: cancelled.clone(),
            generation: 0,
            source_generation: 0,
            sources: vec![],
            result,
        });
        let _ = app.update(super::Message::FileDropped(source));
        assert!(
            cancelled.load(super::Ordering::Relaxed),
            "dropping the old preview job must cancel it"
        );
        assert!(
            app.project.is_none(),
            "drop handling must return before validating and creating the project"
        );
        assert!(
            app.waveforms.is_empty(),
            "drop handling must not decode source samples"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn direct_import_is_deterministic_incremental_and_preserves_valid_session_on_error() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-import-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let a = directory.join("a.wav");
        let b = directory.join("B.wav");
        let invalid = directory.join("short.wav");
        write_wav(&a, &[0.0; 16]);
        write_wav(&b, &[0.0; 16]);
        write_wav(&invalid, &[0.0; 8]);

        let mut app = super::CleanSlateApp::default();
        app.add_stems([b.clone(), a.clone(), b]);
        assert_eq!(app.pending_imports, [a.clone(), directory.join("B.wav")]);
        assert_eq!(app.loading_paths().len(), 2);
        assert!(
            app.source_loading.is_none(),
            "multi-file drops are coalesced before reading"
        );
        finish_import(&mut app);
        assert_eq!(
            app.project.as_ref().unwrap().source_paths(),
            [a, directory.join("B.wav")]
        );
        app.add_stems([invalid]);
        let original_parameters = app.preview_parameters;
        let _ = app.update(super::Message::Attenuation(-3.0));
        assert_eq!(
            app.preview_parameters, original_parameters,
            "edits cannot race the import snapshot"
        );
        finish_import(&mut app);
        assert_eq!(app.project.as_ref().unwrap().sources.len(), 2);
        assert!(app.messages[0].starts_with("Import blocked:"));

        let saved = app.project.as_ref().unwrap().path().to_owned();
        app.begin_source_loading(super::SourceRequest::Open(saved));
        assert!(app.source_loading.is_some());
        finish_import(&mut app);
        assert_eq!(app.waveforms.len(), 2);
        let replacement = directory.join("replacement.wav");
        write_wav(&replacement, &[0.0; 16]);
        app.begin_source_loading(super::SourceRequest::Relink(0, replacement.clone()));
        assert_ne!(app.project.as_ref().unwrap().source_paths()[0], replacement);
        finish_import(&mut app);
        assert_eq!(app.project.as_ref().unwrap().source_paths()[0], replacement);

        std::fs::remove_dir_all(directory).unwrap();
    }

    fn finish_import(app: &mut super::CleanSlateApp) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while app.sources_busy() {
            if app.import_due.is_some() {
                app.import_due = Some(std::time::Instant::now());
            }
            app.poll_sources();
            assert!(
                std::time::Instant::now() < deadline,
                "source loader did not complete"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[test]
    #[ignore = "reads full WAVs from CLEANSLATE_IMPORT_BENCHMARK"]
    fn full_recording_import_timing() {
        let paths: Vec<_> = std::env::split_paths(
            &std::env::var_os("CLEANSLATE_IMPORT_BENCHMARK").expect("set source paths"),
        )
        .collect();
        assert!(!paths.is_empty());
        let directory =
            std::env::temp_dir().join(format!("cleanslate-load-benchmark-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("benchmark.cleanslate"), vec![]).unwrap(),
            ),
            ..Default::default()
        };
        let started = std::time::Instant::now();
        for path in &paths {
            let _ = app.update(super::Message::FileDropped(path.clone()));
        }
        let drop_ms = started.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(app.loading_paths().len(), paths.len());
        assert!(app.source_loading.is_none());
        let mut waveform_ms = Vec::new();
        while app.sources_busy() {
            app.poll_sources();
            if let Some(loading) = &app.source_loading {
                while waveform_ms.len() < loading.waveforms.len() {
                    waveform_ms.push(started.elapsed().as_secs_f64() * 1000.0);
                }
            }
            assert!(started.elapsed().as_secs() < 300, "import timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(
            app.project.as_ref().unwrap().sources.len(),
            paths.len(),
            "{:?}",
            app.messages
        );
        assert_eq!(app.waveforms.len(), paths.len());
        println!(
            "IMPORT_TIMING {}",
            serde_json::json!({"tracks": paths.len(), "drop_handler_ms": drop_ms, "waveforms_ready_ms": waveform_ms, "validated_import_ms": started.elapsed().as_secs_f64() * 1000.0})
        );
        drop(app);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn import_displays_partial_waveforms_and_keeps_drops_received_during_loading() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-import-queue-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let a = directory.join("a.wav");
        let b = directory.join("b.wav");
        write_wav(&a, &[0.0; 16]);
        write_wav(&b, &[0.0; 16]);
        let mut app = super::CleanSlateApp::default();
        app.add_stems([a.clone()]);
        app.import_due = Some(std::time::Instant::now());
        app.poll_sources();
        assert!(app.source_loading.is_some());
        app.add_stems([a.clone(), b.clone()]);
        assert_eq!(app.pending_imports, std::slice::from_ref(&b));
        assert_eq!(app.loading_paths(), [a.clone(), b.clone()]);
        finish_import(&mut app);
        assert_eq!(app.project.as_ref().unwrap().source_paths(), [a, b]);
        assert_eq!(app.waveforms.len(), 2);

        // Deliver a waveform without completing validation. The window must be
        // able to draw it while retaining the previous committed session.
        let (sender, events) = super::mpsc::channel();
        app.source_loading = Some(super::SourceLoading {
            paths: app.project.as_ref().unwrap().source_paths(),
            waveforms: vec![],
            timeline: Default::default(),
            stage: "Loading waveforms",
            events,
        });
        sender
            .send(super::SourceEvent::Waveform(0, app.waveforms[0].clone()))
            .unwrap();
        app.poll_sources();
        assert_eq!(app.source_loading.as_ref().unwrap().waveforms.len(), 1);
        assert_eq!(app.project.as_ref().unwrap().sources.len(), 2);
        let _ = app.workspace(false, 900.0);
        drop(sender);
        app.poll_sources();
        assert!(!app.sources_busy());
        assert!(app.messages[0].contains("stopped unexpectedly"));
        assert_eq!(app.waveforms.len(), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn calibration_range_explains_clicks_and_short_or_reversed_ranges() {
        let mut app = super::CleanSlateApp::default();
        app.timeline.set_total_samples(48_000);
        app.capture_calibration_range(24_000, 24_000);
        assert!(
            app.calibration_range()
                .unwrap_err()
                .contains("end must be after")
        );
        app.capture_calibration_range(24_000, 24_001);
        assert!(
            app.calibration_range()
                .unwrap_err()
                .contains("0.25 seconds")
        );
        app.capture_calibration_range(48_000, 12_000);
        assert_eq!(app.calibration_range().unwrap(), (12_000, 48_000));
        app.calibration_start = "2".into();
        app.calibration_end = "1".into();
        assert!(app.calibration_range().is_err());
        app.calibration_start = "100".into();
        app.calibration_end = "101".into();
        assert!(
            app.calibration_range().is_err(),
            "Save must reject ranges beyond the recording before a click"
        );
    }

    #[test]
    fn missing_sources_can_be_relinked_one_at_a_time() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-missing-relink-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let old_a = directory.join("a.wav");
        let old_b = directory.join("b.wav");
        let new_a = directory.join("moved-a.wav");
        let new_b = directory.join("moved-b.wav");
        write_wav(&old_a, &[0.0; 16]);
        write_wav(&old_b, &[0.0; 16]);
        let saved = directory.join("session.cleanslate");
        crate::Project::create(&saved, vec![old_a.clone(), old_b.clone()]).unwrap();
        std::fs::rename(old_a, &new_a).unwrap();
        std::fs::rename(old_b, &new_b).unwrap();
        let mut app = super::CleanSlateApp::default();
        app.begin_source_loading(super::SourceRequest::Open(saved));
        finish_import(&mut app);
        app.begin_source_loading(super::SourceRequest::Relink(0, new_a));
        finish_import(&mut app);
        assert!(
            app.project.as_ref().unwrap().sources[0].available,
            "{:?}",
            app.messages
        );
        assert!(!app.project.as_ref().unwrap().sources[1].available);
        app.begin_source_loading(super::SourceRequest::Relink(1, new_b));
        finish_import(&mut app);
        assert!(
            app.project
                .as_ref()
                .unwrap()
                .sources
                .iter()
                .all(|source| source.available)
        );
        assert_eq!(app.waveforms.len(), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn inspector_stacks_only_in_compact_windows() {
        assert!(!super::compact_workspace(1280.0));
        assert!(super::compact_workspace(1000.0));
        assert!(!super::compact_workspace(1600.0));
    }

    #[test]
    fn waveform_rows_fill_height_without_becoming_unusable() {
        assert!((super::waveform_height(900.0, 3, false) - 164.67).abs() < 0.01);
        assert_eq!(super::waveform_height(640.0, 12, false), 80.0);
        assert_eq!(super::waveform_height(1600.0, 1, false), 240.0);
        assert_eq!(super::waveform_height(720.0, 3, true), 80.0);
    }

    #[test]
    fn file_dialog_starts_in_an_existing_local_directory() {
        assert!(super::dialog_start_directory().is_dir());
    }

    #[test]
    fn dropped_stems_never_overwrite_an_existing_project() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-path-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("CleanSlate.cleanslate"), b"existing").unwrap();
        assert_eq!(
            super::available_project_path(&directory),
            directory.join("CleanSlate-2.cleanslate")
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn shift_selected_range_protects_all_tracks_at_exact_bounds() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-editor-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("voice.wav");
        std::fs::write(&source, b"source").unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("episode.cleanslate"), vec![source.clone()])
                    .unwrap(),
            ),
            ..Default::default()
        };
        std::fs::remove_file(source).unwrap();
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Select(40, 10),
        ));
        assert!(app.can_protect_selection());
        let _ = app.update(super::Message::ProtectSelection);
        assert_eq!(
            app.project.unwrap().protections(),
            &[crate::Protection {
                track: None,
                start: 10,
                end: 40
            }]
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn current_render_export_readiness_tracks_parameter_staleness() {
        let directory = std::env::temp_dir().join(format!(
            "cleanslate-export-state-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("voice.wav");
        let rendered = directory.join("render/01-voice.wav");
        let report = directory.join("render/report.json");
        std::fs::write(&source, b"source").unwrap();
        std::fs::create_dir_all(report.parent().unwrap()).unwrap();
        let writer =
            bwavfile::WaveWriter::create(&rendered, bwavfile::WaveFmt::new_pcm_mono(48_000, 16))
                .unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(&[0.0_f32, 0.25]).unwrap();
        frames.end().unwrap();
        std::fs::write(&report, br#"{"output":{"stems":["01-voice.wav"]}}"#).unwrap();
        let mut project =
            crate::Project::create(directory.join("episode.cleanslate"), vec![source.clone()])
                .unwrap();
        project.mark_render_current(&report).unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        app.refresh_export_readiness();
        assert!(app.export_ready());
        let delivery = directory.join("delivery");
        app.export_to(delivery.clone());
        assert!(delivery.join("voice-clean.wav").is_file());
        assert!(!delivery.join("report.json").exists());
        assert!(app.messages[0].starts_with("Export committed"));
        let _ = app.update(super::Message::Attenuation(-12.0));
        assert!(!app.export_ready());
        let blocked = directory.join("during-drag");
        app.export_to(blocked.clone());
        assert!(!blocked.exists());
        assert!(matches!(
            app.project.as_ref().unwrap().render(),
            crate::RenderState::Current { .. }
        ));
        app.preview_parameters = app.project.as_ref().unwrap().parameters();
        std::fs::remove_file(rendered).unwrap();
        app.refresh_export_readiness();
        assert!(!app.export_ready());
        assert!(app.export_error.is_some());
        std::fs::remove_file(source).unwrap();
        app.preview_parameters.attenuation_db = -12.0;
        let _ = app.update(super::Message::RefreshPreview);
        assert!(!app.export_ready());
        assert_eq!(app.project.unwrap().render(), &crate::RenderState::Stale);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

pub struct CleanSlateApp {
    project: Option<Project>,
    pending_imports: Vec<PathBuf>,
    import_due: Option<Instant>,
    source_loading: Option<SourceLoading>,
    messages: Vec<String>,
    processing: Option<Processing>,
    review: Option<ReviewWorkspace>,
    audition: Option<LiveAudition>,
    audition_preparation: AuditionPreparation,
    audition_range: Option<(usize, usize)>,
    audition_mode: AuditionMode,
    monitor_db: f32,
    selecting_range: bool,
    calibrating: bool,
    calibration_timing: bool,
    calibration_start: String,
    calibration_end: String,
    calibration_error: Option<String>,
    waveforms: Vec<WaveformCache>,
    timeline: TimelineViewport,
    preview_parameters: AutomixParameters,
    preview: Option<AnalysisPreview>,
    preview_sources: Vec<waveform::SourceStamp>,
    preview_cache: PreviewCache,
    preview_processing: Option<PreviewProcessing>,
    preview_pending: Option<Instant>,
    preview_generation: u64,
    source_generation: u64,
    preview_error: Option<String>,
    export_error: Option<String>,
}

impl Default for CleanSlateApp {
    fn default() -> Self {
        Self {
            project: None,
            pending_imports: vec![],
            import_due: None,
            source_loading: None,
            messages: vec![],
            processing: None,
            review: None,
            audition: None,
            audition_preparation: AuditionPreparation::default(),
            audition_range: None,
            audition_mode: AuditionMode::Original,
            monitor_db: -6.0,
            selecting_range: false,
            calibrating: false,
            calibration_timing: false,
            calibration_start: String::new(),
            calibration_end: String::new(),
            calibration_error: None,
            waveforms: vec![],
            timeline: TimelineViewport::default(),
            preview_parameters: AutomixParameters::default(),
            preview: None,
            preview_sources: vec![],
            preview_cache: PreviewCache::default(),
            preview_processing: None,
            preview_pending: None,
            preview_generation: 0,
            source_generation: 0,
            preview_error: None,
            export_error: None,
        }
    }
}

enum SourceRequest {
    Add(Vec<PathBuf>),
    Open(PathBuf),
    Relink(usize, PathBuf),
    Validate,
}

struct LoadedSession {
    project: Project,
    warnings: Vec<String>,
    review: Option<ReviewWorkspace>,
    export_error: Option<String>,
}

enum SourceEvent {
    Paths(Vec<PathBuf>),
    Waveform(usize, WaveformCache),
    Stage(&'static str),
    Finished(Result<Box<LoadedSession>, String>),
}

struct SourceLoading {
    paths: Vec<PathBuf>,
    waveforms: Vec<WaveformCache>,
    timeline: TimelineViewport,
    stage: &'static str,
    events: Receiver<SourceEvent>,
}

// All source reads, validation, hashing, cache persistence and saved-review reads
// belong to this worker. The window only receives display data and a finished session.
fn load_sources(
    request: SourceRequest,
    project: Option<Project>,
    parameters: AutomixParameters,
    sender: &mpsc::Sender<SourceEvent>,
) -> Result<LoadedSession, String> {
    let mut project = project;
    let opening = matches!(request, SourceRequest::Open(_));
    let partial_relink = if let SourceRequest::Relink(index, path) = &request {
        // A replacement can be checked independently while other missing or
        // changed sources still await the user's relink decisions.
        validate(std::slice::from_ref(path)).map_err(|error| format!("Import blocked: {error}"))?;
        project.as_ref().is_some_and(|project| {
            project
                .sources
                .iter()
                .enumerate()
                .any(|(other, source)| other != *index && !source.available)
        })
    } else {
        false
    };
    if let SourceRequest::Open(path) = &request {
        project = Some(Project::open(path).map_err(|error| error.to_string())?);
    }
    let mut paths = project
        .as_ref()
        .map_or_else(Vec::new, Project::source_paths);
    match &request {
        SourceRequest::Add(additions) => paths.extend(additions.iter().cloned()),
        SourceRequest::Relink(index, path) => {
            *paths
                .get_mut(*index)
                .ok_or("Source track is not in this session.")? = path.clone();
        }
        _ => {}
    }
    let _ = sender.send(SourceEvent::Paths(paths.clone()));
    if paths.is_empty() {
        return Err("No source tracks were selected.".into());
    }
    let project_path = project.as_ref().map_or_else(
        || {
            available_project_path(
                paths[0]
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(".")),
            )
        },
        |project| project.path().to_owned(),
    );
    let _ = sender.send(SourceEvent::Stage("Loading waveforms"));
    let waveform_result =
        waveform::load_or_build_progress(&project_path, &paths, |index, waveform| {
            let _ = sender.send(SourceEvent::Waveform(index, waveform.clone()));
        });
    let mut warnings = Vec::new();
    // A saved session with missing sources must still open so it can be relinked.
    if opening || partial_relink {
        if let Err(error) = waveform_result {
            warnings.push(format!("Waveforms unavailable: {error}"));
        }
    } else {
        waveform_result.map_err(|error| format!("Import blocked: {error}"))?;
        let _ = sender.send(SourceEvent::Stage("Validating audio"));
        warnings = validate(&paths)
            .map_err(|error| format!("Import blocked: {error}"))?
            .warnings;
    }
    let _ = sender.send(SourceEvent::Stage("Saving source identities"));
    match request {
        SourceRequest::Add(additions) => {
            if let Some(project) = &mut project {
                project
                    .add_sources(additions)
                    .map_err(|error| error.to_string())?;
            } else {
                let mut created =
                    Project::create(project_path, additions).map_err(|error| error.to_string())?;
                created
                    .set_parameters(parameters)
                    .map_err(|error| error.to_string())?;
                created
                    .mark_validated()
                    .map_err(|error| error.to_string())?;
                project = Some(created);
            }
        }
        SourceRequest::Relink(index, path) => {
            let project = project.as_mut().ok_or("No session to relink.")?;
            project
                .relink(index, path)
                .map_err(|error| error.to_string())?;
            if !partial_relink {
                project
                    .mark_validated()
                    .map_err(|error| error.to_string())?;
            } else {
                warnings.push("Replacement saved. Relink the remaining unavailable sources to complete validation.".into());
            }
        }
        SourceRequest::Validate => project
            .as_mut()
            .ok_or("No session to validate.")?
            .mark_validated()
            .map_err(|error| error.to_string())?,
        SourceRequest::Open(_) => {}
    }
    let project = project.ok_or("No session was loaded.")?;
    let review = if opening {
        project
            .review_identity()
            .and_then(|identity| ReviewWorkspace::open(identity, &paths).ok())
    } else {
        None
    };
    let export_error = if matches!(project.render(), RenderState::Current { .. }) {
        validate_export(&project)
            .err()
            .map(|error| error.to_string())
    } else {
        None
    };
    Ok(LoadedSession {
        project,
        warnings,
        review,
        export_error,
    })
}

struct Processing {
    cancelled: Arc<AtomicBool>,
    progress: Arc<AtomicU8>,
    result: Receiver<Result<String, String>>,
    snapshot: RenderSnapshot,
}

#[derive(PartialEq)]
struct RenderSnapshot {
    project: PathBuf,
    sources: Vec<waveform::SourceStamp>,
    parameters: AutomixParameters,
    protections: Vec<crate::Protection>,
    ownership: OwnershipSettings,
}

impl RenderSnapshot {
    fn read(project: &Project) -> Result<Self, String> {
        Ok(Self {
            project: project.path().to_owned(),
            sources: source_stamps(&project.source_paths()).map_err(|error| error.to_string())?,
            parameters: project.parameters(),
            protections: project.protections().to_vec(),
            ownership: project.ownership_settings().clone(),
        })
    }
}

struct PreviewProcessing {
    progress: Arc<AtomicU8>,
    cancelled: Arc<AtomicBool>,
    generation: u64,
    source_generation: u64,
    sources: Vec<waveform::SourceStamp>,
    result: Receiver<(Result<AnalysisPreview, String>, PreviewCache)>,
}

impl Drop for PreviewProcessing {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
enum Message {
    ImportStems,
    OpenProject,
    FileDropped(PathBuf),
    Relink(usize),
    Validate,
    Process,
    Cancel,
    Tick,
    SpeechSensitivity(f32),
    BleedDiscrimination(f32),
    Attenuation(f32),
    SpeechPadding(f32),
    OpenTime(f32),
    CloseTime(f32),
    RefreshPreview,
    SelectWarning(usize),
    Original,
    Processed,
    Removed,
    Audition,
    AuditionSelection,
    Stop,
    Previous,
    MonitorGain(f32),
    KeyPressed(keyboard::Key),
    RangeModifier(bool),
    OwnershipMode(bool),
    CalibrationMode(bool),
    CalibrationTiming(bool),
    CalibrationStart(String),
    CalibrationEnd(String),
    CalibrationStartAtPlayhead,
    CalibrationEndAtPlayhead,
    CalibrationFromSelection,
    SaveCalibration(CalibrationKind, usize),
    RemoveCalibration(usize),
    ToggleReviewed,
    ProtectRegion,
    ProtectTrack(usize),
    ProtectSelection,
    Export,
    Timeline(TimelineEvent),
}

pub fn run() -> iced::Result {
    iced::application(
        CleanSlateApp::default,
        CleanSlateApp::update,
        CleanSlateApp::view,
    )
    .title("CleanSlate")
    .theme(CleanSlateApp::theme)
    .subscription(CleanSlateApp::subscription)
    .window(window::Settings {
        maximized: true,
        min_size: Some(iced::Size::new(960.0, 640.0)),
        ..window::Settings::default()
    })
    .run()
}

fn compact_workspace(width: f32) -> bool {
    width < 1100.0
}

fn waveform_height(workspace_height: f32, stem_count: usize, compact: bool) -> f32 {
    let reserved = if compact { 580.0 } else { 220.0 };
    ((workspace_height - reserved) / stem_count.max(1) as f32 - 62.0).clamp(80.0, 240.0)
}

impl CleanSlateApp {
    fn install_waveforms(&mut self, waveforms: Vec<WaveformCache>) {
        self.calibrating = false;
        self.calibration_start.clear();
        self.calibration_end.clear();
        self.calibration_error = None;
        self.audition_preparation.invalidate();
        self.audition_range = None;
        if let Some(audition) = self.audition.take() {
            audition.stop();
        }
        self.preview = None;
        self.preview_sources.clear();
        self.preview_cache = PreviewCache::default();
        self.source_generation += 1;
        self.preview_generation += 1;
        self.preview_pending = None;
        if let Some(processing) = &self.preview_processing {
            processing.cancelled.store(true, Ordering::Relaxed);
        }
        if let Some(cache) = waveforms.first() {
            self.timeline.set_total_samples(cache.sample_count);
        }
        self.waveforms = waveforms;
    }

    fn add_stems(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        if self.processing.is_some() {
            self.messages =
                vec!["Wait for the final render to finish before importing tracks.".into()];
            return;
        }
        let existing = self.loading_paths();
        let mut additions: Vec<_> = paths
            .into_iter()
            .filter(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"))
            })
            .filter(|path| {
                !existing.iter().any(|old| {
                    old.to_string_lossy()
                        .eq_ignore_ascii_case(&path.to_string_lossy())
                })
            })
            .collect();
        additions.sort_by_key(|path| path.to_string_lossy().to_lowercase());
        additions.dedup_by(|a, b| {
            a.to_string_lossy()
                .eq_ignore_ascii_case(&b.to_string_lossy())
        });
        if additions.is_empty() {
            self.messages = vec!["No new WAV stems were selected".into()];
            return;
        }

        self.pending_imports.extend(additions);
        self.pending_imports
            .sort_by_key(|path| path.to_string_lossy().to_lowercase());
        // A desktop multi-file drop arrives as separate events. Collect the burst
        // before starting one source pass, without delaying placeholder rows.
        self.import_due = Some(Instant::now() + Duration::from_millis(150));
        self.messages.clear();
        self.stop_audition();
        self.preview_pending = None;
        self.preview_generation += 1;
        self.preview_processing = None;
    }

    fn sources_busy(&self) -> bool {
        self.source_loading.is_some() || !self.pending_imports.is_empty()
    }

    fn loading_paths(&self) -> Vec<PathBuf> {
        let mut paths = self.source_loading.as_ref().map_or_else(
            || {
                self.project
                    .as_ref()
                    .map_or_else(Vec::new, Project::source_paths)
            },
            |loading| loading.paths.clone(),
        );
        paths.extend(self.pending_imports.iter().cloned());
        paths
    }

    fn begin_source_loading(&mut self, request: SourceRequest) {
        if self.source_loading.is_some() || self.processing.is_some() {
            return;
        }
        self.stop_audition();
        self.preview_pending = None;
        self.preview_generation += 1;
        self.preview_processing = None;
        let project = self.project.clone();
        let parameters = self.preview_parameters;
        let mut paths = project
            .as_ref()
            .map_or_else(Vec::new, Project::source_paths);
        match &request {
            SourceRequest::Add(additions) => paths.extend(additions.iter().cloned()),
            SourceRequest::Open(_) => paths.clear(),
            SourceRequest::Relink(index, path) => {
                if let Some(source) = paths.get_mut(*index) {
                    *source = path.clone();
                }
            }
            SourceRequest::Validate => {}
        }
        let (sender, events) = mpsc::channel();
        std::thread::spawn(move || {
            let result = load_sources(request, project, parameters, &sender).map(Box::new);
            let _ = sender.send(SourceEvent::Finished(result));
        });
        self.source_loading = Some(SourceLoading {
            paths,
            waveforms: vec![],
            timeline: TimelineViewport::default(),
            stage: "Opening audio",
            events,
        });
    }

    fn poll_sources(&mut self) {
        while let Some(loading) = &mut self.source_loading {
            match loading.events.try_recv() {
                Ok(SourceEvent::Paths(paths)) => loading.paths = paths,
                Ok(SourceEvent::Stage(stage)) => loading.stage = stage,
                Ok(SourceEvent::Waveform(index, waveform)) => {
                    if index == loading.waveforms.len() {
                        if index == 0 {
                            loading.timeline.set_total_samples(waveform.sample_count);
                        }
                        loading.waveforms.push(waveform);
                    }
                }
                result => {
                    let result = match result {
                        Ok(SourceEvent::Finished(result)) => result,
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            Err("Source loader stopped unexpectedly.".into())
                        }
                        _ => unreachable!(),
                    };
                    let loading = self.source_loading.take().unwrap();
                    match result {
                        Ok(loaded) => {
                            self.preview_parameters = loaded.project.parameters();
                            self.project = Some(loaded.project);
                            self.install_waveforms(loading.waveforms);
                            self.review = loaded.review;
                            self.export_error = loaded.export_error;
                            self.messages = loaded.warnings;
                        }
                        Err(error) => self.messages = vec![error],
                    }
                    if self.pending_imports.is_empty() {
                        self.start_preview();
                    }
                    break;
                }
            }
        }
        if self.source_loading.is_none() && self.import_due.is_some_and(|due| Instant::now() >= due)
        {
            self.import_due = None;
            let existing = self
                .project
                .as_ref()
                .map_or_else(Vec::new, Project::source_paths);
            let additions: Vec<_> = std::mem::take(&mut self.pending_imports)
                .into_iter()
                .filter(|path| {
                    !existing.iter().any(|old| {
                        old.to_string_lossy()
                            .eq_ignore_ascii_case(&path.to_string_lossy())
                    })
                })
                .collect();
            if !additions.is_empty() {
                self.begin_source_loading(SourceRequest::Add(additions));
            } else {
                self.start_preview();
            }
        }
    }

    fn import_stems(&mut self) {
        let Some(paths) = rfd::FileDialog::new()
            .set_directory(dialog_start_directory())
            .add_filter("WAV audio", &["wav"])
            .pick_files()
        else {
            return;
        };
        self.add_stems(paths);
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn export_current(&mut self) {
        let Some(destination) = rfd::FileDialog::new()
            .set_directory(
                self.project
                    .as_ref()
                    .and_then(|project| project.path().parent())
                    .unwrap_or_else(|| std::path::Path::new(".")),
            )
            .pick_folder()
        else {
            return;
        };
        self.export_to(destination);
    }

    fn export_to(&mut self, destination: PathBuf) {
        if !self.export_ready() {
            self.messages =
                vec!["Export blocked: render the current settings before exporting".into()];
            return;
        }
        let Some(project) = &self.project else { return };
        match export(project, &destination, None) {
            Ok(()) => {
                self.messages = vec![format!("Export committed to {}", destination.display())]
            }
            Err(error) => self.messages = vec![format!("Export blocked: {error}")],
        }
    }

    fn export_ready(&self) -> bool {
        !self.sources_busy()
            && self.processing.is_none()
            && self.export_error.is_none()
            && self.project.as_ref().is_some_and(|project| {
                project.sources.iter().all(|source| source.available)
                    && project.parameters() == self.preview_parameters
                    && matches!(project.render(), RenderState::Current { identity } if identity.is_file())
            })
    }

    fn refresh_export_readiness(&mut self) {
        self.export_error = self.project.as_ref().and_then(|project| {
            if matches!(project.render(), RenderState::Current { .. }) {
                validate_export(project)
                    .err()
                    .map(|error| error.to_string())
            } else {
                None
            }
        });
    }

    fn open_project(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_directory(dialog_start_directory())
            .add_filter("CleanSlate project", &["cleanslate"])
            .pick_file()
        else {
            return;
        };
        self.begin_source_loading(SourceRequest::Open(path));
    }

    fn validate(&mut self) {
        if self.project.is_some() {
            self.begin_source_loading(SourceRequest::Validate);
        }
    }

    fn start_processing(&mut self) {
        if self.processing.is_some() {
            return;
        }
        let Some(project) = &self.project else { return };
        let snapshot = match RenderSnapshot::read(project) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.messages = vec![format!("Final render blocked: {error}")];
                return;
            }
        };
        let stems = project.source_paths();
        let protections = project.protections().to_vec();
        let parameters = project.parameters();
        let ownership = snapshot.ownership.clone();
        let parent = project
            .path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();
        let output_dir = parent.join(format!("render-{id}"));
        let report = output_dir.join("report.json");
        let cancelled = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU8::new(0));
        let request_cancelled = Arc::clone(&cancelled);
        let request_progress = Arc::clone(&progress);
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let outcome = run_with_ownership(
                RunRequest {
                    stems,
                    output_dir,
                    regions: vec![],
                    protections,
                    cancelled: Some(request_cancelled),
                },
                Some(request_progress),
                parameters,
                &ownership,
            )
            .map(|_| report.display().to_string())
            .map_err(|error| error.to_string());
            let _ = sender.send(outcome);
        });
        self.messages = vec!["Final render started".into()];
        self.processing = Some(Processing {
            cancelled,
            progress,
            result,
            snapshot,
        });
    }

    fn start_preview(&mut self) {
        self.queue_preview(Duration::ZERO);
        self.launch_preview();
    }

    fn queue_preview(&mut self, delay: Duration) {
        if self.project.is_none() {
            return;
        }
        // Drag events belong to the next coalesced snapshot. Keep the current
        // snapshot installable until that boundary, even under continuous input.
        if delay.is_zero() {
            self.preview_generation += 1;
        }
        let due = Instant::now() + delay;
        self.preview_pending = Some(self.preview_pending.map_or(due, |old| old.min(due)));
        self.preview_error = None;
        if let Some(processing) = &self.preview_processing {
            let sources = source_stamps(&self.project.as_ref().unwrap().source_paths());
            // Parameter edits must not cancel initial source inference. At 70,
            // analyze has retained its facts and cancellation can preserve them.
            if sources.as_ref().ok() != Some(&processing.sources)
                || (delay.is_zero() && processing.progress.load(Ordering::Relaxed) >= 70)
            {
                processing.cancelled.store(true, Ordering::Relaxed);
            }
        }
    }

    fn launch_preview(&mut self) {
        if self.preview_processing.is_some()
            || self.preview_pending.is_none_or(|due| Instant::now() < due)
        {
            return;
        }
        self.preview_pending = None;
        self.preview_generation += 1;
        let Some(project) = &self.project else { return };
        let stems = project.source_paths();
        let sources = match source_stamps(&stems) {
            Ok(sources) => sources,
            Err(error) => {
                self.preview_error = Some(error.to_string());
                return;
            }
        };
        let protections = project.protections().to_vec();
        let parameters = self.preview_parameters;
        let ownership = project.ownership_settings().clone();
        let progress = Arc::new(AtomicU8::new(0));
        let worker_progress = Arc::clone(&progress);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let mut cache = self.preview_cache.clone();
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let result = cache
                .analyze_with_ownership(
                    &stems,
                    &protections,
                    parameters,
                    &ownership,
                    &worker_progress,
                    Some(&worker_cancelled),
                )
                .map_err(|error| error.to_string());
            let _ = sender.send((result, cache));
        });
        self.preview_processing = Some(PreviewProcessing {
            progress,
            cancelled,
            result,
            sources,
            generation: self.preview_generation,
            source_generation: self.source_generation,
        });
        self.preview_error = None;
    }

    fn load_review(&mut self) {
        self.review = self.project.as_ref().and_then(|project| {
            let identity = project.review_identity()?;
            ReviewWorkspace::open(identity, &project.source_paths()).ok()
        });
        if let Some(review) = &self.review {
            self.timeline.seek(review.playhead());
            self.timeline.set_selection(review.selection());
        }
    }

    fn audition(&mut self, selection_only: bool) {
        let (Some(project), Some(preview)) = (&self.project, &self.preview) else {
            return;
        };
        let selection = if selection_only {
            self.timeline.selection()
        } else {
            None
        };
        let start = selection.map_or(self.timeline.playhead(), |range| range.0.min(range.1));
        let end = selection.map_or_else(
            || {
                preview
                    .envelopes
                    .iter()
                    .map(|envelope| envelope.sample_count())
                    .max()
                    .unwrap_or(start)
            },
            |range| range.0.max(range.1),
        );
        self.audition.take().inspect(|playing| playing.stop());
        self.audition_range = Some((start, end));
        self.audition_preparation.request(project.source_paths());
    }

    fn poll_processing(&mut self) {
        if let Some(processing) = &self.preview_processing
            && self
                .preview_pending
                .is_some_and(|due| Instant::now() >= due)
            && processing.generation == self.preview_generation
        {
            self.preview_generation += 1;
        }
        if let Some(processing) = &self.preview_processing
            && processing.generation != self.preview_generation
            && processing.progress.load(Ordering::Relaxed) >= 70
        {
            processing.cancelled.store(true, Ordering::Relaxed);
        }
        if let Some(processing) = &self.preview_processing
            && let result = processing.result.try_recv()
            && !matches!(result, Err(mpsc::TryRecvError::Empty))
        {
            let processing = self.preview_processing.take().unwrap();
            let (result, cache) = result.unwrap_or_else(|_| {
                (
                    Err("preview worker stopped unexpectedly".into()),
                    self.preview_cache.clone(),
                )
            });
            let sources_match = self.project.as_ref().is_some_and(|project| {
                source_stamps(&project.source_paths()).as_ref().ok() == Some(&processing.sources)
            });
            if processing.source_generation == self.source_generation && sources_match {
                self.preview_cache = cache;
            }
            if processing.generation == self.preview_generation {
                let result = result.and_then(|preview| {
                    if sources_match {
                        Ok(preview)
                    } else {
                        Err("source changed before preview installation; retry".into())
                    }
                });
                match result {
                    Ok(preview) => {
                        if self.preview_sources != processing.sources {
                            self.audition_preparation.invalidate();
                            self.audition_range = None;
                            if let Some(audition) = self.audition.take() {
                                audition.stop();
                            }
                        }
                        if let Some(audition) = &self.audition {
                            audition.replace_envelopes(preview.envelopes.clone());
                        }
                        self.preview_sources = processing.sources.clone();
                        self.preview = Some(preview);
                    }
                    Err(error) => self.preview_error = Some(error),
                }
            }
        }
        self.launch_preview();
        let Some(processing) = &self.processing else {
            return;
        };
        let Ok(result) = processing.result.try_recv() else {
            return;
        };
        let cancelled = processing.cancelled.load(Ordering::Relaxed);
        let current = self
            .project
            .as_ref()
            .and_then(|project| RenderSnapshot::read(project).ok());
        let current = current.as_ref() == Some(&processing.snapshot)
            && self.preview_parameters == processing.snapshot.parameters;
        let same_project = self
            .project
            .as_ref()
            .is_some_and(|project| project.path() == processing.snapshot.project);
        self.processing = None;
        match result {
            Ok(_) if !current => {
                self.messages = vec!["Final render completed for earlier settings or sources; render again before exporting".into()];
                if same_project
                    && let Some(project) = &mut self.project
                    && let Err(error) = project.mark_render_stale()
                {
                    self.messages.push(error.to_string());
                }
                self.refresh_export_readiness();
            }
            Ok(identity) => {
                let project = self.project.as_mut().unwrap();
                self.messages = vec!["Final render current; ready to export WAVs".into()];
                if let Err(error) = project.mark_render_current(identity) {
                    self.messages.push(error.to_string());
                }
                self.refresh_export_readiness();
                if let Some(error) = &self.export_error {
                    self.messages =
                        vec![format!("Final render completed; export blocked: {error}")];
                }
                self.load_review();
            }
            Err(error) => {
                self.messages = vec![if cancelled {
                    "Final render cancelled; no output committed".into()
                } else {
                    format!("Final render failed: {error}")
                }]
            }
        }
    }

    fn poll_audition(&mut self) {
        if let Some(result) = self.audition_preparation.poll()
            && let Some((start, end)) = self.audition_range.take()
        {
            match result {
                Ok(prepared) => {
                    if let (Some(project), Some(preview)) = (&self.project, &self.preview) {
                        if prepared.is_current(&project.source_paths())
                            && prepared.matches_sources(&self.preview_sources)
                        {
                            match LiveAudition::start(
                                prepared,
                                preview.envelopes.clone(),
                                self.audition_mode,
                                10.0_f32.powf(self.monitor_db / 20.0),
                                start,
                                end,
                            ) {
                                Ok(audition) => self.audition = Some(audition),
                                Err(error) => self.messages = vec![error.to_string()],
                            }
                        } else {
                            self.audition_preparation.invalidate();
                            self.messages = vec![
                                "Sources changed; prepare the session again before audition".into(),
                            ];
                        }
                    }
                }
                Err(error) => self.messages = vec![error.to_string()],
            }
        }
        let Some(audition) = &self.audition else {
            return;
        };
        self.timeline.seek(audition.position());
        if let Some(review) = &mut self.review {
            review.seek(self.timeline.playhead());
        }
        if audition.is_finished() {
            self.audition = None;
        }
    }

    fn set_audition_mode(&mut self, mode: AuditionMode) {
        self.audition_mode = mode;
        if let Some(audition) = &self.audition {
            audition.set_mode(mode);
        }
    }

    fn stop_audition(&mut self) {
        self.audition_range = None;
        self.audition_preparation.cancel();
        if let Some(audition) = self.audition.take() {
            audition.stop();
        }
    }

    fn seek_to_selection_start(&mut self) {
        let start = self
            .timeline
            .selection()
            .map_or(0, |range| range.0.min(range.1));
        self.timeline.seek(start);
        if let Some(range) = &mut self.audition_range {
            range.0 = start.min(range.1);
        }
        if let Some(audition) = &self.audition {
            audition.seek(start);
        }
        if let Some(review) = &mut self.review {
            review.seek(start);
        }
    }

    fn can_protect_selection(&self) -> bool {
        self.project.is_some() && self.timeline.selection().is_some()
    }

    fn save_ownership(&mut self, settings: OwnershipSettings) {
        let Some(project) = &mut self.project else {
            return;
        };
        match project.set_ownership_settings(settings) {
            Ok(()) => {
                self.calibration_error = None;
                self.start_preview();
            }
            Err(error) => self.calibration_error = Some(error.to_string()),
        }
    }

    fn save_calibration(&mut self, kind: CalibrationKind, track: usize) {
        let result = (|| {
            let project = self
                .project
                .as_ref()
                .ok_or("Import sources before calibrating.")?;
            if !project.ownership_settings().enabled {
                return Err("Enable joint ownership before adding examples.".to_string());
            }
            let (start, end) = self.calibration_range()?;
            let example = CalibrationExample {
                kind,
                track,
                start,
                end,
            };
            let mut settings = project.ownership_settings().clone();
            if settings.examples.is_empty() {
                settings.sources =
                    source_stamps(&project.source_paths()).map_err(|error| error.to_string())?;
            }
            if !settings.examples.contains(&example) {
                settings.examples.push(example);
            }
            Ok(settings)
        })();
        match result {
            Ok(settings) => self.save_ownership(settings),
            Err(error) => self.calibration_error = Some(error),
        }
    }

    fn capture_calibration_range(&mut self, start: usize, end: usize) {
        self.calibration_start = format!("{:.8}", start.min(end) as f64 / 48_000.0);
        self.calibration_end = format!("{:.8}", start.max(end) as f64 / 48_000.0);
        self.calibration_error = None;
    }

    fn calibration_range(&self) -> Result<(usize, usize), String> {
        let sample = |value: &str| {
            let seconds = value.trim().parse::<f64>().map_err(
                |_| "Drag across a waveform to select an example, or enter start and end seconds.",
            )?;
            let samples = seconds * 48_000.0;
            if !samples.is_finite() || samples < 0.0 || samples >= usize::MAX as f64 {
                return Err("Times must be finite, nonnegative seconds within the recording.");
            }
            Ok(samples.round() as usize)
        };
        let start = sample(&self.calibration_start)?;
        let end = sample(&self.calibration_end)?;
        if end <= start {
            return Err("No usable range: the end must be after the start. Drag across the waveform rather than clicking once.".into());
        }
        if end - start < 12_000 {
            return Err(
                "Select at least 0.25 seconds. A few seconds is easier to judge by listening."
                    .into(),
            );
        }
        if end > self.timeline.total_samples() {
            return Err("Selected range extends past the end of the recording. Choose times within the waveform.".into());
        }
        Ok((start, end))
    }
}

impl CleanSlateApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        if self.sources_busy()
            && let Message::Timeline(event) = &message
        {
            if let Some(loading) = &mut self.source_loading {
                match *event {
                    TimelineEvent::Zoom {
                        cursor_x,
                        width,
                        direction,
                    } => loading.timeline.zoom_at(cursor_x, width, direction),
                    TimelineEvent::Pan { delta_x, width } => {
                        loading.timeline.pan_by_pixels(delta_x, width)
                    }
                    _ => {}
                }
            }
            return Task::none();
        }
        // Keep navigation and further drops responsive, but don't let edits race
        // the session snapshot being validated and saved by the source worker.
        if self.sources_busy()
            && !matches!(
                message,
                Message::FileDropped(_)
                    | Message::ImportStems
                    | Message::Tick
                    | Message::Stop
                    | Message::Timeline(_)
                    | Message::RangeModifier(_)
            )
        {
            return Task::none();
        }
        let previous_parameters = self.preview_parameters;
        match message {
            Message::ImportStems => self.import_stems(),
            Message::OpenProject => self.open_project(),
            Message::FileDropped(path) => {
                if path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
                {
                    self.add_stems([path]);
                } else {
                    self.messages = vec![format!("Ignored non-WAV file: {}", path.display())];
                }
            }
            Message::Relink(index) => {
                let start = self
                    .project
                    .as_ref()
                    .and_then(|project| project.sources.get(index))
                    .and_then(|source| source.path.parent())
                    .map(std::path::Path::to_path_buf)
                    .unwrap_or_else(dialog_start_directory);
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(start)
                    .add_filter("WAV audio", &["wav"])
                    .pick_file()
                {
                    self.begin_source_loading(SourceRequest::Relink(index, path));
                }
            }
            Message::Validate => self.validate(),
            Message::Process => self.start_processing(),
            Message::Cancel => {
                if let Some(processing) = &self.processing {
                    processing.cancelled.store(true, Ordering::Relaxed);
                }
            }
            Message::Tick => {
                self.poll_sources();
                if !self.sources_busy() {
                    self.poll_processing();
                }
                self.poll_audition();
            }
            Message::SpeechSensitivity(value) => {
                self.preview_parameters.speech_sensitivity = 1.0 - value
            }
            Message::BleedDiscrimination(value) => {
                self.preview_parameters.bleed_discrimination_db = value
            }
            Message::Attenuation(value) => self.preview_parameters.attenuation_db = value,
            Message::SpeechPadding(value) => {
                self.preview_parameters.speech_padding_ms = effective_padding_ms(value)
            }
            Message::OpenTime(value) => self.preview_parameters.open_ms = value,
            Message::CloseTime(value) => self.preview_parameters.close_ms = value,
            Message::RefreshPreview => {
                if let Some(project) = &mut self.project
                    && let Err(error) = project.set_parameters(self.preview_parameters)
                {
                    self.messages.push(error.to_string());
                }
                self.start_preview();
            }
            Message::SelectWarning(index) => {
                if let Some(review) = &mut self.review {
                    review.select_warning(index);
                    self.timeline.seek(review.playhead());
                    self.timeline.set_selection(review.selection());
                }
                self.audition(true);
            }
            Message::Original | Message::Processed | Message::Removed => {
                let mode = match message {
                    Message::Original => AuditionMode::Original,
                    Message::Processed => AuditionMode::Processed,
                    _ => AuditionMode::Removed,
                };
                self.set_audition_mode(mode);
                if mode != AuditionMode::Removed
                    && let Some(review) = &mut self.review
                {
                    review.set_comparison(if matches!(message, Message::Original) {
                        Comparison::Original
                    } else {
                        Comparison::Processed
                    });
                }
            }
            Message::Audition => self.audition(false),
            Message::AuditionSelection => self.audition(true),
            Message::Stop => {
                self.stop_audition();
            }
            Message::Previous => self.seek_to_selection_start(),
            Message::MonitorGain(value) => {
                self.monitor_db = value;
                if let Some(audition) = &self.audition {
                    audition.set_monitor(10.0_f32.powf(value / 20.0));
                }
            }
            Message::KeyPressed(key) => match key.as_ref() {
                keyboard::Key::Named(keyboard::key::Named::Space) => {
                    if self.audition.is_some() || self.audition_range.is_some() {
                        self.stop_audition();
                    } else {
                        self.audition(false);
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Home) => self.seek_to_selection_start(),
                keyboard::Key::Character("1") => self.set_audition_mode(AuditionMode::Original),
                keyboard::Key::Character("2") => self.set_audition_mode(AuditionMode::Processed),
                keyboard::Key::Character("3") => self.set_audition_mode(AuditionMode::Removed),
                _ => {}
            },
            Message::RangeModifier(selecting) => self.selecting_range = selecting,
            Message::OwnershipMode(enabled) => {
                if !enabled {
                    self.calibrating = false;
                }
                if let Some(project) = &self.project {
                    let mut settings = project.ownership_settings().clone();
                    settings.enabled = enabled;
                    self.save_ownership(settings);
                }
            }
            Message::CalibrationMode(enabled) => self.calibrating = enabled,
            Message::CalibrationTiming(enabled) => self.calibration_timing = enabled,
            Message::CalibrationStart(value) => {
                self.calibration_start = value;
                self.calibration_error = None;
            }
            Message::CalibrationEnd(value) => {
                self.calibration_end = value;
                self.calibration_error = None;
            }
            Message::CalibrationStartAtPlayhead => {
                self.calibration_start =
                    format!("{:.8}", self.timeline.playhead() as f64 / 48_000.0);
            }
            Message::CalibrationEndAtPlayhead => {
                self.calibration_end = format!("{:.8}", self.timeline.playhead() as f64 / 48_000.0);
            }
            Message::CalibrationFromSelection => {
                if let Some((start, end)) = self.timeline.selection() {
                    self.capture_calibration_range(start, end);
                }
            }
            Message::SaveCalibration(kind, track) => self.save_calibration(kind, track),
            Message::RemoveCalibration(index) => {
                if let Some(project) = &self.project {
                    let mut settings = project.ownership_settings().clone();
                    if index < settings.examples.len() {
                        settings.examples.remove(index);
                        if settings.examples.is_empty() {
                            settings.sources.clear();
                        }
                        self.save_ownership(settings);
                    }
                }
            }
            Message::ToggleReviewed => {
                if let (Some(project), Some(key)) = (
                    self.project.as_mut(),
                    self.review.as_ref().and_then(ReviewWorkspace::selected_key),
                ) {
                    let state = if project.review_states().get(&key) == Some(&ReviewState::Reviewed)
                    {
                        ReviewState::Unreviewed
                    } else {
                        ReviewState::Reviewed
                    };
                    if let Err(error) = project.set_review_state(key, state) {
                        self.messages.push(error.to_string());
                    }
                }
            }
            Message::ProtectRegion => {
                if let (Some(project), Some(review)) = (self.project.as_mut(), &self.review) {
                    let result = if let Some(warning) = review.selected_warning() {
                        project.protect_region(&warning.affected_tracks, warning.start, warning.end)
                    } else if let Some(region) = review.selected_region() {
                        project.protect_region(
                            &region.affected_track.into_iter().collect::<Vec<_>>(),
                            region.start,
                            region.end,
                        )
                    } else {
                        Ok(())
                    };
                    if let Err(error) = result {
                        self.messages.push(error.to_string());
                    }
                }
                if let Some(review) = &mut self.review {
                    review.set_comparison(Comparison::Original);
                }
                self.start_preview();
            }
            Message::ProtectTrack(track) => {
                if let Some(project) = &mut self.project
                    && let Err(error) = project.protect_track(track)
                {
                    self.messages.push(error.to_string());
                }
                if let Some(review) = &mut self.review {
                    review.set_comparison(Comparison::Original);
                }
                self.start_preview();
            }
            Message::ProtectSelection => {
                if let (Some(project), Some((start, end))) =
                    (self.project.as_mut(), self.timeline.selection())
                    && let Err(error) = project.protect_range(start.min(end), start.max(end))
                {
                    self.messages.push(error.to_string());
                }
                self.start_preview();
            }
            Message::Export => self.export_current(),
            Message::Timeline(event) => match event {
                TimelineEvent::Seek(sample)
                | TimelineEvent::Drag(sample)
                | TimelineEvent::DragEnd(sample) => {
                    self.timeline.seek(sample);
                    if let Some(range) = &mut self.audition_range {
                        range.0 = sample.min(range.1);
                    }
                    if let Some(audition) = &self.audition {
                        audition.seek(sample);
                    }
                    if let Some(review) = &mut self.review {
                        review.seek(sample);
                    }
                }
                TimelineEvent::Zoom {
                    cursor_x,
                    width,
                    direction,
                } => self.timeline.zoom_at(cursor_x, width, direction),
                TimelineEvent::Pan { delta_x, width } => {
                    self.timeline.pan_by_pixels(delta_x, width)
                }
                TimelineEvent::Select(start, end) => {
                    if self.calibrating {
                        self.capture_calibration_range(start, end);
                    }
                    self.timeline
                        .set_selection(Some((start.min(end), start.max(end))));
                    if let Some(review) = &mut self.review {
                        review.set_selection(start, end);
                    }
                }
            },
        }
        if previous_parameters != self.preview_parameters {
            self.queue_preview(Duration::from_millis(40));
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let drops = iced::event::listen_with(|event, status, _window| match event {
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if status == iced::event::Status::Ignored =>
            {
                Some(Message::KeyPressed(key))
            }
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::RangeModifier(modifiers.shift()))
            }
            _ => None,
        });
        let processing = if self.processing.is_some()
            || self.sources_busy()
            || self.preview_processing.is_some()
            || self.preview_pending.is_some()
            || self.audition.is_some()
            || self.audition_preparation.is_busy()
        {
            let interval = if self.preview_processing.is_some()
                || self.preview_pending.is_some()
                || self.sources_busy()
            {
                10
            } else {
                100
            };
            iced::time::every(Duration::from_millis(interval)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        };
        Subscription::batch([drops, processing])
    }

    fn view(&self) -> Element<'_, Message> {
        responsive(|size| self.workspace(compact_workspace(size.width), size.height)).into()
    }

    fn calibration_controls(&self) -> Element<'_, Message> {
        let Some(project) = &self.project else {
            return column![].into();
        };
        let settings = project.ownership_settings();
        let mut controls = column![
            text("Microphone calibration").size(18),
            text(if settings.enabled { "Microphone comparison is on" } else { "Legacy detection: examples are disabled" }),
            button(if settings.enabled { "Use legacy detection" } else { "Enable microphone comparison" }).on_press(Message::OwnershipMode(!settings.enabled)),
            text("Optional examples help distinguish direct speech from its quieter copies on other microphones.").size(12),
            button(if self.calibrating { "Finish selecting" } else { "1. Select an example" }).on_press_maybe(settings.enabled.then_some(Message::CalibrationMode(!self.calibrating))),
            text("Drag across a waveform. The highlighted time range applies to all tracks.").size(12),
            text("2. Save the example under the intended track").size(14),
            text("Room tone: nobody is speaking. Solo speech: only this track's participant is speaking; their voice may bleed into other tracks.").size(12),
            text("These examples guide detection. They do not mute or protect the selected audio.").size(12),
        ].spacing(8);
        controls = controls.push(text(if !settings.enabled {
            "Enable microphone comparison above to save examples.".into()
        } else {
            match self.calibration_range() {
                Ok((start, end)) => format!(
                    "Selected {:.3}–{:.3} s ({:.3} s). Use a Save button below the track.",
                    start as f64 / 48_000.0,
                    end as f64 / 48_000.0,
                    (end - start) as f64 / 48_000.0
                ),
                Err(error) => error,
            }
        }));
        controls = controls.push(
            button(if self.calibration_timing {
                "Hide exact times"
            } else {
                "Exact times / existing selection"
            })
            .on_press(Message::CalibrationTiming(!self.calibration_timing)),
        );
        if self.calibration_timing {
            controls = controls
                .push(text("Start in seconds").size(12))
                .push(text_input("Start seconds", &self.calibration_start).on_input(Message::CalibrationStart))
                .push(button("Start at playhead").on_press(Message::CalibrationStartAtPlayhead))
                .push(text("End in seconds").size(12))
                .push(text_input("End seconds", &self.calibration_end).on_input(Message::CalibrationEnd))
                .push(button("End at playhead").on_press(Message::CalibrationEndAtPlayhead))
                .push(button("Copy highlighted timeline range").on_press_maybe(self.timeline.selection().map(|_| Message::CalibrationFromSelection)))
                .push(text("Copies the current highlight into these times. Start/end at playhead copies the white cursor's position.").size(12));
        }
        if let Some(notice) = project.calibration_notice() {
            controls = controls.push(text(notice));
        }
        if let Some(error) = &self.calibration_error {
            controls = controls.push(text(format!("Example not saved: {error}")));
        }
        if settings.examples.is_empty() {
            controls = controls.push(text(
                "No saved examples. With microphone comparison on, automatic estimates are used until you add examples.",
            ));
        }
        for (index, example) in settings.examples.iter().enumerate() {
            let kind = match example.kind {
                CalibrationKind::RoomTone => "Room tone",
                CalibrationKind::SoloSpeech => "Solo speech",
            };
            controls = controls
                .push(text(format!(
                    "{kind}, track {}: {:.5} to {:.5} s",
                    example.track + 1,
                    example.start as f64 / 48_000.0,
                    example.end as f64 / 48_000.0
                )))
                .push(button("Remove example").on_press(Message::RemoveCalibration(index)));
        }
        controls.into()
    }

    fn workspace(&self, compact: bool, height: f32) -> Element<'_, Message> {
        let current = self
            .project
            .as_ref()
            .map_or(WorkflowStep::Import, Project::step);
        let header = row![
            column![
                text("CLEANSLATE").size(24),
                text("UNTITLED SESSION").size(12)
            ]
            .spacing(2),
            Space::new().width(Fill),
            text(if self.sources_busy() {
                "LOADING TRACKS"
            } else {
                match current {
                    WorkflowStep::Import => "READY TO IMPORT",
                    WorkflowStep::Validate => "READY TO VALIDATE",
                    WorkflowStep::Process => "READY TO ANALYZE",
                    WorkflowStep::Review => "ANALYSIS COMPLETE",
                    WorkflowStep::Export => "EXPORT COMPLETE",
                }
            })
            .size(12),
            button("Import stems").on_press(Message::ImportStems),
            button("Open session").on_press_maybe(
                (!self.sources_busy() && self.processing.is_none()).then_some(Message::OpenProject)
            ),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let mut timeline_content = column![].spacing(10).height(Fill);
        if self.sources_busy() {
            let paths = self.loading_paths();
            let track_height = waveform_height(height, paths.len().max(1), compact);
            timeline_content = timeline_content.push(text(self.source_loading.as_ref().map_or(
                "Tracks queued. Loading starts in the background…",
                |loading| loading.stage,
            )));
            // Pending rows deliberately show a loading label, never a flat waveform.
            for (index, path) in paths.iter().enumerate() {
                let waveform: Element<'_, Message> = self
                    .source_loading
                    .as_ref()
                    .and_then(|loading| loading.waveforms.get(index).map(|cache| (loading, cache)))
                    .map_or_else(
                        || {
                            container(text("Waveform loading…"))
                                .height(track_height)
                                .center_y(Fill)
                                .into()
                        },
                        |(loading, cache)| {
                            canvas(TimelineCanvas::waveform(
                                cache,
                                &loading.timeline,
                                Message::Timeline,
                                None,
                                false,
                            ))
                            .width(Fill)
                            .height(track_height)
                            .into()
                        },
                    );
                timeline_content = timeline_content.push(
                    container(
                        column![
                            text(format!("Track {}: {}", index + 1, path.display())),
                            waveform,
                        ]
                        .spacing(8),
                    )
                    .padding(12)
                    .width(Fill)
                    .style(container::bordered_box),
                );
            }
        } else if self.project.is_none() {
            timeline_content = timeline_content.push(
                container(
                    column![
                        text("NO STEMS IMPORTED").size(22),
                        text("Import synchronized mono WAV stems or drop them anywhere in this window."),
                        button("Import WAV stems").on_press(Message::ImportStems),
                    ]
                    .spacing(12)
                    .align_x(iced::Center),
                )
                .center_x(Fill)
                .center_y(Fill)
                .height(Fill),
            );
        } else if let Some(project) = &self.project {
            let waveform_height = waveform_height(height, project.sources.len(), compact);
            timeline_content = timeline_content.push(
                container(
                    canvas(TimelineCanvas::ruler(
                        &self.timeline,
                        self.selecting_range || self.calibrating,
                        Message::Timeline,
                    ))
                    .width(Fill)
                    .height(28),
                )
                .padding([0, 12]),
            );
            for (index, source) in project.sources.iter().enumerate() {
                let mut header = row![text(format!(
                    "Track {}: {}",
                    index + 1,
                    source.path.display()
                ))];
                if self
                    .review
                    .as_ref()
                    .is_some_and(|review| review.highlighted_tracks().contains(&index))
                {
                    header = header.push(text("REVIEW TARGET"));
                }
                if !source.available {
                    header = header.push(button("Relink source").on_press(Message::Relink(index)));
                }
                if matches!(current, WorkflowStep::Review | WorkflowStep::Export) {
                    header = header.push(
                        button("Protect entire track").on_press(Message::ProtectTrack(index)),
                    );
                }
                let waveform: Element<'_, Message> = self.waveforms.get(index).map_or_else(
                    || {
                        container(text("Waveform loading…"))
                            .height(waveform_height)
                            .center_y(Fill)
                            .into()
                    },
                    |cache| {
                        canvas(TimelineCanvas::waveform(
                            cache,
                            &self.timeline,
                            Message::Timeline,
                            self.preview.as_ref().map(|preview| (preview, index)),
                            self.selecting_range || self.calibrating,
                        ))
                        .width(Fill)
                        .height(waveform_height)
                        .into()
                    },
                );
                timeline_content = timeline_content.push(
                    container(
                        column![
                            header,
                            waveform,
                            row![
                                button("Save room tone").on_press_maybe(
                                    (project.ownership_settings().enabled
                                        && self.calibration_range().is_ok())
                                    .then_some(Message::SaveCalibration(
                                        CalibrationKind::RoomTone,
                                        index
                                    ))
                                ),
                                button("Save solo speech").on_press_maybe(
                                    (project.ownership_settings().enabled
                                        && self.calibration_range().is_ok())
                                    .then_some(Message::SaveCalibration(
                                        CalibrationKind::SoloSpeech,
                                        index
                                    ))
                                ),
                            ]
                            .spacing(8)
                        ]
                        .spacing(8),
                    )
                    .padding(12)
                    .width(Fill)
                    .style(container::bordered_box),
                );
            }
            match current {
                WorkflowStep::Import => {}
                WorkflowStep::Validate => {
                    timeline_content = timeline_content
                        .push(button("Validate session").on_press(Message::Validate));
                }
                WorkflowStep::Process => {
                    timeline_content = if let Some(processing) = &self.processing {
                        timeline_content
                            .push(text(format!(
                                "Final render: {}%",
                                processing.progress.load(Ordering::Relaxed)
                            )))
                            .push(button("Cancel final render").on_press(Message::Cancel))
                    } else {
                        timeline_content
                            .push(button("Create final WAV render").on_press(Message::Process))
                    };
                }
                WorkflowStep::Review => {
                    let stale = matches!(project.render(), RenderState::Stale);
                    if let Some(processing) = &self.processing {
                        timeline_content = timeline_content
                            .push(text(format!(
                                "Final render: {}%",
                                processing.progress.load(Ordering::Relaxed)
                            )))
                            .push(button("Cancel final render").on_press(Message::Cancel));
                    } else if stale {
                        timeline_content = timeline_content
                            .push(text("Final render stale · live audition remains available"))
                            .push(
                                button("Create current final WAV render")
                                    .on_press(Message::Process),
                            );
                    } else if let Some(error) = &self.export_error {
                        timeline_content = timeline_content.push(text(format!(
                            "Final render current · Export blocked: {error}"
                        )));
                    } else {
                        timeline_content =
                            timeline_content.push(text("Final render current · WAV export ready"));
                    }
                    if let Some(review) = &self.review {
                        timeline_content = timeline_content.push(
                            row![
                                button("Original").on_press(Message::Original),
                                button("Processed").on_press(Message::Processed),
                                button("Removed").on_press(Message::Removed),
                                button("Audition selection").on_press(Message::AuditionSelection),
                            ]
                            .spacing(8),
                        );
                        for (index, warning) in review.warnings.iter().enumerate() {
                            let reviewed = project.review_states().get(&warning.key())
                                == Some(&ReviewState::Reviewed);
                            let tracks = warning
                                .affected_tracks
                                .iter()
                                .map(|track| (track + 1).to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            timeline_content = timeline_content.push(
                                button(text(format!(
                                    "{}{} · tracks {} · {:.2}s–{:.2}s",
                                    if reviewed { "✓ " } else { "" },
                                    warning.label,
                                    tracks,
                                    warning.start as f64 / 48_000.0,
                                    warning.end as f64 / 48_000.0,
                                )))
                                .on_press(Message::SelectWarning(index)),
                            );
                        }
                        if review.selected_region().is_some() {
                            let reviewed = review.selected_key().is_some_and(|key| {
                                project.review_states().get(&key) == Some(&ReviewState::Reviewed)
                            });
                            let actions = row![
                                button(if reviewed {
                                    "Mark unreviewed"
                                } else {
                                    "Mark reviewed"
                                })
                                .on_press(Message::ToggleReviewed),
                                button("Protect this region").on_press(Message::ProtectRegion),
                            ]
                            .spacing(8);
                            timeline_content = timeline_content.push(actions);
                        }
                        if self.can_protect_selection() {
                            timeline_content = timeline_content.push(
                                button("Protect selected range across all tracks")
                                    .on_press(Message::ProtectSelection),
                            );
                        }
                    }
                    if self.export_ready() {
                        timeline_content = timeline_content
                            .push(button("Export current render").on_press(Message::Export));
                    }
                }
                WorkflowStep::Export => {
                    timeline_content = timeline_content.push(text("Export complete."));
                }
            }
        }
        for message in &self.messages {
            timeline_content = timeline_content.push(text(message));
        }

        let timeline = column![
            text("SESSION TIMELINE").size(12),
            text(if self.calibrating { "Calibration selection: drag a waveform, then save an example on the intended track. No protection is added." } else { "Shift-drag selects a range. Use Protect selected range to protect it, or enable calibration selection to save examples." }).size(11),
            scrollable(timeline_content).height(Fill),
        ]
        .spacing(10)
        .width(Fill)
        .height(Fill);
        let preview_state: Element<'_, Message> = if let Some(processing) = &self.preview_processing
        {
            let progress = processing.progress.load(Ordering::Relaxed) as f32;
            if self.preview_cache.stats().preparations > 0 && !(20.0..70.0).contains(&progress) {
                text("Updating preview · showing previous completed preview").into()
            } else {
                column![
                    text(format!("{}: {progress:.0}%", preview_stage(progress as u8))),
                    progress_bar(0.0..=100.0, progress),
                    text(if self.preview.is_some() {
                        "Showing previous completed preview"
                    } else {
                        "No completed preview yet"
                    }),
                ]
                .spacing(6)
                .into()
            }
        } else if let Some(error) = &self.preview_error {
            text(format!("Preview failed: {error}")).into()
        } else if let Some(preview) = &self.preview {
            text(format!(
                "Preview ready · {} ownership regions",
                preview.regions.len()
            ))
            .into()
        } else {
            text("Preview not analyzed").into()
        };
        let parameters = &self.preview_parameters;
        let inspector = container(
            column![
                text("ANALYSIS PREVIEW").size(12),
                text("Session parameters").size(20),
                text("Preview and audition use these settings. Render again to update exported audio."),
                preview_state,
                self.calibration_controls(),
                text(format!(
                    "Speech sensitivity: {:.2}",
                    1.0 - parameters.speech_sensitivity
                )),
                slider(
                    0.1..=0.9,
                    1.0 - parameters.speech_sensitivity,
                    Message::SpeechSensitivity
                )
                .step(0.01_f32)
                .on_release(Message::RefreshPreview),
                text("Higher sensitivity admits weaker speech evidence.").size(12),
                text(format!(
                    "Bleed discrimination: {:.1} dB",
                    parameters.bleed_discrimination_db
                )),
                slider(
                    0.0..=18.0,
                    parameters.bleed_discrimination_db,
                    Message::BleedDiscrimination
                )
                .step(0.5_f32)
                .on_release(Message::RefreshPreview),
                text("Required margin above each track's own noise floor.").size(12),
                text(format!("Attenuation: {:.1} dB", parameters.attenuation_db)),
                slider(
                    -30.0..=-3.0,
                    parameters.attenuation_db,
                    Message::Attenuation
                )
                .step(1.0_f32)
                .on_release(Message::RefreshPreview),
                text(format!(
                    "Speech padding: {:.0} ms effective",
                    effective_padding_ms(parameters.speech_padding_ms)
                )),
                slider(
                    0.0..=992.0,
                    effective_padding_ms(parameters.speech_padding_ms),
                    Message::SpeechPadding
                )
                .step(32.0_f32)
                .on_release(Message::RefreshPreview),
                text("32 ms steps. Legacy stored values stay unchanged until padding is edited.").size(12),
                text(format!(
                    "Open time: {:.0} ms",
                    parameters.open_ms
                )),
                slider(1.0..=1000.0, parameters.open_ms, Message::OpenTime)
                    .step(1.0_f32)
                    .on_release(Message::RefreshPreview),
                text(format!(
                    "Close time: {:.0} ms",
                    parameters.close_ms
                )),
                slider(1.0..=2000.0, parameters.close_ms, Message::CloseTime)
                    .step(1.0_f32)
                    .on_release(Message::RefreshPreview),
                text(
                    "Overlay: preserved solid · attenuated diagonal hatch · uncertain dotted · protected outline"
                ),
                text("Bright waveform: conservative processed bounds; faint waveform: Original.").size(12),
            ]
            .spacing(12),
        )
        .padding(18)
        .width(if compact { Fill } else { Length::Fixed(280.0) })
        .style(container::bordered_box);
        let inspector: Element<'_, Message> = if self.sources_busy() {
            container(column![text("Loading tracks").size(20),
                text("Waveforms appear as each track is read. Validation and source checks run in the background, followed by speech analysis."),
                text("You can scroll the tracks and drop more WAV files. Editing becomes available when the import is validated."),
            ].spacing(12)).padding(18).width(if compact { Fill } else { Length::Fixed(280.0) }).into()
        } else {
            scrollable(inspector)
                .height(if compact { Length::Fixed(240.0) } else { Fill })
                .into()
        };
        let transport = row![
            text(format!(
                "{:02}:{:06.3}",
                self.timeline.playhead() / 48_000 / 60,
                (self.timeline.playhead() % (48_000 * 60)) as f64 / 48_000.0
            )),
            Space::new().width(Fill),
            button("Previous").on_press(Message::Previous),
            button(if self.audition_range.is_some() {
                "Preparing audio…"
            } else if self.audition.is_some() {
                "Playing"
            } else {
                "Play"
            })
            .on_press_maybe(self.preview.is_some().then_some(Message::Audition)),
            button("Stop").on_press_maybe(
                (self.audition.is_some() || self.audition_range.is_some()).then_some(Message::Stop)
            ),
            Space::new().width(Fill),
            text("AUDITION"),
            button(if self.audition_mode == AuditionMode::Original {
                "Original ✓"
            } else {
                "Original"
            })
            .on_press(Message::Original),
            button(if self.audition_mode == AuditionMode::Processed {
                "Processed ✓"
            } else {
                "Processed"
            })
            .on_press(Message::Processed),
            button(if self.audition_mode == AuditionMode::Removed {
                "Removed ✓"
            } else {
                "Removed"
            })
            .on_press(Message::Removed),
            text(format!("Monitor {:+.1} dB", self.monitor_db)),
            slider(-24.0..=6.0, self.monitor_db, Message::MonitorGain)
                .step(1.0_f32)
                .width(120),
        ]
        .spacing(8)
        .align_y(iced::Center);
        let editor: Element<'_, Message> = if compact {
            column![timeline, inspector].spacing(12).into()
        } else {
            row![timeline, inspector].spacing(12).into()
        };

        container(column![header, editor, transport].spacing(12))
            .padding(12)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
