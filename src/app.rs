#[cfg(test)]
use crate::CalibrationKind;
use crate::{
    AnalysisPreview, AutomixParameters, Comparison, Project, RenderState, ReviewWorkspace,
    SpeakerId, TimeRange,
    audition::{AuditionMode, AuditionPreparation, LiveAudition},
    timebase::{TICKS_PER_SECOND, TimePosition},
    timeline::{PlaybackScroll, TimelineEvent, TimelineViewport},
    validate_export,
    waveform::{self, ExampleEvent, TimelineCanvas, WaveformCache},
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

fn ownership_summary(speakers: &[SpeakerId], states: &[crate::TemporalTrackState]) -> String {
    use crate::TemporalTrackState::{Owned, UncertainUnity};
    let owners = speakers
        .iter()
        .zip(states)
        .filter(|(_, state)| **state == Owned)
        .map(|(speaker, _)| format!("Speaker {}", speaker.0))
        .collect::<Vec<_>>();
    let uncertain = states.contains(&UncertainUnity);
    let mut summary = if owners.is_empty() {
        if uncertain {
            "Uncertain ownership"
        } else {
            "No active speaker"
        }
        .to_string()
    } else {
        format!(
            "{}{}",
            if owners.len() > 1 {
                "Overlap: "
            } else {
                "Active: "
            },
            owners.join(" + ")
        )
    };
    if uncertain && !owners.is_empty() {
        summary.push_str(" · other ownership uncertain");
    }
    summary
}

fn edit_contains(edit: &crate::ManualGainEdit, time: u64) -> bool {
    edit.points.first().is_some_and(|point| point.time <= time)
        && edit.points.last().is_some_and(|point| time < point.time)
}

fn region_marked(constraints: &crate::GainConstraints, track: crate::TrackId, time: u64) -> bool {
    constraints
        .manual_edits
        .iter()
        .any(|edit| edit.source.track == track && edit_contains(edit, time))
        || constraints.protections.iter().any(|protection| {
            protection.track == Some(track)
                && protection.range.start <= time
                && time < protection.range.end
        })
}

fn ownership_meter_left(playhead_x: f32, width: f32, meter_width: f32) -> f32 {
    (playhead_x - meter_width / 2.0).clamp(0.0, (width - meter_width).max(0.0))
}

fn example_time(time: TimePosition) -> String {
    let hundredths = (time as u128 * 100 + TICKS_PER_SECOND as u128 / 2) / TICKS_PER_SECOND as u128;
    format!(
        "{}:{:02}.{:02}",
        hundredths / 6000,
        hundredths / 100 % 60,
        hundredths % 100
    )
}

fn example_speakers(label: &crate::ReferenceLabel) -> String {
    if label.active_speakers.is_empty() {
        "Room tone".to_owned()
    } else {
        label
            .active_speakers
            .iter()
            .map(|speaker| format!("Speaker {}", speaker.0))
            .collect::<Vec<_>>()
            .join(" + ")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn saved_examples_edit_exact_endpoints_speakers_and_history() {
        use super::{ExampleEvent, Message};
        let root =
            std::env::temp_dir().join(format!("cleanslate-example-edit-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let sources = (0..2)
            .map(|index| {
                let path = root.join(format!("speaker-{index}.wav"));
                write_wav(&path, &vec![0.0; 192_000]);
                path
            })
            .collect();
        let mut project = synchronized_project(root.join("session.cleanslate"), sources).unwrap();
        let speakers: Vec<_> = project
            .sources
            .iter()
            .map(|source| source.speaker_id)
            .collect();
        let ticks = crate::TICKS_PER_SECOND;
        let initial = crate::TimeRange::new(ticks / 2 + 1, ticks + 7).unwrap();
        let id = project
            .add_reference_label(initial, vec![speakers[0]], &[])
            .unwrap();
        project
            .add_reference_label(
                crate::TimeRange::new(2 * ticks, 3 * ticks).unwrap(),
                vec![speakers[1]],
                &[],
            )
            .unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            timeline: crate::timeline::TimelineViewport::new(4 * ticks),
            ..Default::default()
        };
        let before = app.project.as_ref().unwrap().reference_labels().to_vec();
        let _ = app.update(Message::ShowExample(id));
        assert_eq!(app.selected_example, Some(id));
        assert!(app.timeline.start() <= initial.start as f64);
        assert!(app.timeline.start() + app.timeline.visible_samples() >= initial.end as f64);
        assert_eq!(
            app.calibration_range().unwrap(),
            (initial.start, initial.end)
        );
        let changed = crate::TimeRange::new(ticks / 4 + 3, ticks + 23).unwrap();
        let _ = app.update(Message::Example(ExampleEvent::Resize(id, changed, false)));
        assert_eq!(app.project.as_ref().unwrap().reference_labels(), before);
        let _ = app.update(Message::Example(ExampleEvent::CancelResize));
        assert_eq!(app.timeline.selection(), Some((initial.start, initial.end)));
        let _ = app.update(Message::Example(ExampleEvent::Resize(id, changed, true)));
        assert!(app.reference_stale);
        assert_eq!(
            app.project.as_ref().unwrap().reference_labels()[0].range,
            changed
        );
        let _ = app.update(Message::UndoReferences);
        assert_eq!(app.project.as_ref().unwrap().reference_labels(), before);
        let _ = app.update(Message::RedoReferences);
        assert_eq!(
            app.project.as_ref().unwrap().reference_labels()[0].range,
            changed
        );
        let conflict = crate::TimeRange::new(changed.start, 5 * ticks / 2).unwrap();
        let _ = app.update(Message::Example(ExampleEvent::Resize(id, conflict, true)));
        assert!(app.calibration_error.is_some());
        assert_eq!(
            app.project.as_ref().unwrap().reference_labels()[0].range,
            changed
        );
        let _ = app.update(Message::OpenTimelineMenu(
            1,
            changed.start,
            iced::Point::ORIGIN,
        ));
        assert!(app.reference_menu.is_some());
        assert_eq!(app.reference_speakers, vec![speakers[0]]);
        let _ = app.update(Message::ContextReference(
            vec![speakers[0], speakers[1]],
            false,
        ));
        let edited = &app.project.as_ref().unwrap().reference_labels()[0];
        assert_eq!(edited.id, id);
        assert_eq!(edited.range, changed);
        assert_eq!(edited.active_speakers, speakers);
        assert_eq!(edited.sources, before[0].sources);
        let reopened = crate::Project::open(app.project.as_ref().unwrap().path()).unwrap();
        assert_eq!(
            reopened.reference_labels(),
            app.project.as_ref().unwrap().reference_labels()
        );
        let _ = app.update(Message::RemoveReference(id));
        assert!(app.selected_example.is_none());
        assert_eq!(app.project.as_ref().unwrap().reference_labels().len(), 1);
        let _ = app.update(Message::UndoReferences);
        assert_eq!(app.project.as_ref().unwrap().reference_labels().len(), 2);
        assert_eq!(super::example_time(148 * ticks + ticks / 10), "2:28.10");
        assert_eq!(super::example_time(60 * ticks - 1), "1:00.00");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lock_buttons_keep_seek_anchor_and_meter_trial_can_be_reverted() {
        use super::Message;
        use crate::timeline::{PlaybackScroll, TimelineEvent, TimelineViewport};
        let mut app = super::CleanSlateApp {
            timeline: TimelineViewport::new(100 * crate::TICKS_PER_SECOND),
            ..Default::default()
        };
        for _ in 0..6 {
            app.timeline.zoom_at(0.0, 1000.0, 1.0);
        }
        app.timeline.seek(app.timeline.sample_at(250.0, 1000.0));
        let _ = app.update(Message::LockPlayhead);
        let _ = app.update(Message::Timeline(TimelineEvent::Seek(
            50 * crate::TICKS_PER_SECOND,
        )));
        assert!((app.timeline.x_for_sample(app.timeline.playhead(), 1000.0) - 250.0).abs() < 0.01);
        assert_eq!(app.playback_scroll, PlaybackScroll::Locked);
        let _ = app.update(Message::CenterPlayhead);
        assert!((app.timeline.x_for_sample(app.timeline.playhead(), 1000.0) - 500.0).abs() < 0.01);
        assert_eq!(app.playback_scroll, PlaybackScroll::Locked);
        let _ = app.update(Message::LockPlayhead);
        assert_eq!(app.playback_scroll, PlaybackScroll::Off);
        let _ = app.update(Message::CenterPlayhead);
        assert_eq!(app.playback_scroll, PlaybackScroll::Off);
        assert!(app.meters_at_playhead);
        let _ = app.update(Message::ToggleMeterPlacement);
        assert!(!app.meters_at_playhead);
        let _ = app.update(Message::ToggleMeterPlacement);
        assert!(app.meters_at_playhead);
        assert_eq!(super::ownership_meter_left(500.0, 1000.0, 170.0), 415.0);
        assert_eq!(super::ownership_meter_left(-10.0, 1000.0, 170.0), 0.0);
        assert_eq!(super::ownership_meter_left(1100.0, 1000.0, 170.0), 830.0);
        assert_eq!(super::ownership_meter_left(50.0, 100.0, 170.0), 0.0);
    }

    #[test]
    fn right_click_removes_only_the_pointed_track_mark_with_undo() {
        use super::Message;
        let root =
            std::env::temp_dir().join(format!("cleanslate-delete-region-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let sources = (0..2)
            .map(|i| {
                let path = root.join(format!("track-{i}.wav"));
                write_wav(&path, &vec![0.0; 48000]);
                path
            })
            .collect();
        let mut project = synchronized_project(root.join("session.cleanslate"), sources).unwrap();
        let range = crate::TimeRange::new(0, crate::TICKS_PER_SECOND / 2).unwrap();
        for (track, action) in [
            (0, crate::GainRegionAction::Preserve),
            (1, crate::GainRegionAction::Attenuate(0.1)),
        ] {
            let draft = project
                .region_gain_edit(project.sources[track].track_id, range, action)
                .unwrap();
            project.commit_gain_edit(draft).unwrap();
        }
        let before = project.gain_constraints();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        let time = crate::TICKS_PER_SECOND / 4;
        for track in 0..2 {
            let _ = app.update(Message::OpenGainMenu(track, time, iced::Point::ORIGIN));
            assert!(app.gain_menu.is_some());
            let _ = app.update(Message::DeleteRegion(track, time));
            let after = app.project.as_ref().unwrap().gain_constraints();
            assert!(!super::region_marked(
                &after,
                app.project.as_ref().unwrap().sources[track].track_id,
                time
            ));
            assert!(super::region_marked(
                &after,
                app.project.as_ref().unwrap().sources[1 - track].track_id,
                time
            ));
            let project = app.project.as_mut().unwrap();
            assert!(project.undo_gain_edit().unwrap());
            assert_eq!(project.gain_constraints(), before);
            assert!(project.redo_gain_edit().unwrap());
            assert_eq!(project.gain_constraints(), after);
            assert!(project.undo_gain_edit().unwrap());
        }
        let _ = app.update(Message::OpenGainMenu(0, range.end, iced::Point::ORIGIN));
        assert!(app.gain_menu.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn solo_buttons_combine_and_do_not_change_the_saved_project() {
        use super::Message;
        let root =
            std::env::temp_dir().join(format!("cleanslate-solo-buttons-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let sources = (0..3)
            .map(|i| {
                let path = root.join(format!("track-{i}.wav"));
                write_wav(&path, &[0.0; 512]);
                path
            })
            .collect();
        let path = root.join("session.cleanslate");
        let project = crate::Project::create(&path, sources).unwrap();
        let saved = std::fs::read(&path).unwrap();
        let constraints = project.gain_constraints();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        for (track, expected) in [(0, 1), (2, 5), (0, 4), (usize::MAX, 4), (2, 0)] {
            let _ = app.update(Message::SoloTrack(track));
            assert_eq!(app.solo_tracks, expected);
        }
        assert_eq!(
            app.project.as_ref().unwrap().gain_constraints(),
            constraints
        );
        assert_eq!(std::fs::read(path).unwrap(), saved);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn manual_pan_disables_playback_follow_and_follow_controls_invalidate_drawing() {
        let mut app = super::CleanSlateApp {
            timeline: crate::timeline::TimelineViewport::new(100 * crate::TICKS_PER_SECOND),
            ..Default::default()
        };
        app.timeline.zoom_at(0.0, 1000.0, 1.0);
        app.timeline.zoom_at(0.0, 1000.0, 1.0);
        app.timeline.seek(50 * crate::TICKS_PER_SECOND);
        let visible = app.timeline.visible_samples();
        let _ = app.update(super::Message::LockPlayhead);
        assert_eq!(app.timeline.start(), 0.0);
        assert_eq!(app.drawing_revision, 1);
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Pan {
                delta_x: 40.0,
                width: 1000.0,
            },
        ));
        assert_eq!(app.playback_scroll, crate::timeline::PlaybackScroll::Off);
        assert_eq!(app.timeline.visible_samples(), visible);
        assert_eq!(app.drawing_revision, 2);
        let start = app.timeline.start();
        let _ = app.update(super::Message::PlaybackFrame);
        assert_eq!(app.timeline.start(), start);
        assert_eq!(app.drawing_revision, 2);
    }

    #[test]
    fn playback_frames_reuse_drawing_without_polling_background_jobs() {
        let mut app = super::CleanSlateApp::default();
        app.timeline.seek(0);
        // A queued import would start on a job tick, but must not start on display frames.
        app.pending_imports
            .push(std::env::temp_dir().join("missing-frame-source.wav"));
        app.import_due = Some(std::time::Instant::now());
        for _ in 0..144 {
            let _ = app.update(super::Message::PlaybackFrame);
        }
        assert!(app.source_loading.is_none());
        assert_eq!(app.pending_imports.len(), 1);
        assert_eq!(app.drawing_revision, 0);
        assert_eq!(app.timeline.playhead(), 0);
        assert!(app.audition.is_none());

        app.pending_imports.clear();
        app.import_due = None;
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Select(0, 1),
        ));
        assert_eq!(app.drawing_revision, 1);
        let _ = app.update(super::Message::Attenuation(-12.0));
        assert_eq!(app.drawing_revision, 2);
        let _ = app.update(super::Message::Tick);
        assert_eq!(app.drawing_revision, 3);
        let _ = app.update(super::Message::PlaybackFrame);
        assert_eq!(app.drawing_revision, 3);
    }

    #[test]
    fn ownership_meter_reports_solo_overlap_silence_and_uncertainty() {
        use crate::TemporalTrackState::{Absent, Owned, UncertainUnity};
        let speakers = [crate::SpeakerId(3), crate::SpeakerId(7)];
        assert_eq!(
            super::ownership_summary(&speakers, &[Owned, Absent]),
            "Active: Speaker 3"
        );
        assert_eq!(
            super::ownership_summary(&speakers, &[Owned, Owned]),
            "Overlap: Speaker 3 + Speaker 7"
        );
        assert_eq!(
            super::ownership_summary(&speakers, &[Absent, Absent]),
            "No active speaker"
        );
        assert_eq!(
            super::ownership_summary(&speakers, &[Absent, UncertainUnity]),
            "Uncertain ownership"
        );
        assert_eq!(
            super::ownership_summary(&speakers, &[Owned, UncertainUnity]),
            "Active: Speaker 3 · other ownership uncertain"
        );
    }

    #[test]
    fn development_startup_uses_the_normal_project_loader() {
        assert!(super::startup(None).source_loading.is_none());
        let missing = std::env::temp_dir().join(format!(
            "cleanslate-missing-dev-project-{}.cleanslate",
            std::process::id()
        ));
        assert!(!missing.exists());
        let mut app = super::startup(Some(missing.clone()));
        assert!(app.source_loading.is_some());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while app.source_loading.is_some() && std::time::Instant::now() < deadline {
            app.poll_sources();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(app.source_loading.is_none());
        assert!(app.project.is_none());
        assert!(!app.messages.is_empty());
        assert!(app.processing.is_none());
        assert!(!missing.exists());
    }

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
            app.calibration_timing,
            "Popup keeps the saved-example list visible"
        );
        let captured_range = app.calibration_selection;
        let _ = app.update(Message::Timeline(TimelineEvent::Select(0, 1)));
        assert_eq!(app.calibration_selection, captured_range);
        assert_eq!(app.reference_speakers, vec![speakers[0]]);
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Character("2".into()),
            iced::keyboard::Modifiers::empty(),
        ));
        assert_eq!(app.reference_speakers, vec![speakers[0], speakers[1]]);
        assert!(
            app.reference_menu.is_some()
                && app.project.as_ref().unwrap().reference_labels().is_empty()
        );
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Character("2".into()),
            iced::keyboard::Modifiers::empty(),
        ));
        assert_eq!(app.reference_speakers, vec![speakers[0]]);
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Character("r".into()),
            iced::keyboard::Modifiers::empty(),
        ));
        assert!(app.reference_speakers.is_empty() && app.reference_menu.is_some());
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Character("1".into()),
            iced::keyboard::Modifiers::empty(),
        ));
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
    fn imported_tracks_report_reference_setup_without_required_model() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-ui-status-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let source = directory.join("voice.wav");
        write_wav(&source, &vec![0.0; 48000]);
        let app = super::CleanSlateApp {
            project: Some(
                crate::Project::create(directory.join("session.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        assert_eq!(app.workspace_status(), "ADD SPEAKER / ROOM REFERENCES");
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
        project.set_nemotron_enabled(true)?;
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
    fn timeline_examples_reject_changed_sources() {
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
        app.capture_calibration_range(
            crate::timebase::detector_time(100),
            crate::timebase::detector_time(30_000),
        );
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
                start: crate::timebase::detector_time(1),
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
            started_at: std::time::Instant::now(),
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

    #[test]
    fn multichannel_picker_selects_names_mixes_mono_and_preserves_session_on_cancel_or_failure() {
        let root = crate::audio_io::multichannel_tests::root();
        let poly = root.join("poly.wav");
        crate::audio_io::multichannel_tests::write_poly(&poly, 3, 32, 4800);
        let mono = root.join("mono.wav");
        write_wav(&mono, &[0.125; 4800]);
        let mut app = super::CleanSlateApp::default();
        app.add_stems([poly.clone(), mono]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while app.import_choices.is_empty() {
            app.import_due = Some(std::time::Instant::now());
            app.poll_sources();
            assert!(
                std::time::Instant::now() < deadline,
                "picker did not appear"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(app.import_choices.len(), 4);
        assert!(app.import_choices.iter().all(|c| c.selected));
        assert!(app.project.is_none());
        let omitted = app
            .import_choices
            .iter()
            .position(|c| c.source.channel == Some(1))
            .unwrap();
        let renamed = app
            .import_choices
            .iter()
            .position(|c| c.source.channel == Some(2))
            .unwrap();
        let _ = app.update(super::Message::ImportChannel(omitted, false));
        let _ = app.update(super::Message::ImportChannelName(renamed, "Alice".into()));
        let _ = app.update(super::Message::ConfirmChannels);
        finish_import(&mut app);
        assert_eq!(app.waveforms.len(), 3);
        let project = app.project.as_ref().unwrap();
        assert_eq!(project.sources.len(), 3);
        assert!(
            project
                .sources
                .iter()
                .any(|s| s.channel == Some(2) && s.display_name() == "Alice")
        );
        assert!(
            project
                .sources
                .iter()
                .all(|s| s.metadata.grid.frame_count == 4800)
        );
        assert!(
            project.require_alignment().is_err(),
            "mixed files still need synchronization confirmation"
        );
        let saved = std::fs::read(project.path()).unwrap();
        app.import_choices = crate::wav_channel_sources(&poly)
            .unwrap()
            .into_iter()
            .map(|source| super::ImportChoice {
                source,
                selected: true,
            })
            .collect();
        let _ = app.update(super::Message::CancelChannels);
        assert_eq!(
            std::fs::read(app.project.as_ref().unwrap().path()).unwrap(),
            saved
        );
        let bad = root.join("different-duration.wav");
        write_wav(&bad, &[0.0; 16]);
        app.add_stems([bad]);
        finish_import(&mut app);
        assert_eq!(
            std::fs::read(app.project.as_ref().unwrap().path()).unwrap(),
            saved
        );
        assert_eq!(app.waveforms.len(), 3);
        assert!(app.messages[0].contains("Import blocked"));
        app.stop_audition();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "reads the owner's full 1.85 GB Zoom poly WAV"]
    fn full_multichannel_zoom_import() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-audio/261005-T001.WAV");
        let before = crate::project::fingerprint_cancellable(&path, None).unwrap();
        let inputs = crate::wav_channel_sources(&path).unwrap();
        assert_eq!(
            inputs
                .iter()
                .map(crate::AudioSource::display_name)
                .collect::<Vec<_>>(),
            ["Tr5", "Tr6", "Tr7"]
        );
        let root = crate::audio_io::multichannel_tests::root();
        let project =
            crate::Project::create_from_sources(root.join("zoom.cleanslate"), inputs).unwrap();
        let (sender, events) = std::sync::mpsc::channel();
        let started = std::time::Instant::now();
        let loaded = super::load_sources(
            super::SourceRequest::Validate,
            Some(project),
            crate::AutomixParameters::default(),
            &sender,
        )
        .unwrap();
        let waveforms: Vec<_> = events
            .try_iter()
            .filter_map(|event| {
                if let super::SourceEvent::Waveform(_, waveform) = event {
                    Some(waveform)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(waveforms.len(), 3);
        for waveform in &waveforms {
            assert_eq!(waveform.sample_rate, 48_000);
            assert_eq!(waveform.sample_count, 154_128_376);
        }
        loaded.project.require_alignment().unwrap();
        crate::Project::open(loaded.project.path())
            .unwrap()
            .validate_sources()
            .unwrap();
        assert_eq!(
            crate::project::fingerprint_cancellable(&path, None).unwrap(),
            before
        );
        eprintln!(
            "Zoom full import, waveforms, validation and reopen: {:.2}s",
            started.elapsed().as_secs_f64()
        );
        std::fs::remove_dir_all(root).unwrap();
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
        assert!(app.source_loading.is_some() || app.source_inspection.is_some());
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
        app.calibration_selection = Some((2 * crate::TICKS_PER_SECOND, crate::TICKS_PER_SECOND));
        assert!(app.calibration_range().is_err());
        app.calibration_selection =
            Some((100 * crate::TICKS_PER_SECOND, 101 * crate::TICKS_PER_SECOND));
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
    fn gap_control_persists_without_starting_inference_or_changing_manual_edits() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-gap-ui-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("voice.wav");
        write_wav(&source, &[0.0; 100]);
        let project =
            synchronized_project(directory.join("episode.cleanslate"), vec![source]).unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        app.timeline.set_selection(Some((10, 40)));
        let _ = app.update(super::Message::GainAction(0, false));
        let constraints = app.project.as_ref().unwrap().gain_constraints();
        for value in [10.0, 5000.0, 768.0] {
            let _ = app.update(super::Message::MinimumGap(value));
            assert_eq!(
                app.project.as_ref().unwrap().parameters().minimum_gap_ms,
                value
            );
            assert_eq!(
                app.project.as_ref().unwrap().gain_constraints(),
                constraints
            );
            assert!(!app.reference_analyze_requested);
            assert!(app.preview_processing.is_none());
        }
        let reopened = crate::Project::open(app.project.as_ref().unwrap().path()).unwrap();
        assert_eq!(reopened.parameters().minimum_gap_ms, 768.0);
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
    source_inspection: Option<SourceInspection>,
    import_choices: Vec<ImportChoice>,
    track_name_draft: Option<(usize, String)>,
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
    calibration_selection: Option<(TimePosition, TimePosition)>,
    calibration_error: Option<String>,
    reference_speakers: Vec<SpeakerId>,
    selection_track: Option<usize>,
    reference_menu: Option<(usize, iced::Point)>,
    selected_example: Option<u64>,
    example_draft: Option<TimeRange>,
    gain_menu: Option<(usize, u64, iced::Point)>,
    model_details: bool,
    setup_details: bool,
    reference_stale: bool,
    reference_analyze_requested: bool,
    waveforms: Vec<WaveformCache>,
    timeline: TimelineViewport,
    playback_scroll: PlaybackScroll,
    meters_at_playhead: bool,
    solo_tracks: u64,
    drawing_revision: u64,
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
            source_inspection: None,
            import_choices: vec![],
            track_name_draft: None,
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
            calibration_selection: None,
            calibration_error: None,
            reference_speakers: vec![],
            selection_track: None,
            reference_menu: None,
            selected_example: None,
            example_draft: None,
            gain_menu: None,
            model_details: false,
            setup_details: false,
            reference_stale: false,
            reference_analyze_requested: false,
            waveforms: vec![],
            timeline: TimelineViewport::default(),
            playback_scroll: PlaybackScroll::default(),
            meters_at_playhead: true,
            solo_tracks: 0,
            drawing_revision: 0,
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
    AddChannels(Vec<crate::AudioSource>),
    Open(PathBuf),
    Relink(usize, PathBuf),
    Validate,
}

struct ImportChoice {
    source: crate::AudioSource,
    selected: bool,
}

struct SourceInspection {
    paths: Vec<PathBuf>,
    result: Receiver<Result<Vec<crate::AudioSource>, String>>,
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
        let mut source = project
            .as_ref()
            .and_then(|p| p.sources.get(*index))
            .ok_or("Unknown source track")?
            .audio_source();
        source.path = path.clone();
        crate::validate_audio_sources(&[source])
            .map_err(|error| format!("Import blocked: {error}"))?;
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
    let mut inputs = project
        .as_ref()
        .map_or_else(Vec::new, Project::audio_sources);
    match &request {
        SourceRequest::AddChannels(additions) => inputs.extend(additions.iter().cloned()),
        SourceRequest::Relink(index, path) => {
            inputs
                .get_mut(*index)
                .ok_or("Source track is not in this session.")?
                .path = path.clone();
        }
        _ => {}
    }
    let paths: Vec<_> = inputs.iter().map(|s| s.path.clone()).collect();
    let _ = sender.send(SourceEvent::Paths(paths.clone()));
    if paths.is_empty() {
        return Err("No source tracks were selected.".into());
    }
    if inputs.len() > 8 {
        return Err("Import blocked: session requires one to eight speaker tracks.".into());
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
    let waveform_result = waveform::load_source_waveforms(&inputs, |index, waveform| {
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
        warnings = crate::validate_audio_sources(&inputs)
            .map_err(|error| format!("Import blocked: {error}"))?
            .warnings;
    }
    let _ = sender.send(SourceEvent::Stage("Saving source identities"));
    match request {
        SourceRequest::AddChannels(additions) => {
            if let Some(project) = &mut project {
                project
                    .add_audio_sources(additions)
                    .map_err(|error| error.to_string())?;
            } else {
                let mut created = Project::create_from_sources(project_path, additions)
                    .map_err(|error| error.to_string())?;
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
    started_at: Instant,
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
    ImportChannel(usize, bool),
    ImportChannelName(usize, String),
    ConfirmChannels,
    CancelChannels,
    EditTrackName(usize),
    TrackName(String),
    SaveTrackName,
    CancelTrackName,
    Example(ExampleEvent),
    ShowExample(u64),
    PlayExample(u64),
    TrackTimeline(usize, TimelineEvent),
    CompleteTrackSelection(usize, TimePosition, TimePosition, iced::Point),
    OpenReferenceMenu(usize, iced::Point),
    OpenTimelineMenu(usize, u64, iced::Point),
    OpenGainMenu(usize, u64, iced::Point),
    DeleteRegion(usize, u64),
    DismissReferenceMenu,
    ContextReference(Vec<SpeakerId>, bool),
    ModelDetails(bool),
    NemotronEnabled(bool),
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
    PlaybackFrame,
    LockPlayhead,
    CenterPlayhead,
    ToggleMeterPlacement,
    SoloTrack(usize),
    SetupDetails(bool),
    #[cfg(test)]
    SpeechSensitivity(f32),
    Attenuation(f32),
    #[cfg(test)]
    SpeechPadding(f32),
    OpenTime(f32),
    CloseTime(f32),
    MinimumGap(f32),
    RefreshPreview,
    Original,
    Processed,
    Removed,
    Audition,
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
    #[cfg(test)]
    SelectSupervisedWarning(usize),
    SaveProject,
    ProtectSelection,
    Export,
    Timeline(TimelineEvent),
}

pub fn run() -> iced::Result {
    iced::application(
        || {
            #[cfg(debug_assertions)]
            let project = std::env::var_os("CLEANSLATE_DEV_PROJECT").map(PathBuf::from);
            #[cfg(not(debug_assertions))]
            let project = None;
            startup(project)
        },
        CleanSlateApp::update,
        CleanSlateApp::view,
    )
    .title(
        if cfg!(debug_assertions) && std::env::var_os("CLEANSLATE_DEV_PROJECT").is_some() {
            "CleanSlate - GUI development"
        } else {
            "CleanSlate"
        },
    )
    .theme(CleanSlateApp::theme)
    .subscription(CleanSlateApp::subscription)
    .window(window::Settings {
        maximized: true,
        min_size: Some(iced::Size::new(960.0, 900.0)),
        ..window::Settings::default()
    })
    .run()
}

fn startup(project: Option<PathBuf>) -> CleanSlateApp {
    let mut app = CleanSlateApp::default();
    if let Some(path) = project {
        app.begin_source_loading(SourceRequest::Open(path));
    }
    app
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
    fn analysis_runtime(&self) -> Option<&crate::model::RuntimeProvenance> {
        self.project
            .as_ref()
            .filter(|project| project.nemotron_enabled())
            .and_then(|_| self.model_acquisition.provenance())
    }
    fn install_waveforms(&mut self, waveforms: Vec<WaveformCache>) {
        self.selected_example = None;
        self.example_draft = None;
        self.gain_menu = None;
        self.reference_menu = None;
        self.selection_track = None;
        self.calibrating = false;
        self.calibration_selection = None;
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
                Ok(_) => self
                    .audition_preparation
                    .request_sources(project.audio_sources()),
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
                }) || self.project.as_ref().is_some_and(|p| {
                    p.sources.iter().any(|s| {
                        s.channel.is_some()
                            && s.path
                                .to_string_lossy()
                                .eq_ignore_ascii_case(&path.to_string_lossy())
                    })
                })
            })
            .collect();
        additions.sort_by_key(|path| path.to_string_lossy().to_lowercase());
        additions.dedup_by(|a, b| {
            a.to_string_lossy()
                .eq_ignore_ascii_case(&b.to_string_lossy())
        });
        if additions.is_empty() {
            self.messages = vec!["No new supported audio tracks were selected".into()];
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
        self.source_loading.is_some()
            || self.source_inspection.is_some()
            || !self.pending_imports.is_empty()
            || !self.import_choices.is_empty()
    }

    fn loading_paths(&self) -> Vec<PathBuf> {
        if let Some(inspection) = &self.source_inspection {
            let mut paths = inspection.paths.clone();
            paths.extend(self.pending_imports.iter().cloned());
            return paths;
        }
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
            SourceRequest::AddChannels(additions) => {
                paths.extend(additions.iter().map(|s| s.path.clone()))
            }
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
        if let Some(inspection) = &self.source_inspection {
            let result = match inspection.result.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Source inspection stopped unexpectedly.".into()))
                }
            };
            if let Some(result) = result {
                self.source_inspection = None;
                match result {
                    Ok(inputs) if inputs.iter().any(|s| s.channel.is_some()) => {
                        self.import_choices = inputs
                            .into_iter()
                            .map(|mut source| {
                                if source.name.is_none() {
                                    source.name = Some(source.display_name());
                                }
                                ImportChoice {
                                    selected: !self.project.as_ref().is_some_and(|p| {
                                        p.sources.iter().any(|s| {
                                            s.path == source.path && s.channel == source.channel
                                        })
                                    }),
                                    source,
                                }
                            })
                            .collect();
                    }
                    Ok(inputs) => self.begin_source_loading(SourceRequest::AddChannels(inputs)),
                    Err(error) => self.messages = vec![error],
                }
            }
        }
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
                            self.solo_tracks = 0;
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
        if self.source_loading.is_none()
            && self.source_inspection.is_none()
            && self.import_choices.is_empty()
            && self.import_due.is_some_and(|due| Instant::now() >= due)
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
                    }) || self.project.as_ref().is_some_and(|p| {
                        p.sources.iter().any(|s| {
                            s.channel.is_some()
                                && s.path
                                    .to_string_lossy()
                                    .eq_ignore_ascii_case(&path.to_string_lossy())
                        })
                    })
                })
                .collect();
            if !additions.is_empty() {
                let (sender, result) = mpsc::channel();
                let mut paths = self
                    .project
                    .as_ref()
                    .map_or_else(Vec::new, Project::source_paths);
                paths.extend(additions.iter().cloned());
                std::thread::spawn(move || {
                    let inputs = additions
                        .into_iter()
                        .map(|path| {
                            let metadata =
                                crate::SourceMetadata::read(&path).map_err(|e| e.to_string())?;
                            if metadata.channels > 1 {
                                crate::wav_channel_sources(&path).map_err(|e| e.to_string())
                            } else {
                                Ok(vec![crate::AudioSource::mono(path)])
                            }
                        })
                        .collect::<Result<Vec<_>, String>>()
                        .map(|inputs| inputs.into_iter().flatten().collect());
                    let _ = sender.send(inputs);
                });
                self.source_inspection = Some(SourceInspection { paths, result });
            } else {
                self.start_preview();
            }
        }
    }

    fn import_stems(&mut self) {
        let Some(paths) = rfd::FileDialog::new()
            .set_directory(dialog_start_directory())
            .add_filter(
                "Audio / multichannel WAV",
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
        let (Some(project), Some(analysis), Some(effective)) = (
            &self.project,
            &self.supervised_result,
            &self.effective_result,
        ) else {
            self.messages = vec!["Final export requires current supervised analysis.".into()];
            return;
        };
        let runtime = self.analysis_runtime();
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
        let runtime = runtime.cloned();
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
                if let Some(runtime) = &runtime {
                    crate::verify_installed_editor_runtime(runtime, &worker_cancel)?;
                }
                let private = crate::prepare_supervised_render(
                    &project,
                    &analysis,
                    &effective,
                    &worker_snapshot,
                    &destination,
                    &worker_progress,
                    Some(&worker_cancel),
                )?;
                if let Some(runtime) = &runtime {
                    crate::verify_installed_editor_runtime(runtime, &worker_cancel)?;
                }
                private.publish(
                    &project,
                    runtime.as_ref(),
                    &gate,
                    generation,
                    Some(&worker_cancel),
                )
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
        let runtime = self.analysis_runtime().cloned();
        let project = project.clone();
        let gate = self.render_generation.clone();
        let generation = gate.current();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancelled.clone();
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let completed = (|| {
                if let Some(runtime) = &runtime {
                    crate::verify_installed_editor_runtime(runtime, &worker_cancel)?;
                }
                let private = crate::prepare_export(&project, &destination, Some(&worker_cancel))?;
                if let Some(runtime) = &runtime {
                    crate::verify_installed_editor_runtime(runtime, &worker_cancel)?;
                }
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
            && self.supervised_result.as_ref().is_some_and(|result| self.analysis_runtime() == result.runtime.as_ref())
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
            let sources = self.project.as_ref().unwrap().source_stamps();
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
        let sources = match project.source_stamps() {
            Ok(sources) => sources,
            Err(error) => {
                self.preview_error = Some(error.to_string());
                return;
            }
        };
        let source_project = project.clone();
        let cache_directory = project.path().with_extension("analysis-cache");
        let runtime = self.analysis_runtime().cloned();
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
            started_at: Instant::now(),
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
                    || self.analysis_runtime() != analysis.runtime.as_ref();
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
            self.audition_preparation
                .request_sources(project.audio_sources());
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
        if self.gain_menu.is_some() {
            if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                self.gain_menu = None;
            }
            return true;
        }
        if self.reference_menu.is_some() {
            let message = match key.as_ref() {
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    Some(Message::DismissReferenceMenu)
                }
                keyboard::Key::Named(keyboard::key::Named::Enter) => Some(
                    Message::ContextReference(self.reference_speakers.clone(), false),
                ),
                keyboard::Key::Character("r" | "R") => {
                    self.reference_speakers.clear();
                    None
                }
                keyboard::Key::Character(number) => number
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| index.checked_sub(1))
                    .and_then(|index| self.project.as_ref()?.sources.get(index))
                    .map(|source| {
                        Message::ReferenceSpeaker(
                            source.speaker_id,
                            !self.reference_speakers.contains(&source.speaker_id),
                        )
                    }),
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
                                if project.nemotron_enabled() {
                                    self.model_acquisition.provenance()
                                } else {
                                    None
                                },
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
            && self.analysis_runtime() == processing.snapshot.runtime.as_ref()
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
                                    if prepared.is_current_sources(&project.audio_sources())
                                        && project.source_stamps().is_ok_and(|sources| {
                                            prepared.matches_sources(&sources)
                                        }) =>
                                {
                                    match LiveAudition::start(
                                        prepared,
                                        envelopes,
                                        self.audition_mode,
                                        10.0_f32.powf(self.monitor_db / 20.0),
                                        start,
                                        end,
                                        self.solo_tracks,
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
        self.update_playback_position();
    }

    fn update_playback_position(&mut self) {
        let Some(audition) = &self.audition else {
            return;
        };
        self.timeline.seek(audition.position());
        if self.timeline.follow_playhead(self.playback_scroll) {
            self.drawing_revision = self.drawing_revision.wrapping_add(1);
        }
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
        self.timeline.follow_playhead(self.playback_scroll);
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
    fn select_example(&mut self, id: u64) {
        self.example_draft = None;
        let Some(project) = &self.project else {
            return;
        };
        let Some(label) = project
            .reference_labels()
            .iter()
            .find(|label| label.id == id)
        else {
            self.selected_example = None;
            self.timeline.set_selection(None);
            return;
        };
        let range = label.range;
        self.reference_speakers = label.active_speakers.clone();
        self.selection_track = Some(0);
        self.selected_example = Some(id);
        self.capture_calibration_range(range.start, range.end);
        self.timeline.set_selection(Some((range.start, range.end)));
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
                    Some(label.id) != self.selected_example
                        && label.range.start < end
                        && start < label.range.end
                        && active != previous
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
            let project = self
                .project
                .as_mut()
                .ok_or("Import sources before labeling.")?;
            if let Some(id) = self.selected_example {
                let mut labels = project.reference_labels().to_vec();
                labels.retain(|label| !replacing.contains(&label.id));
                let label = labels
                    .iter_mut()
                    .find(|label| label.id == id)
                    .ok_or("Example no longer exists.")?;
                label.range = TimeRange { start, end };
                label.active_speakers = self.reference_speakers.clone();
                label.active_speakers.sort_unstable();
                project
                    .replace_reference_labels(labels)
                    .map(|()| id)
                    .map_err(|error| error.to_string())
            } else {
                project
                    .add_reference_label(
                        TimeRange { start, end },
                        self.reference_speakers.clone(),
                        &replacing,
                    )
                    .map_err(|error| error.to_string())
            }
        })();
        match result {
            Ok(_) => self.mark_reference_edit(),
            Err(error) => self.calibration_error = Some(error),
        }
    }

    fn capture_calibration_range(&mut self, start: TimePosition, end: TimePosition) {
        self.calibration_selection = Some((start.min(end), start.max(end)));
        self.calibration_error = None;
    }

    fn calibration_range(&self) -> Result<(TimePosition, TimePosition), String> {
        let (start, end) = self
            .calibration_selection
            .ok_or("Drag across a waveform to select an example.")?;
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
        if matches!(message, Message::PlaybackFrame) {
            self.update_playback_position();
            return Task::none();
        }
        // Playback changes only the cursor; controller/job updates can change cached geometry.
        self.drawing_revision = self.drawing_revision.wrapping_add(1);
        if (self.reference_menu.is_some() || self.gain_menu.is_some())
            && matches!(
                message,
                Message::TrackTimeline(_, _)
                    | Message::CompleteTrackSelection(_, _, _, _)
                    | Message::Timeline(_)
                    | Message::GainEvent(_, _)
                    | Message::Example(_)
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
                    | Message::ImportChannel(_, _)
                    | Message::ImportChannelName(_, _)
                    | Message::ConfirmChannels
                    | Message::CancelChannels
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
                | Message::NemotronEnabled(_)
                | Message::CancelModelDownload
                | Message::Attenuation(_)
                | Message::OpenTime(_)
                | Message::CloseTime(_)
                | Message::MinimumGap(_)
                | Message::RefreshPreview
                | Message::GainEvent(_, _)
                | Message::GainAction(_, _)
                | Message::UndoGain
                | Message::RedoGain
                | Message::DeleteGain
                | Message::InsertGainPoint
                | Message::SaveReference(_)
                | Message::Example(ExampleEvent::Resize(_, _, true))
                | Message::RemoveReference(_)
                | Message::UndoReferences
                | Message::RedoReferences
                | Message::SuppliedAlignment(_)
                | Message::RebindSource(_)
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
            Message::ImportChannel(index, selected) => {
                if let Some(choice) = self.import_choices.get_mut(index) {
                    choice.selected = selected;
                }
            }
            Message::ImportChannelName(index, name) => {
                if let Some(choice) = self.import_choices.get_mut(index) {
                    choice.source.name = Some(name);
                }
            }
            Message::ConfirmChannels => {
                let additions: Vec<_> = self
                    .import_choices
                    .iter()
                    .filter(|c| c.selected)
                    .filter(|c| {
                        !self.project.as_ref().is_some_and(|p| {
                            p.sources
                                .iter()
                                .any(|s| s.path == c.source.path && s.channel == c.source.channel)
                        })
                    })
                    .map(|c| c.source.clone())
                    .collect();
                let count = self.project.as_ref().map_or(0, |p| p.sources.len()) + additions.len();
                if additions.is_empty() || count > 8 {
                    self.messages = vec![
                        "Select at least one channel, with at most eight tracks in the project."
                            .into(),
                    ];
                } else if additions.iter().any(|s| {
                    s.name.as_ref().is_some_and(|name| {
                        name.trim().is_empty()
                            || name.len() > 256
                            || name.chars().any(char::is_control)
                    })
                }) {
                    self.messages = vec![
                        "Track names require 1 to 256 bytes without control characters.".into(),
                    ];
                } else {
                    self.import_choices.clear();
                    self.messages.clear();
                    self.begin_source_loading(SourceRequest::AddChannels(additions));
                }
            }
            Message::CancelChannels => {
                self.import_choices.clear();
                self.messages.clear();
            }
            Message::EditTrackName(index) => {
                if let Some(source) = self.project.as_ref().and_then(|p| p.sources.get(index)) {
                    self.track_name_draft = Some((index, source.display_name()));
                }
            }
            Message::TrackName(name) => {
                if let Some((_, draft)) = &mut self.track_name_draft {
                    *draft = name;
                }
            }
            Message::SaveTrackName => {
                if let Some((index, name)) = self.track_name_draft.clone() {
                    match self.project.as_mut().unwrap().rename_track(index, name) {
                        Ok(()) => {
                            self.track_name_draft = None;
                            self.render_generation.invalidate();
                            self.mark_reference_edit();
                        }
                        Err(error) => self.messages = vec![error.to_string()],
                    }
                }
            }
            Message::CancelTrackName => self.track_name_draft = None,
            Message::ShowExample(id) | Message::PlayExample(id) => {
                let play = matches!(message, Message::PlayExample(_));
                self.stop_audition();
                self.select_example(id);
                if self.selected_example == Some(id) {
                    if let Some((start, end)) = self.timeline.selection() {
                        self.timeline.show_range(start, end);
                        self.playback_scroll = PlaybackScroll::Off;
                        self.timeline.seek(start);
                    }
                    if play {
                        self.audition(true);
                    }
                }
            }
            Message::Example(ExampleEvent::Select(id)) => self.select_example(id),
            Message::Example(ExampleEvent::CancelResize) => {
                self.example_draft = None;
                if let Some(id) = self.selected_example {
                    self.select_example(id);
                }
            }
            Message::Example(ExampleEvent::Resize(id, range, commit)) => {
                if self.selected_example == Some(id) {
                    if commit {
                        let result = (|| {
                            if range.end <= range.start
                                || range.end - range.start < TICKS_PER_SECOND / 4
                            {
                                return Err("Select at least 0.25 seconds.".to_owned());
                            }
                            let project = self
                                .project
                                .as_mut()
                                .ok_or("Import sources before labeling.")?;
                            let mut labels = project.reference_labels().to_vec();
                            let label = labels
                                .iter_mut()
                                .find(|label| label.id == id)
                                .ok_or("Example no longer exists.")?;
                            label.range = range;
                            project
                                .replace_reference_labels(labels)
                                .map_err(|error| error.to_string())
                        })();
                        self.example_draft = None;
                        self.select_example(id);
                        match result {
                            Ok(()) => self.mark_reference_edit(),
                            Err(error) => self.calibration_error = Some(error),
                        }
                    } else {
                        if self.example_draft.is_none() {
                            self.stop_audition();
                            self.playback_scroll = PlaybackScroll::Off;
                        }
                        self.example_draft = Some(range);
                        self.timeline.set_selection(Some((range.start, range.end)));
                    }
                }
            }
            Message::CompleteTrackSelection(track, start, end, point) => {
                let _ = self.update(Message::TrackTimeline(
                    track,
                    TimelineEvent::Select(start, end),
                ));
                return self.update(Message::OpenReferenceMenu(track, point));
            }
            Message::TrackTimeline(track, event) => {
                self.selected_example = None;
                self.example_draft = None;
                self.gain_menu = None;
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
                self.selected_example = None;
                self.example_draft = None;
                self.gain_menu = None;
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
                    self.reference_menu = Some((track, point));
                }
            }
            Message::OpenTimelineMenu(track, time, point) => {
                self.reference_menu = None;
                if let Some(id) = self.project.as_ref().and_then(|project| {
                    project
                        .reference_labels()
                        .iter()
                        .rev()
                        .find(|label| label.range.start <= time && time < label.range.end)
                        .map(|label| label.id)
                }) {
                    self.select_example(id);
                    self.selection_track = Some(track);
                    self.gain_menu = None;
                    self.reference_menu = Some((track, point));
                    return Task::none();
                }
                return self.update(Message::OpenGainMenu(track, time, point));
            }
            Message::OpenGainMenu(track, time, point) => {
                self.reference_menu = None;
                self.gain_menu = self.project.as_ref().and_then(|project| {
                    let track_id = project.sources.get(track)?.track_id;
                    region_marked(&project.gain_constraints(), track_id, time)
                        .then_some((track, time, point))
                });
                if self.gain_menu.is_none() {
                    return self.update(Message::OpenReferenceMenu(track, point));
                }
            }
            Message::DeleteRegion(track, time) => {
                self.gain_menu = None;
                if let Some(project) = &mut self.project
                    && let Some(source) = project.sources.get(track)
                {
                    let track_id = source.track_id;
                    let mut constraints = project.gain_constraints();
                    constraints
                        .manual_edits
                        .retain(|edit| edit.source.track != track_id || !edit_contains(edit, time));
                    constraints.protections.retain(|protection| {
                        protection.track != Some(track_id)
                            || time < protection.range.start
                            || time >= protection.range.end
                    });
                    self.selected_gain_point = None;
                    self.preview_error = project
                        .replace_gain_constraints(constraints)
                        .err()
                        .map(|error| error.to_string());
                    self.rebuild_effective();
                }
            }
            Message::DismissReferenceMenu => {
                self.reference_menu = None;
                self.gain_menu = None;
            }
            Message::ModelDetails(show) => self.model_details = show,
            Message::SetupDetails(show) => self.setup_details = show,
            Message::LockPlayhead => {
                if self.playback_scroll == PlaybackScroll::Locked {
                    self.playback_scroll = PlaybackScroll::Off;
                } else {
                    self.timeline.lock_playhead();
                    self.playback_scroll = PlaybackScroll::Locked;
                    self.timeline.follow_playhead(self.playback_scroll);
                }
            }
            Message::CenterPlayhead => self.timeline.center_playhead(),
            Message::ToggleMeterPlacement => self.meters_at_playhead = !self.meters_at_playhead,
            Message::SoloTrack(track) => {
                if self
                    .project
                    .as_ref()
                    .is_some_and(|p| track < p.sources.len())
                {
                    self.solo_tracks ^= 1u64 << track;
                    if let Some(audition) = &self.audition {
                        audition.set_solo(self.solo_tracks);
                    }
                }
            }
            Message::NemotronEnabled(enabled) => {
                if let Some(project) = &mut self.project {
                    match project.set_nemotron_enabled(enabled) {
                        Ok(()) => {
                            self.mark_reference_edit();
                            if let Some(processing) = &self.preview_processing {
                                processing.cancelled.store(true, Ordering::Relaxed);
                            }
                        }
                        Err(error) => {
                            self.preview_error = Some(format!("Analysis option not saved: {error}"))
                        }
                    }
                }
            }
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
            Message::PlaybackFrame => unreachable!(),
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
            Message::MinimumGap(value) => self.preview_parameters.minimum_gap_ms = value,
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
                        Ok(()) => {
                            if self.selected_example == Some(id) {
                                self.selected_example = None;
                                self.example_draft = None;
                                self.timeline.set_selection(None);
                            }
                            self.reference_menu = None;
                            self.mark_reference_edit();
                        }
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
                        Ok(true) => {
                            self.example_draft = None;
                            if let Some(id) = self.selected_example {
                                self.select_example(id);
                            }
                            self.mark_reference_edit();
                        }
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
            #[cfg(test)]
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
                    self.timeline.follow_playhead(self.playback_scroll);
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
                } => {
                    self.timeline.zoom_at(cursor_x, width, direction);
                    self.timeline.follow_playhead(self.playback_scroll);
                }
                TimelineEvent::Pan { delta_x, width } => {
                    self.playback_scroll = PlaybackScroll::Off;
                    self.timeline.pan_by_pixels(delta_x, width)
                }
                TimelineEvent::Select(start, end) => {
                    self.selected_example = None;
                    self.example_draft = None;
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
        let playback = if self.audition.is_some() {
            window::frames().map(|_| Message::PlaybackFrame)
        } else {
            Subscription::none()
        };
        Subscription::batch([drops, processing, playback])
    }

    fn view(&self) -> Element<'_, Message> {
        if !self.import_choices.is_empty() {
            let mut choices = column![text("Choose speaker channels").size(24), text("Keep the microphone channels you want to edit. All channels keep their recording timing.")].spacing(12);
            choices = choices.push(
                text(format!(
                    "{} selected · {} existing tracks · maximum 8",
                    self.import_choices.iter().filter(|c| c.selected).count(),
                    self.project.as_ref().map_or(0, |p| p.sources.len())
                ))
                .size(12),
            );
            for (index, choice) in self.import_choices.iter().enumerate() {
                choices = choices.push(
                    row![
                        iced::widget::checkbox(choice.selected)
                            .label(choice.source.channel.map_or_else(
                                || "Mono".into(),
                                |channel| format!("Channel {}", channel + 1)
                            ))
                            .on_toggle(move |selected| Message::ImportChannel(index, selected)),
                        text_input("Track name", choice.source.name.as_deref().unwrap_or(""))
                            .on_input(move |name| Message::ImportChannelName(index, name))
                            .width(250),
                        text(
                            choice
                                .source
                                .path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        )
                        .size(12),
                    ]
                    .spacing(12),
                );
            }
            for message in &self.messages {
                choices = choices.push(text(message));
            }
            choices = choices.push(
                row![
                    button("Import selected channels").on_press(Message::ConfirmChannels),
                    button("Cancel")
                        .style(button::secondary)
                        .on_press(Message::CancelChannels)
                ]
                .spacing(12),
            );
            return container(scrollable(choices))
                .padding(24)
                .width(Fill)
                .height(Fill)
                .into();
        }
        if let Some((_, name)) = &self.track_name_draft {
            let mut form = column![
                text("Rename track").size(24),
                text_input("Track name", name).on_input(Message::TrackName),
                row![
                    button("Save name").on_press(Message::SaveTrackName),
                    button("Cancel").on_press(Message::CancelTrackName)
                ]
                .spacing(12)
            ]
            .spacing(12);
            for message in &self.messages {
                form = form.push(text(message));
            }
            return container(form).padding(24).width(Fill).into();
        }
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
        if project.nemotron_enabled() && self.model_acquisition.provenance().is_none() {
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
                || self.analysis_runtime() != result.runtime.as_ref()
            {
                return "ANALYSIS OUT OF DATE".into();
            }
            return "ANALYSIS COMPLETE".into();
        }
        "REFERENCES SAVED · ANALYZE TO VALIDATE".into()
    }

    fn next_action(&self) -> &'static str {
        let Some(project) = &self.project else {
            return "Open a project or import speaker tracks, including multichannel WAVs.";
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
        if project.nemotron_enabled() && self.model_acquisition.provenance().is_none() {
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
            text("Speaker examples & analysis").size(18),
            text("Drag a clear example on any track, then choose who is talking.").size(12),
            text(format!(
                "{} saved examples",
                project.reference_labels().len()
            ))
            .size(12),
        ]
        .spacing(8);
        controls = controls.push(
            row![
                button("Undo label")
                    .style(button::secondary)
                    .on_press(Message::UndoReferences),
                button("Redo label")
                    .style(button::secondary)
                    .on_press(Message::RedoReferences)
            ]
            .spacing(8),
        );
        controls = controls.push(button("Analyze").on_press(Message::AnalyzeReferences));
        if self.preview_processing.is_some() {
            controls = controls.push(button("Cancel analysis").on_press(Message::CancelAnalysis));
        }
        let (missing, room) = self.missing_references();
        controls = controls.push(
            text(if missing.is_empty() && room {
                "Examples supplied. Analyze checks usable speech and room tone.".into()
            } else {
                format!(
                    "Examples needed: {}{}",
                    missing.join(", "),
                    if room { "" } else { "; room tone" }
                )
            })
            .size(12),
        );
        controls = controls.push(
            button(if self.calibration_timing {
                "Hide speaker examples"
            } else {
                "Speaker examples"
            })
            .style(button::secondary)
            .on_press(Message::CalibrationTiming(!self.calibration_timing)),
        );
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
            controls = controls.push(text("Click an example on the waveform to drag its ends. Right-click to change speakers or delete it.").size(12));
            if project.reference_labels().is_empty() {
                controls = controls.push(
                    text("No examples yet. Drag across a waveform and choose who is talking.")
                        .size(12),
                );
            }
            for label in project.reference_labels() {
                controls = controls.push(
                    container(
                        column![
                            text(example_speakers(label)).size(14),
                            text(format!(
                                "{}–{} · {:.2} seconds",
                                example_time(label.range.start),
                                example_time(label.range.end),
                                (label.range.end - label.range.start) as f64
                                    / TICKS_PER_SECOND as f64
                            ))
                            .size(12),
                            row![
                                button(text("Play").size(12))
                                    .style(button::text)
                                    .on_press(Message::PlayExample(label.id)),
                                button(text("Show on timeline").size(12))
                                    .style(button::text)
                                    .on_press(Message::ShowExample(label.id)),
                                button(text("Delete").size(11))
                                    .style(button::text)
                                    .on_press(Message::RemoveReference(label.id)),
                            ]
                            .spacing(5),
                        ]
                        .spacing(4),
                    )
                    .padding([8, 0])
                    .width(Fill),
                );
            }
        }
        controls.into()
    }

    fn model_controls(&self) -> Element<'_, Message> {
        let enabled = self
            .project
            .as_ref()
            .is_some_and(|project| project.nemotron_enabled());
        let ready =
            self.model_acquisition.provenance().is_some() && !self.model_acquisition.is_busy();
        let mut controls = column![
            iced::widget::checkbox(enabled)
                .label("Use Nemotron (optional)")
                .on_toggle(Message::NemotronEnabled),
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
        if !enabled && !self.model_details && !self.model_acquisition.is_busy() {
            return column![
                iced::widget::checkbox(false)
                    .label("Use Nemotron (optional)")
                    .on_toggle(Message::NemotronEnabled),
                text("Acoustic analysis from your references").size(12),
                button("Model details").on_press(Message::ModelDetails(true)),
            ]
            .spacing(6)
            .into();
        }
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

    fn gain_controls(&self) -> Element<'_, Message> {
        let mut controls = column![
            iced::widget::checkbox(self.gain_tool)
                .label("Edit gain points")
                .on_toggle(Message::GainTool),
        ]
        .spacing(8);
        if self.gain_tool {
            controls = controls
                .push(text("Drag pink points to adjust gain and fades. Arrow keys move the selected point; Delete removes its override.").size(12))
                .push(row![
                    button("Undo gain").style(button::secondary).on_press(Message::UndoGain),
                    button("Redo gain").style(button::secondary).on_press(Message::RedoGain),
                ].spacing(6))
                .push(button("Add point at cursor").style(button::secondary).on_press(Message::InsertGainPoint))
                .push(button("Remove selected override").style(button::secondary).on_press(Message::DeleteGain))
                .push(iced::widget::checkbox(self.replace_gain_constraints)
                    .label("Replace overlapping overrides and protections")
                    .on_toggle(Message::ReplaceGainConstraints));
            if self.can_protect_selection() {
                controls = controls.push(
                    button("Preserve selection on all tracks")
                        .style(button::secondary)
                        .on_press(Message::ProtectSelection),
                );
            }
        }
        controls.into()
    }

    fn source_controls(&self) -> Element<'_, Message> {
        let Some(project) = &self.project else {
            return column![].into();
        };
        let mut controls = column![
            text("Sources").size(16),
            button("Recheck source files")
                .style(button::secondary)
                .on_press(Message::Validate),
        ]
        .spacing(8);
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
            if !source.available || project.pending_source_reviews().contains(&source.track_id) {
                controls = controls.push(source_controls);
            }
        }
        controls.into()
    }

    fn export_controls(&self) -> Element<'_, Message> {
        let mut controls = column![text("Export").size(18)].spacing(8);
        if let Some(processing) = &self.processing {
            controls = controls
                .push(text(format!(
                    "Final render: {}%",
                    processing.progress.load(Ordering::Relaxed)
                )))
                .push(button("Cancel final render").on_press(Message::Cancel));
        } else {
            controls = controls.push(
                button("Render WAV files")
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
        for message in &self.messages {
            controls = controls.push(text(message));
        }
        controls.into()
    }

    fn workspace(&self, compact: bool, height: f32) -> Element<'_, Message> {
        let ownership_stale = self.reference_stale
            || self
                .supervised_result
                .as_ref()
                .is_some_and(|analysis| self.analysis_runtime() != analysis.runtime.as_ref());
        let ownership_states = self
            .supervised_result
            .as_ref()
            .and_then(|analysis| analysis.temporal.states_at(self.timeline.playhead()).ok());
        let ownership_estimates = self.supervised_result.as_ref().and_then(|analysis| {
            analysis
                .ownership
                .estimates_at(self.timeline.playhead())
                .ok()
        });
        let ownership_status = match (&self.project, &self.supervised_result, &ownership_states) {
            (Some(project), Some(_), Some(states)) => {
                let speakers = project
                    .sources
                    .iter()
                    .map(|source| source.speaker_id)
                    .collect::<Vec<_>>();
                let summary = ownership_summary(&speakers, states);
                if ownership_stale {
                    format!("Stale analysis · {summary}")
                } else {
                    summary
                }
            }
            (_, Some(_), None) => "Ownership · outside analyzed audio".into(),
            _ => "Ownership · Analyze to see active speakers".into(),
        };
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
            button("Save project")
                .style(button::secondary)
                .on_press_maybe(
                    (self.project.is_some() && !self.sources_busy())
                        .then_some(Message::SaveProject)
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
                            canvas(
                                TimelineCanvas::waveform(
                                    cache,
                                    &loading.timeline,
                                    Message::Timeline,
                                    None,
                                    false,
                                )
                                .with_revision(self.drawing_revision),
                            )
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
                            "Import synchronized speaker tracks or a multichannel WAV, or drop them anywhere in this window."
                        ),
                        button("Import tracks").on_press(Message::ImportStems),
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
                    canvas(
                        TimelineCanvas::ruler(
                            &self.timeline,
                            self.selecting_range || self.calibrating,
                            Message::Timeline,
                        )
                        .with_revision(self.drawing_revision),
                    )
                    .width(Fill)
                    .height(28),
                )
                .padding([0, 12]),
            );
            for (index, source) in project.sources.iter().enumerate() {
                let filename = source.display_name();
                let mut short_name = filename.chars().take(24).collect::<String>();
                if filename.chars().count() > 24 {
                    short_name.push('…');
                }
                let mut header = row![
                    button(
                        text(if self.solo_tracks & (1u64 << index) != 0 {
                            "Solo · on"
                        } else {
                            "Solo"
                        })
                        .size(12)
                    )
                    .style(if self.solo_tracks & (1u64 << index) != 0 {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .on_press(Message::SoloTrack(index)),
                    iced::widget::tooltip(
                        text(format!("Speaker {} · {short_name}", source.speaker_id.0))
                            .size(13)
                            .width(Fill),
                        text(source.path.display().to_string()).size(12),
                        iced::widget::tooltip::Position::Bottom,
                    ),
                    button(text("Rename").size(12))
                        .style(button::secondary)
                        .on_press(Message::EditTrackName(index)),
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
                if !source.available {
                    header = header.push(text("Missing source").size(12));
                }
                if project.pending_source_reviews().contains(&source.track_id) {
                    header = header.push(text("Review source").size(12));
                }
                let mut moving_meter: Option<Element<'_, Message>> = None;
                if let Some(estimates) = &ownership_estimates
                    && let Some(estimate) = estimates
                        .iter()
                        .find(|estimate| estimate.track == source.track_id)
                {
                    let state = self.supervised_result.as_ref().and_then(|analysis| {
                        let index = analysis
                            .temporal
                            .tracks
                            .iter()
                            .position(|track| *track == source.track_id)?;
                        ownership_states.as_ref()?.get(index).copied()
                    });
                    let (label, color) = match state {
                        Some(crate::TemporalTrackState::Owned) => {
                            ("Active", iced::Color::from_rgb(0.2, 0.75, 0.4))
                        }
                        Some(crate::TemporalTrackState::Absent) => {
                            ("Inactive", iced::Color::from_rgb(0.5, 0.55, 0.6))
                        }
                        _ => ("Uncertain", iced::Color::from_rgb(0.9, 0.65, 0.2)),
                    };
                    let meter_label = format!(
                        "{}{label} · {}",
                        if ownership_stale { "Stale · " } else { "" },
                        estimate
                            .probability
                            .map_or_else(|| "unresolved".into(), |p| format!("{:.0}%", p * 100.0))
                    );
                    let probability = estimate.probability.unwrap_or(0.0);
                    let meter = move || {
                        column![
                            text(meter_label.clone()).size(12),
                            progress_bar(0.0..=1.0, probability)
                                .girth(8)
                                .style(move |theme| iced::widget::progress_bar::Style {
                                    bar: color.into(),
                                    ..iced::widget::progress_bar::primary(theme)
                                }),
                        ]
                        .spacing(3)
                        .width(170)
                    };
                    if self.meters_at_playhead {
                        moving_meter = Some(
                            responsive(move |size| {
                                let x = self
                                    .timeline
                                    .x_for_sample(self.timeline.playhead(), size.width);
                                row![
                                    Space::new().width(ownership_meter_left(x, size.width, 170.0)),
                                    meter(),
                                ]
                                .into()
                            })
                            .height(30)
                            .into(),
                        );
                    } else {
                        header = header.push(meter());
                    }
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
                            .with_revision(self.drawing_revision)
                            .with_context(move |point, time| {
                                Message::OpenTimelineMenu(index, time, point)
                            })
                            .with_examples(
                                project.reference_labels(),
                                self.selected_example,
                                self.example_draft,
                                Message::Example,
                            )
                            .with_selection(
                                move |start, end, point| {
                                    Message::CompleteTrackSelection(index, start, end, point)
                                },
                            ),
                        )
                        .width(Fill)
                        .height(Fill)
                        .into()
                    },
                );
                let mut track_content = column![header].spacing(3).height(Fill);
                if let Some(meter) = moving_meter {
                    track_content = track_content.push(meter);
                }
                timeline_content = timeline_content.push(
                    container(
                        track_content.push(waveform).push(
                            canvas(crate::gain_editor::GainCanvas {
                                revision: self.drawing_revision,
                                project,
                                constraints: self
                                    .gain_drag
                                    .as_ref()
                                    .map_or(&self.gain_constraints, |drag| {
                                        &drag.draft().constraints
                                    }),
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
                        ),
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
            row![
                text("SESSION TIMELINE").size(12),
                Space::new().width(Fill),
                button(text(if self.playback_scroll == PlaybackScroll::Locked {
                    "Unlock playhead"
                } else {
                    "Lock playhead"
                }).size(12)).on_press(Message::LockPlayhead)
                    .style(if self.playback_scroll == PlaybackScroll::Locked { button::primary } else { button::secondary }),
                button(text("Center playhead").size(12)).on_press(Message::CenterPlayhead),
                button(text(if self.meters_at_playhead { "Meters: Playhead" } else { "Meters: Right" }).size(12))
                    .on_press(Message::ToggleMeterPlacement),
            ].spacing(8).align_y(iced::Center),
            text(
                "Red background = reduced gain · Click to seek · drag to label · wheel to zoom · Ctrl-wheel or middle-drag to pan"
            )
            .size(11),
            tracks,
        ]
        .spacing(10)
        .width(Fill)
        .height(Fill);
        let preview_state: Element<'_, Message> = if let Some(processing) = &self.preview_processing
        {
            let stage = match processing.progress.load(Ordering::Relaxed) {
                0..=19 => "Preparing audio and validating references",
                20..=54 => "Analyzing optional speaker-activity evidence on CPU",
                55..=79 => "Fitting session ownership estimates",
                80..=94 => "Resolving ownership and uncertainty",
                _ => "Composing the current gain preview",
            };
            column![
                text(stage),
                text(format!(
                    "Working · elapsed {:02}:{:02}",
                    processing.started_at.elapsed().as_secs() / 60,
                    processing.started_at.elapsed().as_secs() % 60
                ))
                .size(12),
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
            || self
                .supervised_result
                .as_ref()
                .is_some_and(|analysis| self.analysis_runtime() != analysis.runtime.as_ref())
        {
            text("Analysis out of date · click Analyze").into()
        } else if self.preview.is_some() {
            text("Preview ready").into()
        } else {
            text("Preview not analyzed").into()
        };
        let parameters = &self.preview_parameters;
        let setup_required = self.project.as_ref().is_some_and(|project| {
            project.require_alignment().is_err()
                || !project.pending_source_reviews().is_empty()
                || project.sources.iter().any(|source| !source.available)
                || project.nemotron_enabled() && self.model_acquisition.provenance().is_none()
        });
        let mut setup = column![
            button(if self.setup_details || setup_required {
                "Model & source setup ▾"
            } else {
                "Model & source setup ▸"
            })
            .style(button::secondary)
            .on_press(Message::SetupDetails(!self.setup_details))
        ]
        .spacing(12);
        if self.setup_details || setup_required || self.model_acquisition.is_busy() {
            setup = setup
                .push(self.model_controls())
                .push(self.source_controls());
        }
        let inspector = container(
            column![
                preview_state,
                text(self.next_action()).size(12),
                container(self.calibration_controls()).width(Fill),
                container(
                    column![
                        text("Sound").size(18),
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
                        text(format!(
                            "Ignore gaps shorter than: {:.0} ms",
                            parameters.minimum_gap_ms
                        )),
                        slider(
                            10.0..=5000.0,
                            parameters.minimum_gap_ms,
                            Message::MinimumGap
                        )
                        .step(1.0_f32)
                        .on_release(Message::RefreshPreview),
                        self.gain_controls(),
                    ]
                    .spacing(8)
                )
                .width(Fill),
                container(self.export_controls()).width(Fill),
                setup,
            ]
            .spacing(20),
        )
        .padding(18)
        .width(if compact { Fill } else { Length::Fixed(280.0) })
        .style(container::bordered_box);
        let inspector: Element<'_, Message> = if self.sources_busy() {
            container(column![text("Loading tracks").size(20),
                text("Waveforms appear as each track is read. Validation and source checks run in the background, then choose references and click Analyze."),
                text("You can scroll the tracks and drop more audio files. Editing becomes available when the import is validated."),
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

        let ownership_monitor = row![
            text(ownership_status).size(16).width(Fill),
            text("Ownership estimate · bars are not audio level or gain").size(11),
        ]
        .spacing(8)
        .align_y(iced::Center);
        let base: Element<'_, Message> =
            container(column![header, editor, ownership_monitor, transport].spacing(12))
                .padding(12)
                .width(Fill)
                .height(Fill)
                .into();
        let Some(point) = self
            .gain_menu
            .map(|(_, _, point)| point)
            .or_else(|| self.reference_menu.map(|(_, point)| point))
        else {
            return base;
        };
        let overlay = responsive(move |size| {
            if let Some((track, time, _)) = self.gain_menu {
                let panel = container(
                    column![
                        button("Delete Preserve / Attenuate mark")
                            .on_press(Message::DeleteRegion(track, time)),
                        text("Restore automatic gain for this section.").size(12),
                        button("Cancel").on_press(Message::DismissReferenceMenu),
                    ]
                    .spacing(6),
                )
                .padding(10)
                .style(container::bordered_box);
                let positioned =
                    iced::widget::float(opaque(panel)).translate(move |bounds, viewport| {
                        iced::Vector::new(
                            point.x.clamp(
                                viewport.x,
                                (viewport.x + viewport.width - bounds.width).max(viewport.x),
                            ) - bounds.x,
                            point.y.clamp(
                                viewport.y,
                                (viewport.y + viewport.height - bounds.height).max(viewport.y),
                            ) - bounds.y,
                        )
                    });
                return opaque(
                    mouse_area(container(positioned).width(Fill).height(Fill))
                        .on_press(Message::DismissReferenceMenu)
                        .on_right_press(Message::DismissReferenceMenu)
                        .on_scroll(|_| Message::DismissReferenceMenu),
                );
            }
            let mut menu = column![
                text("Who is talking?").size(16),
                text("Select everyone speaking. None means room tone.").size(12),
                text("1–8: toggle · R: clear · Enter: save · Esc: close").size(11)
            ]
            .spacing(6);
            if let Some(project) = &self.project {
                for source in &project.sources {
                    let speaker = source.speaker_id;
                    menu = menu.push(
                        iced::widget::checkbox(self.reference_speakers.contains(&speaker))
                            .label(format!("Speaker {}", speaker.0))
                            .on_toggle(move |active| Message::ReferenceSpeaker(speaker, active)),
                    );
                }
                menu = menu.push(
                    row![
                        button(if self.selected_example.is_some() {
                            "Save changes"
                        } else {
                            "Save example"
                        })
                        .on_press(Message::ContextReference(
                            self.reference_speakers.clone(),
                            false
                        )),
                        button("Cancel").on_press(Message::DismissReferenceMenu)
                    ]
                    .spacing(8),
                );
                if let Some(id) = self.selected_example {
                    menu = menu.push(
                        text(
                            "Drag the left or right handle on the waveform to adjust this example.",
                        )
                        .size(12),
                    );
                    menu = menu.push(
                        button("Delete example")
                            .style(button::secondary)
                            .on_press(Message::RemoveReference(id)),
                    );
                }
            }
            if let Some(error) = &self.calibration_error {
                menu = menu.push(iced::widget::tooltip(
                    text("Label not saved. Show details").size(12),
                    text(error).size(12),
                    iced::widget::tooltip::Position::Top,
                ));
                if !self.reference_conflicts().is_empty() {
                    menu = menu.push(button("Replace conflicting whole labels").on_press(
                        Message::ContextReference(self.reference_speakers.clone(), true),
                    ));
                }
            }
            let panel = container(menu)
                .padding(10)
                .width(320.0_f32.min(size.width))
                .style(container::bordered_box);
            let positioned =
                iced::widget::float(opaque(panel)).translate(move |bounds, viewport| {
                    iced::Vector::new(
                        point.x.clamp(
                            viewport.x,
                            (viewport.x + viewport.width - bounds.width).max(viewport.x),
                        ) - bounds.x,
                        point.y.clamp(
                            viewport.y,
                            (viewport.y + viewport.height - bounds.height).max(viewport.y),
                        ) - bounds.y,
                    )
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
