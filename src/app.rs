#[cfg(test)]
use crate::CalibrationKind;
use crate::{
    AnalysisPreview, AutomixParameters, Comparison, Project, RenderState, ReviewState,
    ReviewWorkspace, SpeakerId, TimeRange,
    audition::{AuditionMode, AuditionPreparation, LiveAudition},
    prepared_analysis::source_stamps,
    timebase::{Rounding, TICKS_PER_SECOND, TimePosition, time_from_seconds},
    timeline::{TimelineEvent, TimelineViewport},
    validate, validate_export,
    waveform::{self, TimelineCanvas, WaveformCache},
};
use iced::{
    Element, Fill, Length, Subscription, Task, Theme, keyboard,
    widget::{
        Space, button, canvas, column, container, mouse_area, opaque, progress_bar, responsive,
        row, scrollable, slider, stack, text, text_input,
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
    time::{Duration, Instant},
};

// Match the detector's rounding of 1536-sample frames at 48 kHz.
#[cfg(test)]
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
    fn installing_validated_tracks_prepares_original_without_starting_playback() {
        let root =
            std::env::temp_dir().join(format!("cleanslate-auto-playback-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("voice.wav");
        write_wav(&source, &vec![0.0; 48000]);
        let waveforms = crate::waveform::load_or_build(
            &root.join("session.cleanslate"),
            std::slice::from_ref(&source),
        )
        .unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(root.join("session.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        app.install_waveforms(waveforms);
        assert!(app.audition_preparation.is_busy());
        assert!(app.audition_range.is_none() && app.audition.is_none());
        let _ = app.update(super::Message::Stop);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while app.audition_preparation.is_busy() && std::time::Instant::now() < deadline {
            app.poll_audition();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(!app.audition_preparation.is_busy());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn track_context_labels_validate_replace_undo_and_persist_without_cross_track_selection() {
        use super::Message;
        use crate::timeline::TimelineEvent;
        let root =
            std::env::temp_dir().join(format!("cleanslate-context-menu-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let sources: Vec<_> = (0..3)
            .map(|index| {
                let path = root.join(format!("speaker-{index}.wav"));
                write_wav(&path, &vec![0.0; 48_000]);
                path
            })
            .collect();
        let path = root.join("session.cleanslate");
        let mut app = super::CleanSlateApp {
            project: Some(synchronized_project(&path, sources).unwrap()),
            ..Default::default()
        };
        let speakers: Vec<_> = app
            .project
            .as_ref()
            .unwrap()
            .sources
            .iter()
            .map(|s| s.speaker_id)
            .collect();
        let start = crate::TICKS_PER_SECOND / 4;
        let end = crate::TICKS_PER_SECOND * 3 / 4;
        let point = iced::Point::new(400.0, 300.0);
        app.timeline.set_total_samples(crate::TICKS_PER_SECOND);
        let _ = app.update(Message::TrackTimeline(
            0,
            TimelineEvent::Select(start, start),
        ));
        assert_eq!(app.timeline.playhead(), start);
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        assert!(
            app.reference_menu.is_none(),
            "A click is not a usable range"
        );
        let _ = app.update(Message::TrackTimeline(0, TimelineEvent::Select(end, start)));
        let _ = app.update(Message::OpenReferenceMenu(1, point));
        assert!(
            app.reference_menu.is_none(),
            "Another track's selection cannot label this track"
        );
        app.calibration_timing = true;
        let _ = app.update(Message::CompleteTrackSelection(0, start, end, point));
        assert_eq!(app.reference_menu, Some((0, point)));
        assert!(
            !app.calibration_timing,
            "Popup removes the exact-times input focus"
        );
        let captured_start = app.calibration_start.clone();
        let _ = app.update(Message::CalibrationStart("999".into()));
        assert_eq!(app.calibration_start, captured_start);
        assert_eq!(app.reference_speakers, vec![speakers[0]]);
        let _ = app.update(Message::Timeline(TimelineEvent::Seek(0)));
        let _ = app.update(Message::TrackTimeline(1, TimelineEvent::Select(0, end)));
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Home),
            iced::keyboard::Modifiers::empty(),
        ));
        assert_eq!(app.timeline.playhead(), start);
        assert_eq!(app.timeline.selection(), Some((start, end)));
        assert_eq!(app.selection_track, Some(0));
        let _ = app.update(Message::ContextReference(vec![speakers[1]], false));
        assert!(app.reference_menu.is_none());
        assert!(app.reference_stale && app.preview_pending.is_none());
        assert_eq!(
            app.project.as_ref().unwrap().reference_labels()[0].active_speakers,
            vec![speakers[1]]
        );
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        let _ = app.update(Message::ContextReference(vec![], false));
        assert!(app.calibration_error.is_some() && app.reference_menu.is_some());
        assert_eq!(app.project.as_ref().unwrap().reference_labels().len(), 1);
        let _ = app.update(Message::ContextReference(vec![], true));
        assert!(
            app.project.as_ref().unwrap().reference_labels()[0]
                .active_speakers
                .is_empty()
        );
        let _ = app.update(Message::UndoReferences);
        assert_eq!(
            app.project.as_ref().unwrap().reference_labels()[0].active_speakers,
            vec![speakers[1]]
        );
        let _ = app.update(Message::RedoReferences);
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        let _ = app.update(Message::ContextReference(
            vec![speakers[0], speakers[2]],
            true,
        ));
        assert_eq!(
            crate::Project::open(&path).unwrap().reference_labels()[0].active_speakers,
            vec![speakers[0], speakers[2]]
        );
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::empty(),
        ));
        assert!(app.reference_menu.is_none());
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        let _ = app.update(Message::DismissReferenceMenu);
        assert!(app.reference_menu.is_none());
        let _ = app.update(Message::Timeline(TimelineEvent::Select(start, end)));
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        assert!(
            app.reference_menu.is_none(),
            "Inspector selection has no track binding"
        );
        let _ = app.update(Message::TrackTimeline(0, TimelineEvent::Select(start, end)));
        let mut review = crate::ReviewWorkspace::default();
        review.warnings.push(
            serde_json::from_value(serde_json::json!({
                "start": 0,
                "end": start,
                "label": "Quiet speech",
                "affected_tracks": [1],
                "region": 0,
                "analysis_identity": null
            }))
            .unwrap(),
        );
        app.review = Some(review);
        let _ = app.update(Message::SelectSupervisedWarning(0));
        assert_eq!(app.timeline.selection(), Some((0, start)));
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        assert!(
            app.reference_menu.is_none(),
            "Review navigation cannot reuse an old track binding"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imported_tracks_report_required_model_and_reference_setup() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-ui-status-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let source = directory.join("voice.wav");
        write_wav(&source, &vec![0.0; 48000]);
        let app = super::CleanSlateApp {
            project: Some(
                synchronized_project(directory.join("session.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        assert_eq!(app.workspace_status(), "VERIFY MODEL · ADD REFERENCES");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn fresh_original_playback_and_transient_gain_feedback_do_not_require_analysis() {
        let root =
            std::env::temp_dir().join(format!("cleanslate-fresh-editor-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("source.wav");
        write_wav(&source, &vec![0.0; 48000]);
        let project = synchronized_project(root.join("session.cleanslate"), vec![source]).unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        app.timeline.set_total_samples(crate::TICKS_PER_SECOND);
        let _ = app.update(super::Message::Audition);
        assert!(app.audition_range.is_some());
        assert!(app.audition_preparation.is_busy());
        assert!(app.preview_processing.is_none());
        assert!(app.preview.is_none());
        let _ = app.update(super::Message::Stop);
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Select(
                crate::TICKS_PER_SECOND / 4,
                crate::TICKS_PER_SECOND * 3 / 4,
            ),
        ));
        let _ = app.update(super::Message::GainAction(0, false));
        let before = app.project.as_ref().unwrap().gain_constraints();
        let id = before.manual_edits[0].id;
        let _ = app.update(super::Message::GainTool(true));
        let _ = app.update(super::Message::GainEvent(
            0,
            crate::GainEditorEvent::Begin(id, 0),
        ));
        let _ = app.update(super::Message::GainEvent(
            0,
            crate::GainEditorEvent::Move(crate::TICKS_PER_SECOND / 8, 0.7),
        ));
        assert_eq!(
            app.gain_drag
                .as_ref()
                .unwrap()
                .draft()
                .constraints
                .manual_edits[0]
                .points[0]
                .gain,
            0.7
        );
        assert_eq!(app.project.as_ref().unwrap().gain_constraints(), before);
        let _ = app.update(super::Message::GainEvent(0, crate::GainEditorEvent::Cancel));
        assert!(app.gain_drag.is_none());
        assert_eq!(app.gain_constraints, before);
        let arrow = iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp);
        assert!(
            super::keyboard_message(
                arrow.clone(),
                iced::keyboard::Modifiers::empty(),
                iced::event::Status::Captured
            )
            .is_none(),
            "Text input owns captured editing keys"
        );
        let key = super::keyboard_message(
            arrow,
            iced::keyboard::Modifiers::empty(),
            iced::event::Status::Ignored,
        )
        .unwrap();
        let _ = app.update(key);
        assert!(
            app.project.as_ref().unwrap().manual_gain_edits()[0].points[0].gain
                > before.manual_edits[0].points[0].gain
        );
        let _ = app.update(super::Message::UndoGain);
        assert_eq!(app.project.as_ref().unwrap().gain_constraints(), before);
        assert_eq!(app.supervised_worker.stats().probability_passes, 0);
        app.audition_preparation.cancel();
        std::fs::remove_dir_all(root).unwrap();
    }
    fn synchronized_project(
        path: impl Into<std::path::PathBuf>,
        sources: Vec<std::path::PathBuf>,
    ) -> std::io::Result<crate::Project> {
        let mut project = crate::Project::create(path, sources)?;
        project.set_supplied_alignment(true)?;
        Ok(project)
    }
    #[test]
    fn explicit_reference_analyze_can_start_without_a_completed_preview() {
        let directory = std::env::temp_dir().join(format!(
            "cleanslate-first-label-preview-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("speaker.wav");
        write_wav(&source, &vec![0.0; 48_000]);
        let mut app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("session.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        app.project
            .as_mut()
            .unwrap()
            .set_supplied_alignment(true)
            .unwrap();
        app.mark_reference_edit();
        app.start_preview();
        assert!(app.preview_processing.is_none() && app.preview_pending.is_none());
        let _ = app.update(super::Message::AnalyzeReferences);
        assert!(app.preview_processing.is_some() || app.preview_pending.is_some());
        assert!(
            app.reference_stale,
            "An Analyze request must not declare success before completion"
        );
        if let Some(job) = app.preview_processing.take() {
            job.cancelled.store(true, super::Ordering::Relaxed);
        }
        let project = app.project.as_mut().unwrap();
        project
            .add_reference_label(
                crate::TimeRange::new(0, crate::TICKS_PER_SECOND / 4).unwrap(),
                vec![],
                &[],
            )
            .unwrap();
        let project_path = project.path().to_path_buf();
        app.install_waveforms(vec![]);
        assert!(
            app.reference_stale,
            "Source installation must preserve authoritative label staleness"
        );
        app.project = Some(crate::Project::open(project_path).unwrap());
        app.reference_stale = false; // Simulate a freshly bootstrapped controller.
        let _ = app.update(super::Message::Process);
        assert!(
            app.processing.is_none(),
            "Reopened labels cannot enable retained-detector final rendering"
        );
        assert!(
            app.messages
                .iter()
                .any(|message| message.contains("current supervised"))
        );
        // The worker owns these temporary inputs until it exits.
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
                synchronized_project(directory.join("episode.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        app.timeline
            .set_total_samples(crate::timebase::detector_time(48_000));
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
            crate::timeline::TimelineEvent::Select(
                crate::timebase::detector_time(100),
                crate::timebase::detector_time(30_000),
            ),
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
    fn calibration_drag_saves_outward_examples_without_protection_and_refreshes_preview() {
        let directory =
            std::env::temp_dir().join(format!("calibration-editor-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("voice.wav");
        write_wav(&source, &vec![0.0; 48_000]);
        let mut app = super::CleanSlateApp {
            project: Some(
                synchronized_project(directory.join("episode.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        app.timeline
            .set_total_samples(crate::timebase::detector_time(48_000));
        let _ = app.update(super::Message::CalibrationMode(true));
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Select(
                crate::timebase::detector_time(24_001),
                crate::timebase::detector_time(1),
            ),
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
                start: 587,
                end: crate::timebase::detector_time(24_001),
            }]
        );
        assert!(app.project.as_ref().unwrap().protections().is_empty());
        assert!(app.reference_stale);
        assert!(app.preview_pending.is_none());
        let _ = app.update(super::Message::SaveCalibration(
            crate::CalibrationKind::SoloSpeech,
            0,
        ));
        assert!(
            app.calibration_error.is_some(),
            "contradictory truth must be rejected"
        );
        assert_eq!(
            app.project
                .as_ref()
                .unwrap()
                .ownership_settings()
                .examples
                .len(),
            1
        );
        let _ = app.update(super::Message::RemoveCalibration(0));
        assert!(
            app.project
                .as_ref()
                .unwrap()
                .ownership_settings()
                .examples
                .is_empty()
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
    fn audition_stop_relink_and_error_do_not_start_output() {
        let directory =
            std::env::temp_dir().join(format!("audition-editor-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("voice.wav");
        write_wav(&path, &[0.0; 1537]);
        let mut app = super::CleanSlateApp {
            project: Some(
                synchronized_project(directory.join("episode.cleanslate"), vec![path.clone()])
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
        app.timeline
            .set_total_samples(crate::timebase::detector_time(1537));
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
            snapshot: crate::EditorAnalysisIdentity {
                key: String::new(),
                project: Default::default(),
            },
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
                synchronized_project(directory.join("benchmark.cleanslate"), vec![]).unwrap(),
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
        app.timeline
            .set_total_samples(crate::timebase::detector_time(48_000));
        app.capture_calibration_range(
            crate::timebase::detector_time(24_000),
            crate::timebase::detector_time(24_000),
        );
        assert!(
            app.calibration_range()
                .unwrap_err()
                .contains("end must be after")
        );
        app.capture_calibration_range(
            crate::timebase::detector_time(24_000),
            crate::timebase::detector_time(24_001),
        );
        assert!(
            app.calibration_range()
                .unwrap_err()
                .contains("0.25 seconds")
        );
        app.capture_calibration_range(
            crate::timebase::detector_time(48_000),
            crate::timebase::detector_time(12_000),
        );
        assert_eq!(
            app.calibration_range().unwrap(),
            (
                crate::timebase::detector_time(12_000),
                crate::timebase::detector_time(48_000)
            )
        );
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
        synchronized_project(&saved, vec![old_a.clone(), old_b.clone()]).unwrap();
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
        write_wav(&source, &[0.0; 100]);
        let mut app = super::CleanSlateApp {
            project: Some(
                synchronized_project(directory.join("episode.cleanslate"), vec![source.clone()])
                    .unwrap(),
            ),
            ..Default::default()
        };
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Select(40, 10),
        ));
        assert!(app.can_protect_selection());
        let _ = app.update(super::Message::ProtectSelection);
        assert_eq!(
            app.project.as_ref().unwrap().protections(),
            &[crate::Protection {
                track: None,
                start: 10,
                end: 40
            }]
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
    fn current_render_export_readiness_tracks_parameter_staleness() {
        let directory = std::env::temp_dir().join(format!(
            "cleanslate-export-state-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("voice.wav");
        let rendered = directory.join("render/01-voice.wav");
        let report = directory.join("render/report.json");
        write_wav(&source, &[0.0; 100]);
        std::fs::create_dir_all(report.parent().unwrap()).unwrap();
        let writer =
            bwavfile::WaveWriter::create(&rendered, bwavfile::WaveFmt::new_pcm_mono(48_000, 16))
                .unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(&[0.0_f32, 0.25]).unwrap();
        frames.end().unwrap();
        std::fs::write(&report, br#"{"output":{"stems":["01-voice.wav"]}}"#).unwrap();
        let mut project =
            synchronized_project(directory.join("episode.cleanslate"), vec![source.clone()])
                .unwrap();
        project.mark_render_current(&report).unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        app.refresh_export_readiness();
        assert!(!app.export_ready());
        let delivery = directory.join("delivery");
        app.export_to(delivery.clone());
        assert!(!delivery.exists());
        assert!(!delivery.join("report.json").exists());
        assert!(app.messages[0].starts_with("Export blocked"));
        let _ = app.update(super::Message::Attenuation(-12.0));
        assert!(!app.export_ready());
        let blocked = directory.join("during-drag");
        app.export_to(blocked.clone());
        assert!(!blocked.exists());
        assert!(matches!(
            app.project.as_ref().unwrap().render(),
            crate::RenderState::Stale
        ));
        app.preview_parameters = app.project.as_ref().unwrap().parameters();
        std::fs::remove_file(rendered).unwrap();
        app.refresh_export_readiness();
        assert!(!app.export_ready());
        std::fs::remove_file(source).unwrap();
        app.preview_parameters.attenuation_db = -12.0;
        let _ = app.update(super::Message::RefreshPreview);
        assert!(!app.export_ready());
        assert_eq!(app.project.unwrap().render(), &crate::RenderState::Stale);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

pub struct CleanSlateApp {
    model_acquisition: crate::model::ModelAcquisition,
    project: Option<Project>,
    pending_imports: Vec<PathBuf>,
    import_due: Option<Instant>,
    source_loading: Option<SourceLoading>,
    messages: Vec<String>,
    processing: Option<Processing>,
    exporting: Option<ExportProcessing>,
    render_generation: Arc<crate::RenderGeneration>,
    review: Option<ReviewWorkspace>,
    audition: Option<LiveAudition>,
    audition_preparation: AuditionPreparation,
    audition_ready: bool,
    audition_range: Option<(TimePosition, TimePosition)>,
    audition_mode: AuditionMode,
    monitor_db: f32,
    selecting_range: bool,
    calibrating: bool,
    calibration_timing: bool,
    calibration_start: String,
    calibration_end: String,
    calibration_error: Option<String>,
    reference_speakers: Vec<SpeakerId>,
    selection_track: Option<usize>,
    reference_menu: Option<(usize, iced::Point)>,
    model_details: bool,
    reference_stale: bool,
    reference_analyze_requested: bool,
    waveforms: Vec<WaveformCache>,
    timeline: TimelineViewport,
    preview_parameters: AutomixParameters,
    preview: Option<AnalysisPreview>,
    preview_sources: Vec<waveform::SourceStamp>,
    supervised_worker: crate::EditorAnalysisWorker,
    supervised_result: Option<Arc<crate::EditorAnalysisResult>>,
    effective_cache: crate::EffectiveEnvelopeCache,
    effective_result: Option<Arc<crate::EffectiveEnvelopes>>,
    gain_tool: bool,
    gain_drag: Option<crate::GainPointDrag>,
    gain_constraints: crate::GainConstraints,
    selected_gain_point: Option<(usize, u64, usize)>,
    replace_gain_constraints: bool,
    selected_supervised_warning: Option<usize>,
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
            model_acquisition: crate::model::ModelAcquisition::default(),
            project: None,
            pending_imports: vec![],
            import_due: None,
            source_loading: None,
            messages: vec![],
            processing: None,
            exporting: None,
            render_generation: Arc::new(crate::RenderGeneration::default()),
            review: None,
            audition: None,
            audition_preparation: AuditionPreparation::default(),
            audition_ready: false,
            audition_range: None,
            audition_mode: AuditionMode::Original,
            monitor_db: -6.0,
            selecting_range: false,
            calibrating: false,
            calibration_timing: false,
            calibration_start: String::new(),
            calibration_end: String::new(),
            calibration_error: None,
            reference_speakers: vec![],
            selection_track: None,
            reference_menu: None,
            model_details: false,
            reference_stale: false,
            reference_analyze_requested: false,
            waveforms: vec![],
            timeline: TimelineViewport::default(),
            preview_parameters: AutomixParameters::default(),
            preview: None,
            preview_sources: vec![],
            supervised_worker: crate::EditorAnalysisWorker::default(),
            supervised_result: None,
            effective_cache: crate::EffectiveEnvelopeCache::default(),
            effective_result: None,
            gain_tool: false,
            gain_drag: None,
            gain_constraints: crate::GainConstraints {
                manual_edits: vec![],
                protections: vec![],
            },
            selected_gain_point: None,
            replace_gain_constraints: false,
            selected_supervised_warning: None,
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
    result: Receiver<Result<crate::PublishedSupervisedRender, String>>,
    snapshot: crate::SupervisedRenderSnapshot,
    generation: u64,
}
impl Drop for Processing {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}
struct ExportProcessing {
    cancelled: Arc<AtomicBool>,
    result: Receiver<Result<PathBuf, String>>,
}
impl Drop for ExportProcessing {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

struct PreviewProcessing {
    progress: Arc<AtomicU8>,
    cancelled: Arc<AtomicBool>,
    generation: u64,
    source_generation: u64,
    sources: Vec<waveform::SourceStamp>,
    snapshot: crate::EditorAnalysisIdentity,
    result: Receiver<(
        Result<Arc<crate::EditorAnalysisResult>, String>,
        crate::EditorAnalysisWorker,
    )>,
}

impl Drop for PreviewProcessing {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
enum Message {
    TrackTimeline(usize, TimelineEvent),
    CompleteTrackSelection(usize, TimePosition, TimePosition, iced::Point),
    OpenReferenceMenu(usize, iced::Point),
    DismissReferenceMenu,
    ContextReference(Vec<SpeakerId>, bool),
    ModelDetails(bool),
    DownloadModel,
    VerifyModel,
    CancelModelDownload,
    SuppliedAlignment(bool),
    ImportStems,
    OpenProject,
    FileDropped(PathBuf),
    Relink(usize),
    Validate,
    Process,
    Cancel,
    Tick,
    #[cfg(test)]
    SpeechSensitivity(f32),
    Attenuation(f32),
    #[cfg(test)]
    SpeechPadding(f32),
    OpenTime(f32),
    CloseTime(f32),
    RefreshPreview,
    Original,
    Processed,
    Removed,
    Audition,
    AuditionSelection,
    Stop,
    Previous,
    MonitorGain(f32),
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    RangeModifier(bool),
    #[cfg(test)]
    OwnershipMode(bool),
    #[cfg(test)]
    CalibrationMode(bool),
    CalibrationTiming(bool),
    CalibrationStart(String),
    CalibrationEnd(String),
    CalibrationStartAtPlayhead,
    CalibrationEndAtPlayhead,
    CalibrationFromSelection,
    #[cfg(test)]
    SaveCalibration(CalibrationKind, usize),
    #[cfg(test)]
    RemoveCalibration(usize),
    ReferenceSpeaker(SpeakerId, bool),
    SaveReference(bool),
    RemoveReference(u64),
    UndoReferences,
    RedoReferences,
    AnalyzeReferences,
    CancelAnalysis,
    GainTool(bool),
    GainEvent(usize, crate::GainEditorEvent),
    GainAction(usize, bool),
    ReplaceGainConstraints(bool),
    UndoGain,
    RedoGain,
    DeleteGain,
    InsertGainPoint,
    RebindSource(usize),
    SelectSupervisedWarning(usize),
    SaveProject,
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
        min_size: Some(iced::Size::new(960.0, 900.0)),
        ..window::Settings::default()
    })
    .run()
}

fn keyboard_message(
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
    status: iced::event::Status,
) -> Option<Message> {
    (status == iced::event::Status::Ignored).then_some(Message::KeyPressed(key, modifiers))
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
        self.reference_menu = None;
        self.selection_track = None;
        self.calibrating = false;
        self.calibration_start.clear();
        self.calibration_end.clear();
        self.calibration_error = None;
        self.reference_speakers.clear();
        self.reference_stale = self.project.as_ref().is_some_and(|project| {
            !project.reference_labels().is_empty()
                || matches!(project.result(), crate::ResultState::Stale { .. })
        });
        self.reference_analyze_requested = false;
        self.audition_preparation.cancel();
        self.audition_ready = false;
        self.audition_range = None;
        if let Some(audition) = self.audition.take() {
            audition.stop();
        }
        self.preview = None;
        self.preview_sources.clear();
        self.supervised_result = None;
        self.effective_result = None;
        self.effective_cache = crate::EffectiveEnvelopeCache::default();
        self.gain_drag = None;
        self.gain_constraints = self.project.as_ref().map_or(
            crate::GainConstraints {
                manual_edits: vec![],
                protections: vec![],
            },
            Project::gain_constraints,
        );
        self.source_generation += 1;
        self.preview_generation += 1;
        self.preview_pending = None;
        if let Some(processing) = &self.preview_processing {
            processing.cancelled.store(true, Ordering::Relaxed);
        }
        if let Some(cache) = waveforms.first() {
            self.timeline.set_total_samples(cache.duration());
        }
        self.waveforms = waveforms;
        if let Some(project) = &self.project {
            match crate::original_envelopes(project) {
                Ok(_) => self.audition_preparation.request(project.source_paths()),
                Err(error) => self.messages.push(format!("Playback unavailable: {error}")),
            }
        }
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
                path.extension().is_some_and(|ext| {
                    [
                        "wav", "bwf", "aif", "aiff", "flac", "mp3", "m4a", "mp4", "ogg",
                    ]
                    .iter()
                    .any(|format| ext.eq_ignore_ascii_case(format))
                })
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
            self.messages = vec!["No new supported mono tracks were selected".into()];
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
                            loading.timeline.set_total_samples(waveform.duration());
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
                            self.messages = loaded.warnings;
                            self.install_waveforms(loading.waveforms);
                            self.review = loaded.review;
                            self.export_error = loaded.export_error;
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
            .add_filter(
                "Mono audio",
                &[
                    "wav", "bwf", "aif", "aiff", "flac", "mp3", "m4a", "mp4", "ogg",
                ],
            )
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

    fn start_processing(&mut self) {
        if self.processing.is_some() {
            return;
        }
        let (Some(project), Some(analysis), Some(effective), Some(runtime)) = (
            &self.project,
            &self.supervised_result,
            &self.effective_result,
            self.model_acquisition.provenance(),
        ) else {
            self.messages = vec![
                "Final export requires current supervised analysis and a verified model.".into(),
            ];
            return;
        };
        if self.reference_stale
            || self.gain_drag.is_some()
            || self.preview_processing.is_some()
            || self.preview_pending.is_some()
        {
            self.messages = vec![
                "Finish editing and Analyze before rendering the current supervised preview."
                    .into(),
            ];
            return;
        }
        let snapshot =
            match crate::SupervisedRenderSnapshot::capture(project, analysis, effective, runtime) {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    self.messages = vec![error.to_string()];
                    return;
                }
            };
        let project = project.clone();
        let analysis = analysis.clone();
        let effective = effective.clone();
        let runtime = runtime.clone();
        let generation = self.render_generation.current();
        let gate = self.render_generation.clone();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let destination = project
            .path()
            .with_extension("renders")
            .join(format!("{}-{generation}-{nonce}", std::process::id()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU8::new(0));
        let (sender, result) = mpsc::channel();
        let worker_cancel = cancelled.clone();
        let worker_progress = progress.clone();
        let worker_snapshot = snapshot.clone();
        std::thread::spawn(move || {
            let completed = (|| {
                crate::verify_installed_editor_runtime(&runtime, &worker_cancel)?;
                let private = crate::prepare_supervised_render(
                    &project,
                    &analysis,
                    &effective,
                    &worker_snapshot,
                    &destination,
                    &worker_progress,
                    Some(&worker_cancel),
                )?;
                crate::verify_installed_editor_runtime(&runtime, &worker_cancel)?;
                private.publish(&project, &runtime, &gate, generation, Some(&worker_cancel))
            })()
            .map_err(|error: crate::RenderBlocker| error.to_string());
            let _ = sender.send(completed);
        });
        self.processing = Some(Processing {
            cancelled,
            progress,
            result,
            snapshot,
            generation,
        });
        self.messages =
            vec!["Rendering the current effective envelopes at each source's native rate.".into()];
    }

    fn export_to(&mut self, destination: PathBuf) {
        if !self.export_ready() {
            self.messages =
                vec!["Export blocked: render the current settings before exporting".into()];
            return;
        }
        let Some(project) = &self.project else { return };
        let Some(runtime) = self.model_acquisition.provenance().cloned() else {
            return;
        };
        let project = project.clone();
        let gate = self.render_generation.clone();
        let generation = gate.current();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancelled.clone();
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let completed = (|| {
                crate::verify_installed_editor_runtime(&runtime, &worker_cancel)?;
                let private = crate::prepare_export(&project, &destination, Some(&worker_cancel))?;
                crate::verify_installed_editor_runtime(&runtime, &worker_cancel)?;
                private.publish(&project, &gate, generation, Some(&worker_cancel))?;
                Ok(destination)
            })()
            .map_err(|error: crate::RenderBlocker| error.to_string());
            let _ = sender.send(completed);
        });
        self.exporting = Some(ExportProcessing { cancelled, result });
        self.messages =
            vec!["Verifying and exporting current native WAVs in the background.".into()];
    }

    fn poll_exporting(&mut self) {
        let Some(exporting) = &self.exporting else {
            return;
        };
        let completed = match exporting.result.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("Delivery worker stopped unexpectedly".into())
            }
        };
        self.exporting = None;
        self.messages = vec![match completed {
            Ok(destination) => format!("Export committed to {}", destination.display()),
            Err(error) => format!("Export was not committed: {error}"),
        }];
    }
    fn export_ready(&self) -> bool {
        !self.sources_busy()
            && !self.reference_stale
            && self.gain_drag.is_none()
            && self.preview_pending.is_none()
            && self.preview_processing.is_none()
            && self.supervised_result.as_ref().is_some_and(|result| self.model_acquisition.provenance() == Some(&result.runtime))
            && self.supervised_result.as_ref().is_some_and(|result| self.project.as_ref().is_some_and(|project| result.snapshot.matches(project)))
            && self.effective_result.as_ref().is_some_and(|effective| effective.analysis_current)
            && self.processing.is_none()
            && self.exporting.is_none()
            && self.export_error.is_none()
              && self.project.as_ref().is_some_and(|project| {
                project.sources.iter().all(|source| source.available)
                    && project.parameters() == self.preview_parameters
                    && matches!(project.render(), RenderState::Current { identity } if identity.is_file())
            })
    }

    fn refresh_export_readiness(&mut self) {
        // Strong source/report/payload verification runs on the delivery worker.
        self.export_error = self.project.as_ref().and_then(|project| {
            if matches!(project.render(), RenderState::Current { .. }) {
                project
                    .validate_interactive_sources()
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

    fn start_preview(&mut self) {
        self.queue_preview(Duration::ZERO);
        self.launch_preview();
    }

    fn queue_preview(&mut self, delay: Duration) {
        if self.reference_stale && self.preview.is_none() && !self.reference_analyze_requested {
            return;
        }
        if self.project.is_none() {
            return;
        }
        // Drag events belong to the next coalesced snapshot. Keep the current
        // snapshot installable until that boundary, even under continuous input.
        if delay.is_zero() && self.reference_analyze_requested {
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
                || (delay.is_zero() && self.reference_analyze_requested)
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
        if !self.reference_analyze_requested {
            self.rebuild_effective();
            return;
        }
        self.reference_analyze_requested = false;
        let Some(project) = &self.project else { return };
        let snapshot = match crate::EditorAnalysisIdentity::capture(project) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.preview_error = Some(error.to_string());
                return;
            }
        };
        self.preview_generation += 1;
        let sources = match source_stamps(&project.source_paths()) {
            Ok(sources) => sources,
            Err(error) => {
                self.preview_error = Some(error.to_string());
                return;
            }
        };
        let source_project = project.clone();
        let cache_directory = project.path().with_extension("analysis-cache");
        let runtime = self.model_acquisition.provenance().cloned();
        let progress = Arc::new(AtomicU8::new(0));
        let worker_progress = progress.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        let mut worker = std::mem::take(&mut self.supervised_worker);
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let completed = worker
                .analyze(
                    &source_project,
                    &cache_directory,
                    runtime.as_ref(),
                    &worker_progress,
                    &worker_cancelled,
                )
                .map_err(|e| e.to_string());
            let _ = sender.send((completed, worker));
        });
        self.preview_processing = Some(PreviewProcessing {
            progress,
            cancelled,
            result,
            sources,
            snapshot,
            generation: self.preview_generation,
            source_generation: self.source_generation,
        });
        self.preview_error = None;
    }

    fn rebuild_effective(&mut self) {
        if let Some(project) = &self.project {
            self.gain_constraints = project.gain_constraints();
        }
        let (Some(project), Some(analysis)) = (&self.project, &self.supervised_result) else {
            return;
        };
        let result = crate::project_fade_settings(project).and_then(|settings| {
            self.effective_cache
                .update(project, &analysis.temporal, settings, None)
        });
        match result {
            Ok(effective) => {
                self.reference_stale = !effective.analysis_current
                    || self.model_acquisition.provenance() != Some(&analysis.runtime);
                if let Some(audition) = &self.audition {
                    audition.replace_envelopes(effective.envelopes.clone());
                }
                if let Some(preview) = &mut self.preview {
                    preview.envelopes = effective.envelopes.clone();
                    preview.parameters = project.parameters();
                    preview.protections = project.protections().to_vec();
                } else {
                    self.preview = Some(analysis.preview(project, &effective));
                }
                self.effective_result = Some(effective);
            }
            Err(error) => self.preview_error = Some(error.to_string()),
        }
    }
    fn audition(&mut self, selection_only: bool) {
        let Some(project) = &self.project else { return };
        let envelopes = match crate::editor_audition_envelopes(
            project,
            self.preview.as_ref(),
            self.audition_mode,
        ) {
            Ok(envelopes) => envelopes,
            Err(error) => {
                self.messages = vec![error.to_string()];
                return;
            }
        };
        let selection = if selection_only {
            self.timeline.selection()
        } else {
            None
        };
        let start = selection.map_or(self.timeline.playhead(), |range| range.0.min(range.1));
        let end = selection.map_or_else(
            || {
                envelopes
                    .iter()
                    .map(|envelope| envelope.duration())
                    .max()
                    .unwrap_or(start)
            },
            |range| range.0.max(range.1),
        );
        self.audition.take().inspect(|playing| playing.stop());
        self.audition_range = Some((start, end));
        if !self.audition_preparation.is_requested() {
            self.audition_preparation.request(project.source_paths());
        }
    }

    fn gain_event(&mut self, track: usize, event: crate::GainEditorEvent) {
        self.render_generation.invalidate();
        match event {
            crate::GainEditorEvent::Begin(id, point) => {
                if let Some(project) = &self.project
                    && let Some(source) = project.sources.get(track)
                {
                    match crate::GainPointDrag::begin(project, source.track_id, id, point) {
                        Ok(drag) => {
                            self.gain_drag = Some(drag);
                            self.selected_gain_point = Some((track, id, point));
                        }
                        Err(error) => self.preview_error = Some(error.to_string()),
                    }
                }
            }
            crate::GainEditorEvent::Move(time, gain) => {
                let Some(drag) = &mut self.gain_drag else {
                    return;
                };
                if let Err(error) = drag.move_to(time, gain) {
                    self.preview_error = Some(error.to_string());
                    return;
                }
                if let (Some(project), Some(analysis)) = (&self.project, &self.supervised_result) {
                    let result = crate::project_fade_settings(project).and_then(|settings| {
                        self.effective_cache.preview_edit(
                            project,
                            &analysis.temporal,
                            settings,
                            drag.draft(),
                            None,
                        )
                    });
                    match result {
                        Ok(effective) => {
                            if let Some(audition) = &self.audition {
                                audition.replace_envelopes(effective.envelopes.clone());
                            }
                            if let Some(preview) = &mut self.preview {
                                preview.envelopes = effective.envelopes.clone();
                            } else {
                                self.preview = Some(analysis.preview(project, &effective));
                            }
                            self.effective_result = Some(effective);
                        }
                        Err(error) => self.preview_error = Some(error.to_string()),
                    }
                }
            }
            crate::GainEditorEvent::End => {
                if let (Some(drag), Some(project)) = (self.gain_drag.take(), self.project.as_mut())
                    && let Err(error) = project.commit_gain_edit(drag.finish())
                {
                    self.preview_error = Some(error.to_string());
                }
                self.rebuild_effective();
            }
            crate::GainEditorEvent::Cancel => {
                self.gain_drag = None;
                self.rebuild_effective();
            }
        }
    }

    fn gain_action(&mut self, track: usize, preserve: bool) {
        let (Some(project), Some((start, end))) = (&mut self.project, self.timeline.selection())
        else {
            return;
        };
        let Some(source) = project.sources.get(track) else {
            return;
        };
        let track_id = source.track_id;
        let Ok(range) = TimeRange::new(start.min(end), start.max(end)) else {
            return;
        };
        let action = if preserve {
            crate::GainRegionAction::Preserve
        } else {
            crate::GainRegionAction::Attenuate(
                10_f32.powf(self.preview_parameters.attenuation_db / 20.0),
            )
        };
        let result = project
            .region_gain_edit(track_id, range, action)
            .and_then(|mut draft| {
                if self.replace_gain_constraints {
                    let new_edit = draft.constraints.manual_edits.last().map(|edit| edit.id);
                    draft.constraints.manual_edits.retain(|edit| {
                        (!preserve && Some(edit.id) == new_edit)
                            || edit.source.track != track_id
                            || edit.points.last().is_none_or(|p| p.time <= range.start)
                            || edit.points.first().is_none_or(|p| p.time >= range.end)
                    });
                    let last = draft.constraints.protections.len().saturating_sub(1);
                    draft.constraints.protections = draft
                        .constraints
                        .protections
                        .into_iter()
                        .enumerate()
                        .filter_map(|(index, protection)| {
                            (preserve && index == last
                                || !protection.sources.iter().any(|s| s.track == track_id)
                                || protection.range.end <= range.start
                                || protection.range.start >= range.end)
                                .then_some(protection)
                        })
                        .collect();
                }
                project.commit_gain_edit(draft)
            });
        if let Err(error) = result {
            self.preview_error = Some(format!(
                "Edit not saved: {error}. Select explicit replacement to replace overlapping edits."
            ));
        } else {
            self.preview_error = None;
        }
        self.rebuild_effective();
    }

    fn handle_edit_key(&mut self, key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
        if self.reference_menu.is_some() {
            let message = match key.as_ref() {
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    Some(Message::DismissReferenceMenu)
                }
                keyboard::Key::Named(keyboard::key::Named::Enter) => Some(
                    Message::ContextReference(self.reference_speakers.clone(), false),
                ),
                keyboard::Key::Character("r" | "R") => {
                    Some(Message::ContextReference(vec![], false))
                }
                keyboard::Key::Character(number) => number
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| index.checked_sub(1))
                    .and_then(|index| self.project.as_ref()?.sources.get(index))
                    .map(|source| Message::ContextReference(vec![source.speaker_id], false)),
                _ => None,
            };
            if let Some(message) = message {
                let _ = self.update(message);
            }
            return true;
        }
        if modifiers.command() {
            let message = match key.as_ref() {
                keyboard::Key::Character("z") if self.gain_tool && modifiers.shift() => {
                    Some(Message::RedoGain)
                }
                keyboard::Key::Character("z") if self.gain_tool => Some(Message::UndoGain),
                keyboard::Key::Character("y") if self.gain_tool => Some(Message::RedoGain),
                keyboard::Key::Character("z") if modifiers.shift() => Some(Message::RedoReferences),
                keyboard::Key::Character("z") => Some(Message::UndoReferences),
                keyboard::Key::Character("y") => Some(Message::RedoReferences),
                _ => None,
            };
            if let Some(message) = message {
                let _ = self.update(message);
                return true;
            }
        }
        if !self.gain_tool {
            return false;
        }
        if key.as_ref() == keyboard::Key::Named(keyboard::key::Named::Escape) {
            self.gain_drag = None;
            self.rebuild_effective();
            return true;
        }
        if key.as_ref() == keyboard::Key::Named(keyboard::key::Named::Delete) {
            let _ = self.update(Message::DeleteGain);
            return true;
        }
        let direction = match key.as_ref() {
            keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => Some((-1_i64, 0.0)),
            keyboard::Key::Named(keyboard::key::Named::ArrowRight) => Some((1, 0.0)),
            keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some((0, 0.01)),
            keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some((0, -0.01)),
            _ => None,
        };
        if let (Some((time_delta, gain_delta)), Some((track, id, point)), Some(project)) =
            (direction, self.selected_gain_point, &self.project)
            && let Some(old) = project
                .manual_gain_edits()
                .iter()
                .find(|edit| edit.id == id)
                .and_then(|edit| edit.points.get(point))
                .copied()
        {
            let time = old
                .time
                .saturating_add_signed(time_delta * (TICKS_PER_SECOND / 100) as i64);
            self.gain_event(track, crate::GainEditorEvent::Begin(id, point));
            self.gain_event(
                track,
                crate::GainEditorEvent::Move(time, (old.gain + gain_delta).clamp(0.0, 1.0)),
            );
            self.gain_event(track, crate::GainEditorEvent::End);
            return true;
        }
        false
    }

    fn poll_processing(&mut self) {
        if let Some(processing) = &self.preview_processing
            && let result = processing.result.try_recv()
            && !matches!(result, Err(mpsc::TryRecvError::Empty))
        {
            let processing = self.preview_processing.take().unwrap();
            let cancelled = processing.cancelled.load(Ordering::Relaxed);
            let sources_match = self
                .project
                .as_ref()
                .is_some_and(|project| processing.snapshot.matches(project));
            if let Ok((completed, worker)) = result {
                if processing.source_generation == self.source_generation {
                    self.supervised_worker = worker;
                }
                if !cancelled && processing.generation == self.preview_generation && sources_match {
                    match completed {
                        Ok(analysis) => {
                            let project = self.project.as_mut().unwrap();
                            match analysis.install(
                                project,
                                &mut self.effective_cache,
                                self.model_acquisition.provenance(),
                                None,
                            ) {
                                Ok(effective) => {
                                    let preview = analysis.preview(project, &effective);
                                    if let Some(audition) = &self.audition {
                                        audition.replace_envelopes(preview.envelopes.clone());
                                    }
                                    self.preview_sources = processing.sources.clone();
                                    self.preview = Some(preview);
                                    self.effective_result = Some(effective);
                                    self.review =
                                        Some(ReviewWorkspace::from_ownership(&analysis.temporal));
                                    self.supervised_result = Some(analysis);
                                    self.reference_stale = false;
                                    self.gain_constraints = project.gain_constraints();
                                    self.preview_error = None;
                                }
                                Err(error) => self.preview_error = Some(error.to_string()),
                            }
                        }
                        Err(error) => {
                            self.preview_error = Some(error);
                            self.reference_stale = true;
                        }
                    }
                }
            } else if !cancelled && processing.generation == self.preview_generation {
                self.preview_error =
                    Some("Analysis worker stopped unexpectedly; previous preview retained.".into());
            }
        }
        self.launch_preview();
        let Some(processing) = &self.processing else {
            return;
        };
        let completed = match processing.result.try_recv() {
            Ok(completed) => completed,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("Final render worker stopped unexpectedly".into())
            }
        };
        let processing = self.processing.take().unwrap();
        let current = !processing.cancelled.load(Ordering::Relaxed)
            && self.render_generation.current() == processing.generation
            && self.model_acquisition.provenance() == Some(&processing.snapshot.runtime)
            && self
                .project
                .as_ref()
                .is_some_and(|project| processing.snapshot.matches(project));
        match completed {
            Ok(artifact) if current => {
                let project = self.project.as_mut().unwrap();
                match artifact.accept(project) {
                    Ok(()) => {
                        self.messages = vec!["Final render current; ready to export WAVs".into()];
                        self.refresh_export_readiness();
                    }
                    Err(error) => {
                        self.messages = vec![format!("Final render not accepted: {error}")]
                    }
                }
            }
            Ok(_) => {
                self.messages =
                    vec!["Final render became obsolete; previous accepted output retained.".into()]
            }
            Err(error) => self.messages = vec![format!("Final render not committed: {error}")],
        }
    }
    fn poll_audition(&mut self) {
        if let Some(result) = self.audition_preparation.poll() {
            self.audition_ready = result.is_ok();
            if let Err(error) = &result {
                self.messages = vec![format!("Playback unavailable: {error}")];
            }
            if let Some((start, end)) = self.audition_range.take() {
                match result {
                    Ok(prepared) => {
                        if let Some(project) = &self.project {
                            let envelopes = crate::editor_audition_envelopes(
                                project,
                                self.preview.as_ref(),
                                self.audition_mode,
                            );
                            match envelopes {
                                Ok(envelopes)
                                    if prepared.is_current(&project.source_paths())
                                        && source_stamps(&project.source_paths()).is_ok_and(
                                            |sources| prepared.matches_sources(&sources),
                                        ) =>
                                {
                                    match LiveAudition::start(
                                        prepared,
                                        envelopes,
                                        self.audition_mode,
                                        10.0_f32.powf(self.monitor_db / 20.0),
                                        start,
                                        end,
                                    ) {
                                        Ok(audition) => self.audition = Some(audition),
                                        Err(error) => self.messages = vec![error.to_string()],
                                    }
                                }
                                Ok(_) => {
                                    self.audition_preparation.invalidate();
                                    self.audition_ready = false;
                                    self.messages = vec![
                                    "Sources changed; prepare the session again before audition."
                                        .into(),
                                ];
                                }
                                Err(error) => self.messages = vec![error.to_string()],
                            }
                        }
                    }
                    Err(error) => self.messages = vec![error.to_string()],
                }
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
        if mode != AuditionMode::Original && self.preview.is_none() {
            return;
        }
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

    #[cfg(test)]
    fn save_calibration(&mut self, kind: CalibrationKind, track: usize) {
        let Some(project) = &self.project else {
            self.calibration_error = Some("Import sources before labeling.".into());
            return;
        };
        if !project.ownership_settings().enabled {
            self.calibration_error = Some("Enable joint ownership before adding examples.".into());
            return;
        }
        let Some(source) = project.sources.get(track) else {
            return;
        };
        self.reference_speakers = if kind == CalibrationKind::SoloSpeech {
            vec![source.speaker_id]
        } else {
            vec![]
        };
        self.save_reference(false);
    }

    fn mark_reference_edit(&mut self) {
        self.reference_stale = true;
        self.reference_analyze_requested = false;
        self.preview_generation = self.preview_generation.wrapping_add(1);
        self.preview_pending = None;
        self.calibration_error = None;
        // Completed fixed evidence can be reused; this generation cannot install.
    }
    fn reference_conflicts(&self) -> Vec<u64> {
        let Ok((start, end)) = self.calibration_range() else {
            return vec![];
        };
        let mut active = self.reference_speakers.clone();
        active.sort_unstable();
        self.project.as_ref().map_or_else(Vec::new, |project| {
            project
                .reference_labels()
                .iter()
                .filter(|label| {
                    let mut previous = label.active_speakers.clone();
                    previous.sort_unstable();
                    label.range.start < end && start < label.range.end && active != previous
                })
                .map(|label| label.id)
                .collect()
        })
    }
    fn save_reference(&mut self, replace_conflicts: bool) {
        let result = (|| {
            let (start, end) = self.calibration_range()?;
            let replacing = if replace_conflicts {
                self.reference_conflicts()
            } else {
                vec![]
            };
            self.project
                .as_mut()
                .ok_or("Import sources before labeling.")?
                .add_reference_label(
                    TimeRange { start, end },
                    self.reference_speakers.clone(),
                    &replacing,
                )
                .map_err(|error| error.to_string())
        })();
        match result {
            Ok(_) => self.mark_reference_edit(),
            Err(error) => self.calibration_error = Some(error),
        }
    }

    fn capture_calibration_range(&mut self, start: TimePosition, end: TimePosition) {
        self.calibration_start = format!("{:.8}", start.min(end) as f64 / TICKS_PER_SECOND as f64);
        self.calibration_end = format!("{:.8}", start.max(end) as f64 / TICKS_PER_SECOND as f64);
        self.calibration_error = None;
    }

    fn calibration_range(&self) -> Result<(TimePosition, TimePosition), String> {
        let sample = |value: &str, round_up: bool| {
            let seconds = value.trim().parse::<f64>().map_err(
                |_| "Drag across a waveform to select an example, or enter start and end seconds.",
            )?;
            time_from_seconds(
                seconds,
                if round_up {
                    Rounding::Ceil
                } else {
                    Rounding::Floor
                },
            )
            .map_err(|_| "Times must be finite, nonnegative seconds within the recording.")
        };
        let start = sample(&self.calibration_start, false)?;
        let end = sample(&self.calibration_end, true)?;
        if end <= start {
            return Err("No usable range: the end must be after the start. Drag across the waveform rather than clicking once.".into());
        }
        if end - start < TICKS_PER_SECOND / 4 {
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
        if self.reference_menu.is_some()
            && matches!(
                message,
                Message::TrackTimeline(_, _)
                    | Message::CompleteTrackSelection(_, _, _, _)
                    | Message::Timeline(_)
                    | Message::GainEvent(_, _)
                    | Message::CalibrationStart(_)
                    | Message::CalibrationEnd(_)
            )
        {
            return Task::none();
        }
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
                    | Message::DownloadModel
                    | Message::VerifyModel
                    | Message::CancelModelDownload
                    | Message::ImportStems
                    | Message::Tick
                    | Message::Stop
                    | Message::Timeline(_)
                    | Message::RangeModifier(_)
            )
        {
            return Task::none();
        }
        if matches!(
            &message,
            Message::AnalyzeReferences
                | Message::Cancel
                | Message::CancelAnalysis
                | Message::DownloadModel
                | Message::VerifyModel
                | Message::CancelModelDownload
                | Message::Attenuation(_)
                | Message::OpenTime(_)
                | Message::CloseTime(_)
                | Message::RefreshPreview
                | Message::GainEvent(_, _)
                | Message::GainAction(_, _)
                | Message::UndoGain
                | Message::RedoGain
                | Message::DeleteGain
                | Message::InsertGainPoint
                | Message::SaveReference(_)
                | Message::RemoveReference(_)
                | Message::UndoReferences
                | Message::RedoReferences
                | Message::SuppliedAlignment(_)
                | Message::RebindSource(_)
                | Message::ProtectTrack(_)
                | Message::ProtectRegion
                | Message::ProtectSelection
                | Message::ImportStems
                | Message::OpenProject
                | Message::FileDropped(_)
                | Message::Relink(_)
        ) {
            self.render_generation.invalidate();
        }
        let previous_parameters = self.preview_parameters;
        match message {
            Message::CompleteTrackSelection(track, start, end, point) => {
                let _ = self.update(Message::TrackTimeline(
                    track,
                    TimelineEvent::Select(start, end),
                ));
                return self.update(Message::OpenReferenceMenu(track, point));
            }
            Message::TrackTimeline(track, event) => {
                self.reference_menu = None;
                if let TimelineEvent::Select(start, end) = event
                    && start == end
                {
                    let _ = self.update(Message::Timeline(TimelineEvent::Seek(start)));
                }
                let selecting = matches!(event, TimelineEvent::Select(_, _));
                let task = self.update(Message::Timeline(event));
                if selecting {
                    self.selection_track = Some(track);
                }
                return task;
            }
            Message::OpenReferenceMenu(track, point) => {
                self.reference_menu = None;
                if self.selection_track == Some(track)
                    && self
                        .project
                        .as_ref()
                        .is_some_and(|p| track < p.sources.len())
                    && let Some((start, end)) = self.timeline.selection()
                    && start < end
                {
                    self.capture_calibration_range(start, end);
                    self.reference_speakers =
                        vec![self.project.as_ref().unwrap().sources[track].speaker_id];
                    self.calibration_error = None;
                    self.calibration_timing = false;
                    self.reference_menu = Some((track, point));
                }
            }
            Message::DismissReferenceMenu => self.reference_menu = None,
            Message::ModelDetails(show) => self.model_details = show,
            Message::ContextReference(speakers, replace) => {
                if let Some((track, _)) = self.reference_menu
                    && self.selection_track == Some(track)
                {
                    self.reference_speakers = speakers;
                    let _ = self.update(Message::SaveReference(replace));
                    if self.calibration_error.is_none() {
                        self.reference_menu = None;
                        self.calibrating = false;
                    }
                }
            }
            Message::DownloadModel => self.model_acquisition.begin(true),
            Message::VerifyModel => self.model_acquisition.begin(false),
            Message::CancelModelDownload => self.model_acquisition.cancel(),
            Message::ImportStems => self.import_stems(),
            Message::OpenProject => self.open_project(),
            Message::FileDropped(path) => {
                if path.extension().is_some_and(|extension| {
                    [
                        "wav", "bwf", "aif", "aiff", "flac", "mp3", "m4a", "mp4", "ogg",
                    ]
                    .iter()
                    .any(|supported| extension.eq_ignore_ascii_case(supported))
                }) {
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
                    .add_filter(
                        "Mono audio",
                        &[
                            "wav", "bwf", "aif", "aiff", "flac", "mp3", "m4a", "mp4", "ogg",
                        ],
                    )
                    .pick_file()
                {
                    self.begin_source_loading(SourceRequest::Relink(index, path));
                }
            }
            Message::Validate => self.validate(),
            Message::SuppliedAlignment(declared) => {
                if let Some(project) = &mut self.project {
                    if let Err(error) = project.set_supplied_alignment(declared) {
                        self.messages = vec![error.to_string()];
                    } else {
                        self.start_preview();
                    }
                }
            }
            Message::Process => self.start_processing(),
            Message::Cancel => {
                if let Some(processing) = &self.processing {
                    processing.cancelled.store(true, Ordering::Relaxed);
                }
                if let Some(exporting) = &self.exporting {
                    exporting.cancelled.store(true, Ordering::Relaxed);
                }
            }
            Message::Tick => {
                self.model_acquisition.poll();
                self.poll_sources();
                if !self.sources_busy() {
                    self.poll_processing();
                }
                self.poll_audition();
                self.poll_exporting();
            }
            #[cfg(test)]
            Message::SpeechSensitivity(value) => {
                self.preview_parameters.speech_sensitivity = 1.0 - value
            }
            Message::Attenuation(value) => self.preview_parameters.attenuation_db = value,
            #[cfg(test)]
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
            Message::KeyPressed(key, modifiers) => {
                if self.handle_edit_key(&key, modifiers) {
                    return Task::none();
                }
                match key.as_ref() {
                    keyboard::Key::Named(keyboard::key::Named::Space) => {
                        if self.audition.is_some() || self.audition_range.is_some() {
                            self.stop_audition();
                        } else {
                            self.audition(false);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Home) => {
                        self.seek_to_selection_start()
                    }
                    keyboard::Key::Character("1") => self.set_audition_mode(AuditionMode::Original),
                    keyboard::Key::Character("2") => {
                        self.set_audition_mode(AuditionMode::Processed)
                    }
                    keyboard::Key::Character("3") => self.set_audition_mode(AuditionMode::Removed),
                    _ => {}
                }
            }
            Message::RangeModifier(selecting) => self.selecting_range = selecting,
            #[cfg(test)]
            Message::OwnershipMode(enabled) => {
                if !enabled {
                    self.calibrating = false;
                }
                if let Some(project) = &mut self.project {
                    match project.set_ownership_enabled(enabled) {
                        Ok(()) => self.mark_reference_edit(),
                        Err(error) => self.calibration_error = Some(error.to_string()),
                    }
                }
            }
            #[cfg(test)]
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
                self.calibration_start = format!(
                    "{:.8}",
                    self.timeline.playhead() as f64 / TICKS_PER_SECOND as f64
                );
            }
            Message::CalibrationEndAtPlayhead => {
                self.calibration_end = format!(
                    "{:.8}",
                    self.timeline.playhead() as f64 / TICKS_PER_SECOND as f64
                );
            }
            Message::CalibrationFromSelection => {
                if let Some((start, end)) = self.timeline.selection() {
                    self.capture_calibration_range(start, end);
                }
            }
            #[cfg(test)]
            Message::SaveCalibration(kind, track) => self.save_calibration(kind, track),
            Message::ReferenceSpeaker(speaker, active) => {
                self.reference_speakers.retain(|id| *id != speaker);
                if active {
                    self.reference_speakers.push(speaker);
                }
            }
            Message::SaveReference(replace) => self.save_reference(replace),
            Message::RemoveReference(id) => {
                if let Some(project) = &mut self.project {
                    match project.remove_reference_label(id) {
                        Ok(()) => self.mark_reference_edit(),
                        Err(error) => self.calibration_error = Some(error.to_string()),
                    }
                }
            }
            Message::UndoReferences | Message::RedoReferences => {
                if let Some(project) = &mut self.project {
                    let result = if matches!(message, Message::UndoReferences) {
                        project.undo_reference_labels()
                    } else {
                        project.redo_reference_labels()
                    };
                    match result {
                        Ok(true) => self.mark_reference_edit(),
                        Ok(false) => {}
                        Err(error) => self.calibration_error = Some(error.to_string()),
                    }
                }
            }
            Message::AnalyzeReferences => {
                self.reference_analyze_requested = true;
                self.start_preview();
            }
            Message::CancelAnalysis => {
                self.preview_generation = self.preview_generation.wrapping_add(1);
                self.reference_analyze_requested = false;
                self.preview_pending = None;
                if let Some(job) = &self.preview_processing {
                    job.cancelled.store(true, Ordering::Relaxed);
                }
                self.messages = vec!["Analysis cancelled; previous preview retained.".into()];
            }
            Message::GainTool(enabled) => {
                self.gain_tool = enabled;
                self.gain_drag = None;
                self.rebuild_effective();
            }
            Message::GainEvent(track, event) => self.gain_event(track, event),
            Message::GainAction(track, preserve) => self.gain_action(track, preserve),
            Message::ReplaceGainConstraints(replace) => self.replace_gain_constraints = replace,
            Message::UndoGain | Message::RedoGain => {
                self.gain_drag = None;
                if let Some(project) = &mut self.project {
                    let result = if matches!(message, Message::UndoGain) {
                        project.undo_gain_edit()
                    } else {
                        project.redo_gain_edit()
                    };
                    if let Err(error) = result {
                        self.preview_error = Some(error.to_string());
                    }
                }
                self.rebuild_effective();
            }
            Message::DeleteGain => {
                if let (Some((_, id, _)), Some(project)) =
                    (self.selected_gain_point.take(), self.project.as_mut())
                {
                    let mut constraints = project.gain_constraints();
                    constraints.manual_edits.retain(|edit| edit.id != id);
                    if let Err(error) = project.replace_gain_constraints(constraints) {
                        self.preview_error = Some(error.to_string());
                    }
                    self.rebuild_effective();
                }
            }
            Message::InsertGainPoint => {
                if let (Some((_, id, _)), Some(project)) =
                    (self.selected_gain_point, self.project.as_mut())
                {
                    let time = self.timeline.playhead();
                    let mut constraints = project.gain_constraints();
                    if let Some(edit) = constraints
                        .manual_edits
                        .iter_mut()
                        .find(|edit| edit.id == id)
                        && time > edit.points[0].time
                        && time < edit.points.last().unwrap().time
                        && !edit.points.iter().any(|point| point.time == time)
                    {
                        let after = edit.points.partition_point(|point| point.time < time);
                        let left = edit.points[after - 1];
                        let right = edit.points[after];
                        let gain = left.gain
                            + (right.gain - left.gain) * (time - left.time) as f32
                                / (right.time - left.time) as f32;
                        edit.points
                            .insert(after, crate::EnvelopePoint { time, gain });
                        if let Err(error) = project.replace_gain_constraints(constraints) {
                            self.preview_error = Some(error.to_string());
                        }
                        self.rebuild_effective();
                    }
                }
            }
            Message::RebindSource(index) => {
                if let Some(project) = &mut self.project
                    && let Some(source) = project.sources.get(index)
                {
                    let track = source.track_id;
                    match project.review_and_rebind_source(track) {
                        Ok(()) => self.mark_reference_edit(),
                        Err(error) => self.preview_error = Some(error.to_string()),
                    }
                }
            }
            Message::SelectSupervisedWarning(index) => {
                if let Some(review) = &mut self.review {
                    self.selection_track = None;
                    self.reference_menu = None;
                    review.select_warning(index);
                    self.timeline.seek(review.playhead());
                    self.timeline.set_selection(review.selection());
                    if let Some(audition) = &self.audition {
                        audition.seek(review.playhead());
                    }
                    self.selected_supervised_warning = Some(index);
                }
            }
            Message::SaveProject => {
                if let Some(project) = &self.project {
                    self.messages = vec![match project.save() {
                        Ok(()) => "Editable project saved.".into(),
                        Err(error) => format!("Project was not saved: {error}"),
                    }];
                }
            }
            #[cfg(test)]
            Message::RemoveCalibration(index) => {
                let id = self
                    .project
                    .as_ref()
                    .and_then(|project| project.reference_labels().get(index))
                    .map(|label| label.id);
                if let Some(id) = id
                    && let Some(project) = &mut self.project
                {
                    match project.remove_reference_label(id) {
                        Ok(()) => self.mark_reference_edit(),
                        Err(error) => self.calibration_error = Some(error.to_string()),
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
                    self.selection_track = None;
                    self.reference_menu = None;
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
            if let Some(project) = &mut self.project
                && let Err(error) = project.set_parameters(self.preview_parameters)
            {
                self.preview_error = Some(error.to_string());
            }
            self.queue_preview(Duration::from_millis(40));
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let drops = iced::event::listen_with(|event, status, _window| match event {
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                keyboard_message(key, modifiers, status)
            }
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::RangeModifier(modifiers.shift()))
            }
            _ => None,
        });
        let processing = if self.processing.is_some()
            || self.exporting.is_some()
            || self.model_acquisition.is_busy()
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
        responsive(|size| {
            let count = self
                .project
                .as_ref()
                .map_or(0, |project| project.sources.len());
            self.workspace(compact_workspace(size.width) && count <= 3, size.height)
        })
        .into()
    }

    fn missing_references(&self) -> (Vec<String>, bool) {
        let Some(project) = &self.project else {
            return (vec![], false);
        };
        let missing = project
            .sources
            .iter()
            .filter(|source| {
                !project.reference_labels().iter().any(|label| {
                    label.active_speakers == vec![source.speaker_id]
                        && project.reference_label_is_current(label)
                })
            })
            .map(|source| format!("Speaker {} solo", source.speaker_id.0))
            .collect();
        let room = project.reference_labels().iter().any(|label| {
            label.active_speakers.is_empty() && project.reference_label_is_current(label)
        });
        (missing, room)
    }

    fn workspace_status(&self) -> String {
        if self.sources_busy() {
            return "LOADING TRACKS".into();
        }
        let Some(project) = &self.project else {
            return "IMPORT TRACKS".into();
        };
        if self.preview_processing.is_some() {
            return "ANALYZING".into();
        }
        if self.model_acquisition.is_busy() {
            return "MODEL SETUP".into();
        }
        let (missing, room) = self.missing_references();
        if self.model_acquisition.provenance().is_none() {
            return if missing.is_empty() && room {
                "VERIFY MODEL"
            } else {
                "VERIFY MODEL · ADD REFERENCES"
            }
            .into();
        }
        if !missing.is_empty() || !room {
            return "ADD SPEAKER / ROOM REFERENCES".into();
        }
        if let Some(result) = &self.supervised_result {
            if !result.snapshot.matches(project)
                || self.model_acquisition.provenance() != Some(&result.runtime)
            {
                return "ANALYSIS OUT OF DATE".into();
            }
            let warnings = self
                .review
                .as_ref()
                .map_or(0, |review| review.warnings.len());
            return if warnings == 0 {
                "ANALYSIS COMPLETE".into()
            } else {
                format!("ANALYSIS COMPLETE · {warnings} REVIEW REGIONS")
            };
        }
        "REFERENCES SAVED · ANALYZE TO VALIDATE".into()
    }

    fn next_action(&self) -> &'static str {
        let Some(project) = &self.project else {
            return "Open a project or import mono speaker tracks.";
        };
        if self.preview_processing.is_some() {
            return "Analyzing. Original playback remains available.";
        }
        let (missing, room) = self.missing_references();
        if !missing.is_empty() || !room {
            return "Listen in Original. Drag a few solo examples and room tone, then label them.";
        }
        if project.require_alignment().is_err() {
            return "Confirm synchronized recording starts below before Analyze.";
        }
        if self.model_acquisition.provenance().is_none() {
            return "Verify the model below, then Analyze your references.";
        }
        if self.reference_stale || self.supervised_result.is_none() {
            return "Click Analyze to learn from your saved references.";
        }
        "Listen in Processed and Removed. Adjust attenuation and fades, then export."
    }

    fn calibration_controls(&self) -> Element<'_, Message> {
        let Some(project) = &self.project else {
            return column![].into();
        };
        let mut controls = column![
            text("Speaker references").size(18),
            text("Drag a clear example on any track, then choose who is talking.").size(12),
            text(format!(
                "{} saved references",
                project.reference_labels().len()
            ))
            .size(12),
        ]
        .spacing(8);
        if self.calibration_timing {
            for source in &project.sources {
                let speaker = source.speaker_id;
                controls = controls.push(
                    iced::widget::checkbox(self.reference_speakers.contains(&speaker))
                        .label(format!("Speaker {}", speaker.0))
                        .on_toggle(move |active| Message::ReferenceSpeaker(speaker, active)),
                );
            }
            controls =
                controls.push(button("Save reference").on_press(Message::SaveReference(false)));
            controls = controls.push(
                button("Listen to selected range").on_press_maybe(
                    self.timeline
                        .selection()
                        .map(|_| Message::AuditionSelection),
                ),
            );
            if !self.reference_conflicts().is_empty() {
                controls = controls.push(
                    button("Replace conflicting whole labels")
                        .on_press(Message::SaveReference(true)),
                );
            }
        }
        controls = controls.push(
            row![
                button("Undo label").on_press(Message::UndoReferences),
                button("Redo label").on_press(Message::RedoReferences)
            ]
            .spacing(8),
        );
        controls = controls.push(
            column![
                button("Analyze").on_press(Message::AnalyzeReferences),
                button("Save project").on_press(Message::SaveProject),
            ]
            .spacing(6),
        );
        if self.preview_processing.is_some() {
            controls = controls.push(button("Cancel analysis").on_press(Message::CancelAnalysis));
        }
        controls = controls.push(
            iced::widget::checkbox(self.gain_tool)
                .label("Envelope edit tool")
                .on_toggle(Message::GainTool),
        );
        if self.gain_tool {
            controls = controls.push(text("Drag pink points to change gain or fade duration. Arrow keys move a selected point; Delete removes its override. Ctrl+Z / Ctrl+Y undo and redo.").size(12));
            controls = controls.push(
                row![
                    button("Undo gain").on_press(Message::UndoGain),
                    button("Redo gain").on_press(Message::RedoGain)
                ]
                .spacing(6),
            );
            controls = controls.push(
                column![
                    button("Add point at cursor").on_press(Message::InsertGainPoint),
                    button("Remove selected override").on_press(Message::DeleteGain)
                ]
                .spacing(6),
            );
            controls = controls.push(
                iced::widget::checkbox(self.replace_gain_constraints)
                    .label("Replace entire overlapping overrides and protections")
                    .on_toggle(Message::ReplaceGainConstraints),
            );
        }
        let (missing, room) = self.missing_references();
        controls = controls.push(
            text(if missing.is_empty() && room {
                "References supplied; Analyze checks usable speech and room tone.".into()
            } else {
                format!(
                    "References needed: {}{}",
                    missing.join(", "),
                    if room { "" } else { "; room tone" }
                )
            })
            .size(12),
        );
        if let Some(review) = &self.review {
            let warnings = &review.warnings;
            controls =
                controls.push(text(format!("{} regions need review", warnings.len())).size(12));
            if !warnings.is_empty() {
                let selected = self
                    .selected_supervised_warning
                    .unwrap_or(0)
                    .min(warnings.len() - 1);
                let warning = &warnings[selected];
                controls = controls.push(
                    button(text(format!(
                        "Review {:.2}–{:.2}s: {}",
                        warning.start as f64 / TICKS_PER_SECOND as f64,
                        warning.end as f64 / TICKS_PER_SECOND as f64,
                        warning.label
                    )))
                    .on_press(Message::SelectSupervisedWarning(selected)),
                );
                controls = controls.push(
                    column![
                        button("Previous review").on_press_maybe(
                            selected
                                .checked_sub(1)
                                .map(Message::SelectSupervisedWarning)
                        ),
                        button("Next review").on_press_maybe(
                            (selected + 1 < warnings.len())
                                .then_some(Message::SelectSupervisedWarning(selected + 1))
                        ),
                        button("Mark selected reviewed").on_press_maybe(
                            review.selected_warning().map(|_| Message::ToggleReviewed)
                        )
                    ]
                    .spacing(6),
                );
            }
        }
        if self.calibration_timing {
            controls = controls.push(text({
            match self.calibration_range() {
                Ok((start, end)) => format!(
                    "Selected {:.3}–{:.3} s ({:.3} s). Choose the speakers above and Save reference.",
                    start as f64 / TICKS_PER_SECOND as f64,
                    end as f64 / TICKS_PER_SECOND as f64,
                    (end - start) as f64 / TICKS_PER_SECOND as f64
                ),
                Err(error) => error,
            }
        }));
        }
        controls = controls.push(
            button(if self.calibration_timing {
                "Hide exact times"
            } else {
                "Exact times / saved labels"
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
        if let Some(notice) = project.calibration_notice()
            && (!project.pending_source_reviews().is_empty()
                || self
                    .supervised_result
                    .as_ref()
                    .is_none_or(|result| !result.snapshot.matches(project)))
        {
            controls = controls.push(text(notice));
        }
        if let Some(error) = &self.calibration_error {
            controls = controls.push(text(format!("Example not saved: {error}")));
        }
        if self.calibration_timing {
            for label in project.reference_labels() {
                let speakers = if label.active_speakers.is_empty() {
                    "Room tone".to_owned()
                } else {
                    label
                        .active_speakers
                        .iter()
                        .map(|speaker| format!("Speaker {}", speaker.0))
                        .collect::<Vec<_>>()
                        .join(" + ")
                };
                controls = controls
                    .push(text(format!(
                        "{speakers}: {:.5} to {:.5} s",
                        label.range.start as f64 / TICKS_PER_SECOND as f64,
                        label.range.end as f64 / TICKS_PER_SECOND as f64
                    )))
                    .push(button("Remove reference").on_press(Message::RemoveReference(label.id)));
            }
        }
        controls.into()
    }

    fn model_controls(&self) -> Element<'_, Message> {
        let ready =
            self.model_acquisition.provenance().is_some() && !self.model_acquisition.is_busy();
        let mut controls = column![
            text(if ready {
                "Model ready · offline"
            } else {
                self.model_acquisition.status()
            })
            .size(12),
            button(if self.model_details {
                "Hide model details"
            } else {
                "Model details"
            })
            .on_press(Message::ModelDetails(!self.model_details)),
        ]
        .spacing(6);
        if ready && !self.model_details {
            return controls.into();
        }
        let actions = if self.model_acquisition.is_busy() {
            column![
                progress_bar(0.0..=1.0, self.model_acquisition.progress()),
                button("Cancel model acquisition").on_press(Message::CancelModelDownload),
            ]
        } else {
            column![
                button("Download / verify Nemotron").on_press(Message::DownloadModel),
                button("Verify installed model offline").on_press(Message::VerifyModel),
            ]
        }
        .spacing(6);
        if !ready {
            controls = controls.push(actions);
        }
        if self.model_details {
            controls = controls.push(column![
            iced::widget::tooltip(
                text("Nemotron q8_0 · 107 MB · CPU / offline").size(12),
                text(format!("Pinned {} · SHA-256 {}",crate::model::MODEL_REVISION,crate::model::NEMOTRON.sha256)).size(12),
                iced::widget::tooltip::Position::Bottom,
            ),
            text("Weights: OpenMDW 1.1 · runtime: Apache-2.0").size(11),
            text("Download: NVIDIA / Hugging Face. Agreement: https://openmdw.ai/license/1-1/. Notices accompany the installed files.").size(11),
            button("Verify installed model offline").on_press(Message::VerifyModel),
        ].spacing(6));
        }
        controls.into()
    }

    fn session_controls(&self) -> Element<'_, Message> {
        let Some(project) = &self.project else {
            return column![].into();
        };
        let mut controls =
            column![button("Recheck source files").on_press(Message::Validate),].spacing(8);
        if let Err(error) = project.require_alignment() {
            controls = controls
                .push(
                    text("Before Analyze: confirm the tracks share the same recording start.")
                        .size(12),
                )
                .push(
                    iced::widget::checkbox(project.supplied_alignment())
                        .label("Tracks are synchronized")
                        .on_toggle(Message::SuppliedAlignment),
                )
                .push(text(error.to_string()).size(12));
        }
        for (index, source) in project.sources.iter().enumerate() {
            let mut source_controls =
                column![text(format!("Speaker {} source", source.speaker_id.0)).size(12)]
                    .spacing(4);
            if !source.available {
                source_controls =
                    source_controls.push(button("Relink source").on_press(Message::Relink(index)));
            }
            if project.pending_source_reviews().contains(&source.track_id) {
                source_controls = source_controls.push(
                    button("Accept changed source bindings").on_press(Message::RebindSource(index)),
                );
            }
            if self.preview.is_some() {
                source_controls = source_controls
                    .push(button("Protect entire track").on_press(Message::ProtectTrack(index)));
            }
            if !source.available
                || project.pending_source_reviews().contains(&source.track_id)
                || self.preview.is_some()
            {
                controls = controls.push(source_controls);
            }
        }
        if let Some(processing) = &self.processing {
            controls = controls
                .push(text(format!(
                    "Final render: {}%",
                    processing.progress.load(Ordering::Relaxed)
                )))
                .push(button("Cancel final render").on_press(Message::Cancel));
        } else {
            controls = controls.push(
                button("Create current final WAV render")
                    .on_press_maybe(self.supervised_result.as_ref().map(|_| Message::Process)),
            );
        }
        if self.export_ready() {
            controls = controls.push(button("Export current render").on_press(Message::Export));
        }
        if let Some(error) = &self.export_error {
            controls = controls.push(text(format!("Export blocked: {error}")));
        }
        if self.exporting.is_some() {
            controls = controls
                .push(text("Verifying and copying current delivery…"))
                .push(button("Cancel delivery").on_press(Message::Cancel));
        }
        if self.can_protect_selection() {
            controls = controls.push(
                button("Protect selection across all tracks").on_press(Message::ProtectSelection),
            );
        }
        if self.review.as_ref().is_some_and(|review| {
            review.selected_region().is_some() || review.selected_warning().is_some()
        }) {
            controls =
                controls.push(button("Protect reviewed region").on_press(Message::ProtectRegion));
        }
        for message in &self.messages {
            controls = controls.push(text(message));
        }
        controls.into()
    }

    fn workspace(&self, compact: bool, height: f32) -> Element<'_, Message> {
        let header = row![
            column![
                text("CLEANSLATE").size(24),
                text(
                    self.project
                        .as_ref()
                        .and_then(|project| project.path().file_stem())
                        .map_or_else(
                            || "UNTITLED SESSION".into(),
                            |name| name.to_string_lossy().into_owned()
                        )
                )
                .size(12)
            ]
            .spacing(2),
            Space::new().width(Fill),
            text(self.workspace_status()).size(12),
            button("Import stems").on_press(Message::ImportStems),
            button("Open session").on_press_maybe(
                (!self.sources_busy() && self.processing.is_none()).then_some(Message::OpenProject)
            ),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let mut timeline_content = column![].spacing(6).height(Fill);
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
                        text(
                            "Import synchronized mono tracks or drop them anywhere in this window."
                        ),
                        button("Import mono tracks").on_press(Message::ImportStems),
                    ]
                    .spacing(12)
                    .align_x(iced::Center),
                )
                .center_x(Fill)
                .center_y(Fill)
                .height(Fill),
            );
        } else if let Some(project) = &self.project {
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
                let filename = source
                    .path
                    .file_name()
                    .unwrap_or(source.path.as_os_str())
                    .to_string_lossy();
                let mut short_name = filename.chars().take(24).collect::<String>();
                if filename.chars().count() > 24 {
                    short_name.push('…');
                }
                let mut header = row![
                    iced::widget::tooltip(
                        text(format!("Speaker {} · {short_name}", source.speaker_id.0))
                            .size(13)
                            .width(Fill),
                        text(source.path.display().to_string()).size(12),
                        iced::widget::tooltip::Position::Bottom,
                    ),
                    button(text("Preserve").size(12)).on_press_maybe(
                        self.timeline
                            .selection()
                            .map(|_| Message::GainAction(index, true))
                    ),
                    button(text("Attenuate").size(12)).on_press_maybe(
                        self.timeline
                            .selection()
                            .map(|_| Message::GainAction(index, false))
                    ),
                ]
                .spacing(8)
                .align_y(iced::Center);
                if self
                    .review
                    .as_ref()
                    .is_some_and(|review| review.highlighted_tracks().contains(&index))
                {
                    header = header.push(text("REVIEW").size(12));
                }
                if !source.available {
                    header = header.push(text("Missing source").size(12));
                }
                if project.pending_source_reviews().contains(&source.track_id) {
                    header = header.push(text("Review source").size(12));
                }
                if let Some(analysis) = &self.supervised_result
                    && let Ok(estimates) = analysis.ownership.estimates_at(self.timeline.playhead())
                    && let Some(estimate) = estimates
                        .iter()
                        .find(|estimate| estimate.track == source.track_id)
                {
                    header = header.push(
                        text(estimate.probability.map_or_else(
                            || "Unresolved · unity".into(),
                            |p| format!("Ownership {p:.2}"),
                        ))
                        .size(12),
                    );
                }
                let waveform: Element<'_, Message> = self.waveforms.get(index).map_or_else(
                    || {
                        container(text("Waveform loading…"))
                            .height(Fill)
                            .center_y(Fill)
                            .into()
                    },
                    |cache| {
                        canvas(
                            TimelineCanvas::waveform(
                                cache,
                                &self.timeline,
                                move |event| Message::TrackTimeline(index, event),
                                self.preview.as_ref().map(|preview| (preview, index)),
                                self.selecting_range || self.calibrating,
                            )
                            .with_context(move |point| Message::OpenReferenceMenu(index, point))
                            .with_selection(move |start, end, point| {
                                Message::CompleteTrackSelection(index, start, end, point)
                            })
                            .with_overview(
                                self.supervised_result
                                    .as_ref()
                                    .map(|result| &result.overview),
                            ),
                        )
                        .width(Fill)
                        .height(Fill)
                        .into()
                    },
                );
                timeline_content = timeline_content.push(
                    container(
                        column![
                            header,
                            waveform,
                            canvas(crate::gain_editor::GainCanvas {
                                project,
                                constraints: self
                                    .gain_drag
                                    .as_ref()
                                    .map_or(&self.gain_constraints, |drag| &drag
                                        .draft()
                                        .constraints),
                                track: source.track_id,
                                viewport: &self.timeline,
                                automatic: self
                                    .effective_result
                                    .as_ref()
                                    .and_then(|result| result.automatic.get(index))
                                    .map(AsRef::as_ref),
                                effective: self
                                    .preview
                                    .as_ref()
                                    .and_then(|preview| preview.envelopes.get(index)),
                                editing: self.gain_tool,
                                on_event: Box::new(move |event| Message::GainEvent(index, event)),
                            })
                            .width(Fill)
                            .height(20),
                        ]
                        .spacing(3)
                        .height(Fill),
                    )
                    .padding(4)
                    .height(Length::FillPortion(1))
                    .width(Fill)
                    .style(container::bordered_box),
                );
            }
        }

        let tracks: Element<'_, Message> = if self.sources_busy() || self.project.is_none() {
            scrollable(timeline_content).height(Fill).into()
        } else {
            timeline_content.into()
        };
        let timeline = column![
            text("SESSION TIMELINE").size(12),
            text(
                "Click to seek · drag to label · wheel to zoom · Ctrl-wheel or middle-drag to pan"
            )
            .size(11),
            tracks,
        ]
        .spacing(10)
        .width(Fill)
        .height(Fill);
        let preview_state: Element<'_, Message> = if let Some(processing) = &self.preview_processing
        {
            let progress = processing.progress.load(Ordering::Relaxed) as f32;
            let stage = match progress as u8 {
                0..=19 => "Preparing audio and validating references",
                20..=54 => "Verifying Nemotron and analyzing speaker activity on CPU",
                55..=79 => "Fitting session ownership estimates",
                80..=94 => "Resolving ownership and uncertainty",
                _ => "Composing the current gain preview",
            };
            column![
                text(stage),
                progress_bar(0.0..=100.0, progress),
                text("Stage progress; inference duration is measured, not predicted."),
                text(if self.preview.is_some() {
                    "Previous completed preview remains available"
                } else {
                    "No completed preview yet"
                })
            ]
            .spacing(6)
            .into()
        } else if let Some(error) = &self.preview_error {
            text(format!("Preview failed: {error}")).into()
        } else if self.reference_stale
            || self.supervised_result.as_ref().is_some_and(|analysis| {
                self.model_acquisition.provenance() != Some(&analysis.runtime)
            })
        {
            text("Analysis out of date · click Analyze").into()
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
                self.model_controls(),
                text("Next step").size(18),
                text(self.next_action()).size(13),
                preview_state,
                self.calibration_controls(),
                self.session_controls(),
                text(format!("Attenuation: {:.1} dB", parameters.attenuation_db)),
                slider(
                    -30.0..=-3.0,
                    parameters.attenuation_db,
                    Message::Attenuation
                )
                .step(1.0_f32)
                .on_release(Message::RefreshPreview),
                text(format!("Open fade duration: {:.0} ms", parameters.open_ms)),
                slider(1.0..=1000.0, parameters.open_ms, Message::OpenTime)
                    .step(1.0_f32)
                    .on_release(Message::RefreshPreview),
                text(format!(
                    "Close fade duration: {:.0} ms",
                    parameters.close_ms
                )),
                slider(1.0..=2000.0, parameters.close_ms, Message::CloseTime)
                    .step(1.0_f32)
                    .on_release(Message::RefreshPreview),
                text("Solid: preserved · hatch: attenuated · dots: uncertain").size(11),
            ]
            .spacing(12),
        )
        .padding(18)
        .width(if compact { Fill } else { Length::Fixed(280.0) })
        .style(container::bordered_box);
        let inspector: Element<'_, Message> = if self.sources_busy() {
            container(column![text("Loading tracks").size(20),
                text("Waveforms appear as each track is read. Validation and source checks run in the background, then choose references and click Analyze."),
                text("You can scroll the tracks and drop more supported mono files. Editing becomes available when the import is validated."),
            ].spacing(12)).padding(18).width(if compact { Fill } else { Length::Fixed(280.0) }).into()
        } else {
            scrollable(inspector)
                .height(if compact { Length::Fixed(240.0) } else { Fill })
                .into()
        };
        let transport = row![
            text(format!(
                "{:02}:{:06.3}",
                self.timeline.playhead() / TICKS_PER_SECOND / 60,
                (self.timeline.playhead() % (TICKS_PER_SECOND * 60)) as f64
                    / TICKS_PER_SECOND as f64
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
            .on_press_maybe(
                (self.preview.is_some()
                    || self.audition_mode == AuditionMode::Original
                        && self
                            .project
                            .as_ref()
                            .is_some_and(|project| crate::original_envelopes(project).is_ok()))
                .then_some(Message::Audition)
            ),
            button("Stop").on_press_maybe(
                (self.audition.is_some()
                    || self.audition_range.is_some()
                    || self.audition_preparation.is_busy())
                .then_some(Message::Stop)
            ),
            text(if self.audition_preparation.is_busy() {
                "Preparing playback…"
            } else if self.audition_ready {
                "Playback ready"
            } else {
                ""
            })
            .size(11),
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
            .on_press_maybe(self.preview.as_ref().map(|_| Message::Processed)),
            button(if self.audition_mode == AuditionMode::Removed {
                "Removed ✓"
            } else {
                "Removed"
            })
            .on_press_maybe(self.preview.as_ref().map(|_| Message::Removed)),
            text(format!("Monitor {:+.1} dB", self.monitor_db)),
            slider(-24.0..=6.0, self.monitor_db, Message::MonitorGain)
                .step(1.0_f32)
                .width(120),
        ]
        .spacing(8)
        .align_y(iced::Center);
        let editor: Element<'_, Message> = if compact {
            column![timeline, inspector].spacing(12).height(Fill).into()
        } else {
            row![timeline, inspector].spacing(12).height(Fill).into()
        };

        let base: Element<'_, Message> = container(column![header, editor, transport].spacing(12))
            .padding(12)
            .width(Fill)
            .height(Fill)
            .into();
        let Some((_, point)) = self.reference_menu else {
            return base;
        };
        let overlay = responsive(move |size| {
            let mut menu = column![
                text("Who is talking?").size(16),
                text("1–8: solo · R: room tone · Enter: selected speakers · Esc: close").size(11)
            ]
            .spacing(6);
            if let Some(project) = &self.project {
                for source in &project.sources {
                    let speaker = source.speaker_id;
                    menu = menu.push(
                        button(text(format!("Speaker {} · solo speech", speaker.0)))
                            .on_press(Message::ContextReference(vec![speaker], false))
                            .width(Fill),
                    );
                }
                menu = menu.push(
                    button("Room tone · nobody speaking")
                        .on_press(Message::ContextReference(vec![], false))
                        .width(Fill),
                );
                menu = menu.push(text("Overlap: select everyone speaking").size(12));
                for source in &project.sources {
                    let speaker = source.speaker_id;
                    menu = menu.push(
                        iced::widget::checkbox(self.reference_speakers.contains(&speaker))
                            .label(format!("Speaker {}", speaker.0))
                            .on_toggle(move |active| Message::ReferenceSpeaker(speaker, active)),
                    );
                }
                menu = menu.push(button("Save selected speakers").on_press(
                    Message::ContextReference(self.reference_speakers.clone(), false),
                ));
            }
            if let Some(error) = &self.calibration_error {
                menu = menu.push(text(error).size(12));
                if !self.reference_conflicts().is_empty() {
                    menu = menu.push(button("Replace conflicting whole labels").on_press(
                        Message::ContextReference(self.reference_speakers.clone(), true),
                    ));
                }
            }
            menu = menu.push(button("Cancel").on_press(Message::DismissReferenceMenu));
            let width = 280.0_f32.min(size.width);
            let rows = self.project.as_ref().map_or(0, |p| p.sources.len());
            let height = (150.0
                + rows as f32 * 64.0
                + if self.calibration_error.is_some() {
                    120.0
                } else {
                    0.0
                })
            .min(520.0)
            .min(size.height);
            let x = point.x.min((size.width - width).max(0.0)).max(0.0);
            let y = point.y.min((size.height - height).max(0.0)).max(0.0);
            let panel = container(scrollable(menu))
                .padding(10)
                .width(width)
                .height(height)
                .style(container::bordered_box);
            let positioned = container(opaque(panel)).padding(iced::Padding {
                top: y,
                left: x,
                ..iced::Padding::ZERO
            });
            opaque(
                mouse_area(container(positioned).width(Fill).height(Fill))
                    .on_press(Message::DismissReferenceMenu)
                    .on_right_press(Message::DismissReferenceMenu)
                    .on_scroll(|_| Message::DismissReferenceMenu),
            )
        });
        stack![base, overlay].into()
    }
}
