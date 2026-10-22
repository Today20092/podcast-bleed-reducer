#[cfg(test)]
use crate::CalibrationKind;
use crate::{
    AnalysisPreview, AutomixParameters, Comparison, Project, RenderState, ReviewWorkspace,
    SpeakerId, TimeRange,
    appearance::{self, Appearance, colors},
    audition::{AuditionMode, AuditionPreparation, LiveAudition},
    gui_controls::button,
    gui_icons::ToolbarIcon,
    shortcuts::{Binding, Command, Context, DEFINITIONS, Shortcuts},
    timebase::{TICKS_PER_SECOND, TimePosition},
    timeline::{PlaybackScroll, TimelineEvent, TimelineViewport},
    validate_export,
    waveform::{self, ExampleEvent, TimelineCanvas, WaveformCache},
};
use iced::{
    Element, Fill, Length, Subscription, Task, Theme, keyboard,
    widget::{
        Space, canvas, column, container, mouse_area, opaque, progress_bar, responsive, row,
        scrollable, slider, stack, text, text_input,
    },
    window,
};
mod button {
    pub use iced::widget::button::*;
}
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};

#[cfg(test)]
fn editor_theme() -> Theme {
    Appearance::Dark.theme()
}

fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let c = colors(theme);
    let mut style = button::secondary(theme, status);
    style.background = Some(iced::Background::Color(match status {
        button::Status::Hovered => c.hover,
        button::Status::Pressed => c.pressed,
        _ => c.control,
    }));
    style.text_color = if status == button::Status::Disabled {
        c.disabled
    } else {
        theme.palette().text
    };
    style.border = iced::Border {
        color: c.border,
        width: 1.0,
        radius: 6.0.into(),
    };
    style.shadow = if matches!(status, button::Status::Pressed | button::Status::Disabled) {
        iced::Shadow::default()
    } else {
        iced::Shadow {
            color: iced::Color {
                a: 0.12,
                ..iced::Color::BLACK
            },
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 2.0,
        }
    };
    style
}

fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let c = colors(theme);
    if status == button::Status::Disabled {
        return secondary_button(theme, status);
    }
    button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => c.action_hover,
            button::Status::Pressed => c.action_pressed,
            _ => theme.palette().primary,
        })),
        text_color: c.action_text,
        border: iced::Border {
            radius: 6.0.into(),
            ..iced::Border::default()
        },
        shadow: if status == button::Status::Pressed {
            iced::Shadow::default()
        } else {
            iced::Shadow {
                color: iced::Color {
                    a: 0.16,
                    ..iced::Color::BLACK
                },
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            }
        },
        ..button::primary(theme, status)
    }
}

fn editor_panel(theme: &Theme) -> container::Style {
    let c = colors(theme);
    container::Style {
        background: Some(c.panel.into()),
        text_color: Some(theme.palette().text),
        border: iced::Border {
            color: c.panel_border,
            width: 1.0,
            radius: 5.0.into(),
        },
        ..container::Style::default()
    }
}

fn inspector_heading(label: &str) -> iced::widget::Text<'_> {
    text(label).size(16).font(iced::Font {
        weight: iced::font::Weight::Semibold,
        ..iced::Font::with_name("Segoe UI")
    })
}

fn inspector_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = secondary_button(theme, status);
    style.border.color = colors(theme).panel_border;
    style
}

fn sound_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = secondary_button(theme, status);
    style.shadow = iced::Shadow::default();
    if status == button::Status::Active {
        style.background = Some(colors(theme).surface.into());
    }
    style
}

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

fn available_export_path(directory: &std::path::Path) -> PathBuf {
    let mut path = directory.join("CleanSlate export");
    for suffix in 2.. {
        if !path.exists() {
            return path;
        }
        path = directory.join(format!("CleanSlate export {suffix}"));
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
            "Speaker activity uncertain"
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
        summary.push_str(" · other speakers uncertain");
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
    fn embedded_application_icon_has_valid_rgba_dimensions() {
        iced::window::icon::from_rgba(include_bytes!("../assets/cleanslate.rgba").to_vec(), 64, 64)
            .expect("embedded application icon must remain valid");
    }

    #[test]
    fn appearance_changes_only_drawing_and_survives_new_project_after_save_failure() {
        let mut app = super::CleanSlateApp {
            timeline: super::TimelineViewport::new(100),
            ..super::CleanSlateApp::default()
        };
        let root = std::env::temp_dir().join(format!(
            "cleanslate-appearance-state-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let source_path = root.join("speaker.wav");
        write_wav(&source_path, &vec![0.0; 1920]);
        app.project =
            Some(synchronized_project(root.join("session.cleanslate"), vec![source_path]).unwrap());
        let project = serde_json::to_value(&app.project).unwrap();
        app.timeline.set_selection(Some((10, 20)));
        app.timeline.seek(15);
        app.solo_tracks = 3;
        app.gain_tool = true;
        app.audition_mode = super::AuditionMode::Processed;
        let selection = app.timeline.selection();
        let render = app.render_generation.current();
        let preview = app.preview_generation;
        let source = app.source_generation;
        let parameters = app.preview_parameters;
        let drawing = app.drawing_revision;
        let _ = app.update(super::Message::Appearance(
            crate::appearance::Appearance::Light,
        ));
        assert_eq!(app.appearance, crate::appearance::Appearance::Light);
        assert!(app.appearance_error.is_some());
        assert_eq!(app.timeline.selection(), selection);
        assert_eq!(serde_json::to_value(&app.project).unwrap(), project);
        assert_eq!(app.timeline.playhead(), 15);
        assert_eq!(app.solo_tracks, 3);
        assert!(app.gain_tool);
        assert_eq!(app.audition_mode, super::AuditionMode::Processed);
        assert_eq!(app.render_generation.current(), render);
        assert_eq!(app.preview_generation, preview);
        assert_eq!(app.source_generation, source);
        assert_eq!(app.preview_parameters, parameters);
        assert_ne!(app.drawing_revision, drawing);
        app.reset_project(true);
        assert_eq!(app.appearance, crate::appearance::Appearance::Light);
        assert!(app.appearance_error.is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
    mod export_fixture {
        use crate as rust_app;
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/common/supervised_fixture.rs"
        ));
    }

    #[test]
    fn completed_analysis_keeps_timings_in_collapsed_details() {
        use super::{CleanSlateApp, Message};
        let (root, project, _audio) = export_fixture::session();
        let mut app = CleanSlateApp {
            preview_parameters: project.parameters(),
            project: Some(project),
            messages: vec!["Previous informational message".into()],
            ..Default::default()
        };
        let _ = app.update(Message::AnalyzeReferences);
        app.launch_preview();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while app.preview_processing.is_some() {
            app.poll_processing();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(app.preview.is_some(), "{:?}", app.preview_error);
        assert!(app.messages.is_empty());
        assert!(
            app.analysis_details
                .as_ref()
                .unwrap()
                .iter()
                .any(|(label, seconds)| *label == "Source checks" && seconds.is_finite())
        );
        assert!(!app.analysis_details_open);
        let parameters = app.preview_parameters;
        let _ = app.update(Message::AnalysisDetails(true));
        assert!(app.analysis_details_open);
        assert_eq!(app.preview_parameters, parameters);
        app.reset_project(true);
        assert!(app.analysis_details.is_none() && !app.analysis_details_open);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catch_up_stays_selected_through_pan_until_explicitly_disabled() {
        use super::{CleanSlateApp, Message, PlaybackScroll};
        let mut app = CleanSlateApp::default();
        assert_eq!(app.playback_scroll, PlaybackScroll::Page);
        let _ = app.update(Message::LockPlayhead);
        assert_eq!(app.playback_scroll, PlaybackScroll::Locked);
        let _ = app.update(Message::ToggleCatchUp);
        assert_eq!(app.playback_scroll, PlaybackScroll::Page);
        let _ = app.update(Message::Timeline(crate::timeline::TimelineEvent::Pan {
            delta_x: -10.0,
            width: 1000.0,
        }));
        assert_eq!(app.playback_scroll, PlaybackScroll::Page);
        let _ = app.update(Message::Timeline(crate::timeline::TimelineEvent::PanStart));
        let _ = app.update(Message::Timeline(crate::timeline::TimelineEvent::Pan {
            delta_x: 10.0,
            width: 1000.0,
        }));
        assert_eq!(app.playback_scroll, PlaybackScroll::Page);
        let _ = app.update(Message::Timeline(crate::timeline::TimelineEvent::PanEnd));
        assert_eq!(app.playback_scroll, PlaybackScroll::Page);
        let _ = app.update(Message::ToggleCatchUp);
        assert_eq!(app.playback_scroll, PlaybackScroll::Off);
    }

    #[test]
    fn catch_up_shortcut_remaps_persists_and_uses_the_same_toggle() {
        use super::{CleanSlateApp, Command, Message, PlaybackScroll};
        use iced::keyboard::{Key, Modifiers};
        let mut app = CleanSlateApp::default();
        let button = crate::button_action::message(super::RegisteredButton(Command::CatchUp));
        let _ = app.update(button);
        assert_eq!(app.playback_scroll, PlaybackScroll::Off);
        let _ = app.update(Message::KeyPressed(
            Key::Character("c".into()),
            Modifiers::SHIFT,
        ));
        assert_eq!(app.playback_scroll, PlaybackScroll::Page);
        let key = Key::Character("c".into());
        let _ = app.update(Message::KeyPressed(key.clone(), Modifiers::SHIFT));
        assert_eq!(app.playback_scroll, PlaybackScroll::Off);
        let binding =
            crate::shortcuts::Binding::capture(&Key::Character("j".into()), Modifiers::SHIFT)
                .unwrap();
        app.shortcuts
            .assign(Command::CatchUp, Some(binding))
            .unwrap();
        let saved = serde_json::to_value(&app.shortcuts).unwrap();
        let (reloaded, errors) = crate::shortcuts::Shortcuts::from_value(&saved);
        assert!(errors.is_empty());
        app.shortcuts = reloaded;
        assert_eq!(app.shortcuts.hint(Command::CatchUp), "Shift+J");
        let _ = app.update(Message::KeyPressed(key, Modifiers::SHIFT));
        assert_eq!(app.playback_scroll, PlaybackScroll::Off);
        let _ = app.update(Message::KeyPressed(
            Key::Character("j".into()),
            Modifiers::SHIFT,
        ));
        assert_eq!(app.playback_scroll, PlaybackScroll::Page);
    }

    #[test]
    fn initially_unbound_timeline_action_can_be_discovered_remapped_saved_and_invoked() {
        use super::{CleanSlateApp, Command, Message};
        use iced::keyboard::{Key, Modifiers};
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("preferences.json");
        let mut app = CleanSlateApp {
            preferences_path: Some(path.clone()),
            ..Default::default()
        };
        assert!(
            super::DEFINITIONS
                .iter()
                .any(|d| d.command == Command::Meters
                    && d.name == "Move speaker meters"
                    && d.defaults.is_empty())
        );
        assert_eq!(app.shortcuts.hint(Command::Meters), "Unbound");
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::CaptureShortcut(Command::Meters));
        let _ = app.update(Message::KeyPressed(
            Key::Character("j".into()),
            Modifiers::SHIFT,
        ));
        let _ = app.update(Message::ApplyShortcut);
        let (reloaded, _, error) = crate::appearance::load_shortcuts(&path);
        assert!(error.is_none());
        app.shortcuts = reloaded;
        assert_eq!(
            app.action_hint(Command::Meters),
            "Move speaker meters · Shift+J"
        );
        let _ = app.update(Message::CloseHelp);
        let initial = app.meters_at_playhead;
        let _ = app.update(Message::InvokeAction(Command::Meters));
        assert_ne!(app.meters_at_playhead, initial);
        let _ = app.update(Message::KeyPressed(
            Key::Character("j".into()),
            Modifiers::SHIFT,
        ));
        assert_eq!(app.meters_at_playhead, initial);
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::CaptureShortcut(Command::Meters));
        let _ = app.update(Message::KeyPressed(
            Key::Character("k".into()),
            Modifiers::SHIFT,
        ));
        let _ = app.update(Message::ApplyShortcut);
        let (reassigned, _, error) = crate::appearance::load_shortcuts(&path);
        assert!(error.is_none());
        app.shortcuts = reassigned;
        assert_eq!(
            app.action_hint(Command::Meters),
            "Move speaker meters · Shift+K"
        );
        let _ = app.update(Message::CloseHelp);
        let _ = app.update(Message::KeyPressed(
            Key::Character("j".into()),
            Modifiers::SHIFT,
        ));
        assert_eq!(app.meters_at_playhead, initial);
        let _ = app.update(Message::KeyPressed(
            Key::Character("k".into()),
            Modifiers::SHIFT,
        ));
        assert_ne!(app.meters_at_playhead, initial);
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::UnbindShortcut(Command::Meters));
        assert_eq!(app.shortcuts.hint(Command::Meters), "Unbound");
        let _ = app.update(Message::RestoreShortcut(Command::Meters));
        assert_eq!(app.shortcuts.hint(Command::Meters), "Unbound");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registered_timeline_buttons_and_keys_obey_the_same_modal_availability() {
        use super::{CleanSlateApp, Command, Message};
        use iced::keyboard::{Key, Modifiers};
        let mut app = CleanSlateApp::default();
        let mode = app.playback_scroll;
        let _ = app.update(Message::Settings);
        assert!(!app.action_available(Command::CatchUp));
        let _ = app.update(crate::button_action::message(super::RegisteredButton(
            Command::CatchUp,
        )));
        let _ = app.update(Message::KeyPressed(
            Key::Character("c".into()),
            Modifiers::SHIFT,
        ));
        assert_eq!(app.playback_scroll, mode);
        let _ = app.update(Message::CloseHelp);
        assert!(app.action_available(Command::CatchUp));
        let _ = app.update(crate::button_action::message(super::RegisteredButton(
            Command::CatchUp,
        )));
        assert_ne!(app.playback_scroll, mode);
    }

    #[test]
    fn inspector_emphasis_requires_ready_examples_and_safe_sources() {
        assert!(!super::CleanSlateApp::default().analysis_is_next_action());
        let (root, project, _) = export_fixture::session();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        assert!(app.analysis_is_next_action());
        app.project.as_mut().unwrap().sources[0].available = false;
        assert!(!app.analysis_is_next_action());
        assert_eq!(
            app.next_action(),
            "Review source files below before Analyze or Export."
        );
        app.project.as_mut().unwrap().sources[0].available = true;
        assert!(app.analysis_is_next_action());
        assert!(!app.can_prepare_export());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn tab_navigation_preserves_drawing_and_existing_menu_ownership() {
        use iced::keyboard::{Key, Modifiers, key::Named};
        let mut app = super::CleanSlateApp::default();
        let revision = app.drawing_revision;
        for import in [false, true] {
            app.import_dialog = import;
            app.sound_draft = import.then_some((super::SoundParameter::Open, "42".into()));
            assert!(!app.sound_input_owns_keyboard());
            let _ = app.update(super::Message::KeyPressed(
                Key::Named(Named::Tab),
                Modifiers::empty(),
            ));
            assert_eq!(app.drawing_revision, revision);
            assert_eq!(app.import_dialog, import);
        }
        app.import_dialog = false;
        app.menu_open = true;
        let selected = app.menu_selected;
        let _ = app.update(super::Message::KeyPressed(
            Key::Named(Named::Tab),
            Modifiers::empty(),
        ));
        assert_ne!(app.menu_selected, selected);
    }

    #[test]
    fn sound_navigation_is_routed_once_using_subscription_ownership() {
        use iced::keyboard::{
            Event, Key, Location, Modifiers,
            key::{Code, Named, Physical},
        };
        for key in [Named::Tab, Named::Escape] {
            let event = iced::Event::Keyboard(Event::KeyPressed {
                key: Key::Named(key),
                modified_key: Key::Named(key),
                physical_key: Physical::Code(if key == Named::Tab {
                    Code::Tab
                } else {
                    Code::Escape
                }),
                location: Location::Standard,
                modifiers: Modifiers::empty(),
                text: None,
                repeat: false,
            });
            let mut app = super::CleanSlateApp::default();
            let _ = app.update(super::Message::EditSound(super::SoundParameter::Open));
            let deferred_global = super::application_input_event(
                event.clone(),
                iced::event::Status::Ignored,
                app.sound_input_owns_keyboard(),
            );
            let _ = app.update(if key == Named::Tab {
                super::Message::SoundTab(false)
            } else {
                super::Message::CancelSound
            });
            assert!(app.sound_draft.is_none());
            assert!(
                deferred_global.is_none(),
                "sound subscription already owns this event"
            );
            assert!(matches!(
                super::application_input_event(
                    event,
                    iced::event::Status::Ignored,
                    app.sound_input_owns_keyboard()
                ),
                Some(super::Message::KeyPressed(_, _))
            ));
        }
    }

    #[test]
    fn editing_target_tracks_active_tool_range_and_escape_without_reassigning_time() {
        use super::Message;
        let (root, project, _) = export_fixture::session();
        let t = crate::TICKS_PER_SECOND;
        let mut app = super::CleanSlateApp {
            project: Some(project),
            gain_tool: true,
            timeline: super::TimelineViewport::new(3 * t),
            ..Default::default()
        };
        app.timeline.set_selection(Some((t / 10, t / 5)));
        app.selection_track = Some(0);
        let _ = app.update(Message::GainAction(1, false));
        let edit = app
            .project
            .as_ref()
            .unwrap()
            .manual_gain_edits()
            .last()
            .unwrap()
            .id;
        let _ = app.update(Message::GainEvent(
            1,
            crate::GainEditorEvent::Begin(edit, 0),
        ));
        let _ = app.update(Message::GainEvent(1, crate::GainEditorEvent::End));
        assert_eq!(app.editing_track(), Some(1));
        let range = app.timeline.selection();
        let _ = app.update(Message::GainTool(false));
        assert_eq!(app.editing_track(), Some(0));
        assert_eq!(app.timeline.selection(), range);
        let _ = app.update(Message::GainTool(true));
        assert_eq!(app.editing_track(), Some(1));
        let _ = app.update(Message::TrackTimeline(
            0,
            super::TimelineEvent::Select(t / 3, t / 2),
        ));
        assert_eq!(app.editing_track(), Some(0));
        assert!(app.selected_gain_point.is_none());
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::empty(),
        ));
        assert_eq!(app.editing_track(), None);
        let label = app.project.as_ref().unwrap().reference_labels()[0].id;
        app.selected_gain_point = Some((1, edit, 0));
        app.select_example(label);
        assert!(app.selected_gain_point.is_none());
        assert_eq!(
            app.editing_track(),
            None,
            "examples describe a session-wide range"
        );
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_dialog_cancel_and_submit_keep_destination_transient_and_solo_out_of_delivery() {
        let (root, mut project, audio) = export_fixture::session();
        project
            .set_export_encoding(crate::ExportEncoding::Pcm24)
            .unwrap();
        let progress = std::sync::atomic::AtomicU8::new(0);
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let analysis = crate::EditorAnalysisWorker::default()
            .analyze_acoustic_evidence(&project, &audio, &progress, &cancel)
            .unwrap();
        let effective = analysis
            .install(
                &mut project,
                &mut crate::EffectiveEnvelopeCache::default(),
                None,
            )
            .unwrap();
        let mut app = super::CleanSlateApp {
            preview_parameters: project.parameters(),
            supervised_result: Some(analysis),
            effective_result: Some(effective),
            project: Some(project),
            solo_tracks: 1,
            ..Default::default()
        };
        assert!(app.can_prepare_export());
        assert!(!app.analysis_is_next_action());
        let _ = app.update(super::Message::Export);
        assert_eq!(
            app.export_dialog.as_ref().unwrap().settings.encoding,
            crate::ExportEncoding::Pcm24
        );
        assert!(app.export_dialog.as_ref().unwrap().destination.is_none());
        let _ = app.update(super::Message::Attenuation(-12.0));
        assert_eq!(
            app.preview_parameters.attenuation_db,
            app.project.as_ref().unwrap().parameters().attenuation_db
        );
        let _ = app.view();
        let _ = app.update(super::Message::CancelExportDialog);
        assert!(app.processing.is_none() && app.exporting.is_none());
        let _ = app.update(super::Message::Export);
        let _ = app.update(super::Message::ExportLayout(
            crate::ExportLayout::Multichannel,
        ));
        let _ = app.update(super::Message::ExportEncoding(
            crate::ExportEncoding::Float32,
        ));
        app.export_dialog.as_mut().unwrap().destination = Some(root.join("delivery"));
        let _ = app.update(super::Message::SubmitExport);
        assert!(app.export_dialog.is_none());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while app.processing.is_some() || app.exporting.is_some() {
            app.poll_processing();
            app.poll_exporting();
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(
            app.messages[0].starts_with("WAV files saved"),
            "{:?}",
            app.messages
        );
        let mut reader =
            bwavfile::WaveReader::open(root.join("delivery/session-clean.wav")).unwrap();
        assert_eq!(reader.format().unwrap().channel_count, 2);
        let mut samples = vec![0.0_f32; 128_000];
        reader
            .audio_frame_reader()
            .unwrap()
            .read_frames(&mut samples)
            .unwrap();
        assert!(
            samples
                .as_chunks::<2>()
                .0
                .iter()
                .any(|frame| frame[1] != 0.0),
            "Solo must not remove other delivered channels"
        );
        let _ = app.update(super::Message::Export);
        assert!(app.export_dialog.as_ref().unwrap().destination.is_none());
        assert_eq!(
            app.export_dialog.as_ref().unwrap().settings.encoding,
            crate::ExportEncoding::Float32
        );
        let _ = app.update(super::Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::default(),
        ));
        assert!(app.export_dialog.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn read_only_originals_and_saved_project_survive_local_analysis_and_destination_retry() {
        let (media, mut project, _audio) = export_fixture::session();
        let destination = crate::audio_io::multichannel_tests::root();
        let project_folder = media.join("projects");
        std::fs::create_dir(&project_folder).unwrap();
        let original_project = project_folder.join("episode.cleanslate");
        std::fs::copy(project.path(), &original_project).unwrap();
        project = crate::Project::open(&original_project).unwrap();
        let saved = std::fs::read(&original_project).unwrap();
        let sources: Vec<_> = project
            .sources
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        let original_permissions: Vec<_> = sources
            .iter()
            .map(|(path, _)| std::fs::metadata(path).unwrap().permissions())
            .collect();
        for (path, _) in &sources {
            let mut permissions = std::fs::metadata(path).unwrap().permissions();
            permissions.set_readonly(true);
            std::fs::set_permissions(path, permissions).unwrap();
        }
        let entries: std::collections::BTreeSet<_> = std::fs::read_dir(&media)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert!(project.localize_for_editing().unwrap());
        let local = project.path().to_owned();
        assert!(
            !project.localize_for_editing().unwrap(),
            "reopening a local project must retain its location"
        );
        let working = crate::storage::working_directory(&local).unwrap();
        assert!(!working.starts_with(&media));
        let progress = std::sync::atomic::AtomicU8::new(0);
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let analysis = crate::EditorAnalysisWorker::default()
            .analyze(
                &project,
                &working.join("analysis-cache"),
                &progress,
                &cancel,
            )
            .unwrap();
        assert!(working.join("analysis-cache/audio").is_dir());
        let effective = analysis
            .install(
                &mut project,
                &mut crate::EffectiveEnvelopeCache::default(),
                None,
            )
            .unwrap();
        let mut app = super::CleanSlateApp {
            preview_parameters: project.parameters(),
            supervised_result: Some(analysis),
            effective_result: Some(effective),
            project: Some(project),
            ..Default::default()
        };
        let blocked = destination.join("blocked");
        std::fs::write(&blocked, b"retain existing file").unwrap();
        app.export_current();
        app.export_dialog.as_mut().unwrap().destination = Some(blocked.join("delivery"));
        app.submit_export();
        let finish = |app: &mut super::CleanSlateApp| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while app.processing.is_some() || app.exporting.is_some() {
                app.poll_processing();
                app.poll_exporting();
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        };
        finish(&mut app);
        assert!(
            app.messages[0].contains("Could not create export parent folder"),
            "{:?}",
            app.messages
        );
        assert!(
            app.project
                .as_ref()
                .unwrap()
                .render_record()
                .unwrap()
                .report
                .starts_with(working.join("renders"))
        );
        assert_eq!(std::fs::read(&blocked).unwrap(), b"retain existing file");
        app.export_current();
        let delivery = super::available_export_path(&destination);
        app.export_dialog.as_mut().unwrap().destination = Some(delivery.clone());
        app.submit_export();
        finish(&mut app);
        assert!(
            app.messages[0].starts_with("WAV files saved"),
            "{:?}",
            app.messages
        );
        assert!(delivery.join("speaker-0-clean.wav").is_file());
        assert_eq!(std::fs::read(&original_project).unwrap(), saved);
        assert_eq!(
            std::fs::read_dir(&media)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect::<std::collections::BTreeSet<_>>(),
            entries
        );
        for ((path, bytes), permissions) in sources.into_iter().zip(original_permissions) {
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            std::fs::set_permissions(path, permissions).unwrap();
        }
        assert_eq!(
            crate::Project::open(local)
                .unwrap()
                .reference_labels()
                .len(),
            3
        );
        std::fs::remove_dir_all(media).unwrap();
        std::fs::remove_dir_all(destination).unwrap();
    }

    #[test]
    fn new_project_cancels_preview_clears_workspace_and_preserves_saved_files() {
        let root =
            std::env::temp_dir().join(format!("cleanslate-new-project-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("voice.wav");
        write_wav(&source, &[0.0; 48_000]);
        let mut app = super::CleanSlateApp::default();
        app.add_stems([source.clone()]);
        finish_import(&mut app);
        let old_path = app.project.as_ref().unwrap().path().to_owned();
        let speaker = app.project.as_ref().unwrap().sources[0].speaker_id;
        app.project
            .as_mut()
            .unwrap()
            .add_reference_label(
                crate::TimeRange::new(0, crate::TICKS_PER_SECOND / 2).unwrap(),
                vec![speaker],
                &[],
            )
            .unwrap();
        app.timeline
            .set_selection(Some((0, crate::TICKS_PER_SECOND / 2)));
        let _ = app.update(super::Message::GainAction(0, false));
        app.timeline.seek(crate::TICKS_PER_SECOND / 4);
        app.track_name_draft = Some((0, "Draft".into()));
        app.selected_example = Some(1);
        app.reference_menu = Some((0, iced::Point::ORIGIN));
        app.gain_tool = true;
        app.solo_tracks = 1;
        app.audition_range = Some((0, crate::TICKS_PER_SECOND));
        app.preview = Some(crate::AnalysisPreview {
            stems: vec![source.clone()],
            regions: vec![],
            envelopes: vec![crate::GainEnvelope::constant(48_000, 1.0)],
            protections: vec![],
            parameters: Default::default(),
        });
        let project_bytes = std::fs::read(&old_path).unwrap();
        let source_bytes = std::fs::read(&source).unwrap();
        let old_render_generation = app.render_generation.clone();
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (_sender, result) = super::mpsc::channel();
        app.preview_processing = Some(super::PreviewProcessing {
            started_at: std::time::Instant::now(),
            snapshot: crate::EditorAnalysisIdentity {
                key: String::new(),
                project: Default::default(),
            },
            feedback: Default::default(),
            cancelled: cancelled.clone(),
            generation: 0,
            source_generation: 0,
            sources: vec![],
            result,
        });

        app.reset_project(false);
        assert!(app.project.is_some() && app.preview.is_some());
        assert!(app.track_name_draft.is_some());
        assert!(!cancelled.load(super::Ordering::Relaxed));
        app.reset_project(true);
        assert!(cancelled.load(super::Ordering::Relaxed));
        assert!(!std::sync::Arc::ptr_eq(
            &old_render_generation,
            &app.render_generation
        ));
        assert!(app.project.is_none() && app.waveforms.is_empty());
        assert!(app.review.is_none() && app.preview.is_none());
        assert!(app.supervised_result.is_none() && app.effective_result.is_none());
        assert!(app.preview_processing.is_none() && app.preview_pending.is_none());
        assert!(app.audition.is_none() && app.audition_range.is_none());
        assert!(!app.audition_preparation.is_busy());
        assert_eq!(app.timeline.playhead(), 0);
        assert!(app.timeline.selection().is_none());
        assert!(app.track_name_draft.is_none() && app.selected_example.is_none());
        assert!(app.reference_menu.is_none() && app.gain_menu.is_none());
        assert!(app.reference_speakers.is_empty() && app.gain_drag.is_none());
        assert!(app.gain_constraints.manual_edits.is_empty());
        assert!(app.gain_constraints.protections.is_empty());
        assert!(!app.gain_tool && app.solo_tracks == 0);
        assert!(app.pending_imports.is_empty() && app.import_choices.is_empty());
        assert!(app.import_due.is_none() && app.source_loading.is_none());
        let _ = app.update(super::Message::UndoGain);
        let _ = app.update(super::Message::UndoReferences);
        assert!(app.project.is_none());

        app.add_stems([source.clone()]);
        finish_import(&mut app);
        assert_ne!(app.project.as_ref().unwrap().path(), old_path);
        assert!(app.project.as_ref().unwrap().reference_labels().is_empty());
        assert!(
            app.project
                .as_ref()
                .unwrap()
                .gain_constraints()
                .manual_edits
                .is_empty()
        );
        assert_eq!(std::fs::read(old_path).unwrap(), project_bytes);
        assert_eq!(std::fs::read(source).unwrap(), source_bytes);
        app.stop_audition();
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn new_project_cancels_pending_imports_but_is_blocked_while_exporting() {
        let mut app = super::CleanSlateApp::default();
        app.pending_imports.push("queued.wav".into());
        assert!(app.can_start_new_project());
        app.reset_project(true);
        assert!(app.pending_imports.is_empty());
        app.pending_imports.clear();
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (_sender, result) = super::mpsc::channel();
        app.exporting = Some(super::ExportProcessing {
            cancelled: cancelled.clone(),
            result,
        });
        assert!(!app.can_start_new_project());
        app.reset_project(true);
        assert!(app.exporting.is_some());
        assert!(!cancelled.load(super::Ordering::Relaxed));
        app.exporting = None;
        assert!(app.can_start_new_project());
        let _ = app.update(super::Message::NewProject);
        assert!(app.project.is_none());
    }

    #[test]
    fn inline_track_names_validate_cancel_and_persist_without_playback_shortcuts() {
        use super::{CleanSlateApp, Message};
        let directory =
            std::env::temp_dir().join(format!("cleanslate-inline-name-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let audio = directory.join("voice.wav");
        write_wav(&audio, &[0.0; 16]);
        let mut app = CleanSlateApp::default();
        app.add_stems([audio]);
        finish_import(&mut app);
        let original = app.project.as_ref().unwrap().sources[0].display_name();
        let path = app.project.as_ref().unwrap().path().to_path_buf();
        let _ = app.update(Message::EditTrackName(0));
        let _ = app.update(Message::TrackName("Unsaved name".into()));
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Character("3".into()),
            iced::keyboard::Modifiers::default(),
        ));
        assert_eq!(app.audition_mode, super::AuditionMode::Original);
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::default(),
        ));
        assert!(app.track_name_draft.is_none());
        assert_eq!(
            app.project.as_ref().unwrap().sources[0].display_name(),
            original
        );
        let _ = app.update(Message::EditTrackName(0));
        let _ = app.update(Message::TrackName("  ".into()));
        let _ = app.update(Message::SaveTrackName);
        assert!(app.track_name_draft.is_some());
        assert!(!app.messages.is_empty());
        assert_eq!(
            app.project.as_ref().unwrap().sources[0].display_name(),
            original
        );
        let _ = app.update(Message::TrackName("  Alice  ".into()));
        let _ = app.update(Message::SaveTrackName);
        assert!(app.track_name_draft.is_none() && app.messages.is_empty());
        assert_eq!(
            app.project.as_ref().unwrap().sources[0].display_name(),
            "Alice"
        );
        assert_eq!(
            crate::Project::open(&path).unwrap().sources[0].display_name(),
            "Alice"
        );
        let _ = app.update(Message::EditTrackName(0));
        let _ = app.update(Message::TrackName("Discard this".into()));
        let _ = app.update(Message::CancelTrackName);
        assert_eq!(
            crate::Project::open(&path).unwrap().sources[0].display_name(),
            "Alice"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn editor_text_and_enabled_buttons_have_readable_contrast() {
        fn luminance(color: iced::Color) -> f32 {
            let linear = |channel: f32| {
                if channel <= 0.04045 {
                    channel / 12.92
                } else {
                    ((channel + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
        }
        let contrast = |foreground, background| {
            let a = luminance(foreground);
            let b = luminance(background);
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        };
        let theme = super::editor_theme();
        for background in [theme.palette().background, iced::color!(0x292D33)] {
            assert!(contrast(theme.palette().text, background) >= 4.5);
            assert!(contrast(iced::color!(0xADB5C0), background) >= 4.5);
        }
        for status in [
            iced::widget::button::Status::Active,
            iced::widget::button::Status::Hovered,
            iced::widget::button::Status::Pressed,
        ] {
            for style in [
                super::secondary_button(&theme, status),
                super::primary_button(&theme, status),
            ] {
                let Some(iced::Background::Color(background)) = style.background else {
                    panic!("Expected solid button background");
                };
                assert!(contrast(style.text_color, background) >= 4.5, "{status:?}");
            }
        }
        for appearance in [super::Appearance::Dark, super::Appearance::Light] {
            let theme = appearance.theme();
            let panel = super::colors(&theme).panel;
            for status in [
                iced::widget::button::Status::Active,
                iced::widget::button::Status::Hovered,
                iced::widget::button::Status::Pressed,
                iced::widget::button::Status::Disabled,
            ] {
                for selected in [false, true] {
                    let style = super::flat_button(&theme, status, selected);
                    assert_eq!(
                        style.border.width, 0.0,
                        "Flat controls must not regain outlines"
                    );
                    assert_eq!(style.shadow.blur_radius, 0.0);
                    let background = match style.background {
                        Some(iced::Background::Color(color)) => iced::Color::from_rgb(
                            color.r * color.a + panel.r * (1.0 - color.a),
                            color.g * color.a + panel.g * (1.0 - color.a),
                            color.b * color.a + panel.b * (1.0 - color.a),
                        ),
                        None => panel,
                        _ => panic!("Unexpected flat-control gradient"),
                    };
                    if status != iced::widget::button::Status::Disabled {
                        assert!(
                            contrast(style.text_color, background) >= 4.5,
                            "{appearance:?} {status:?}"
                        );
                    }
                }
            }
        }
    }

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
    fn locked_middle_pan_preserves_context_and_reanchors_only_once() {
        use super::{Message, PlaybackScroll, TimelineEvent, TimelineViewport};
        let root =
            std::env::temp_dir().join(format!("cleanslate-middle-pan-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("voice.wav");
        write_wav(&source, &[0.0; 100]);
        let path = root.join("session.cleanslate");
        let project = synchronized_project(&path, vec![source]).unwrap();
        let saved = std::fs::read(&path).unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            timeline: TimelineViewport::new(100 * crate::TICKS_PER_SECOND),
            playback_scroll: PlaybackScroll::Locked,
            selected_example: Some(7),
            selection_track: Some(0),
            selected_gain_point: Some((0, 8, 0)),
            ..Default::default()
        };
        app.timeline
            .fit_range(30 * crate::TICKS_PER_SECOND, 50 * crate::TICKS_PER_SECOND);
        app.timeline.seek(40 * crate::TICKS_PER_SECOND);
        app.timeline.lock_playhead();
        let selection = Some((crate::TICKS_PER_SECOND, 2 * crate::TICKS_PER_SECOND));
        app.timeline.set_selection(selection);
        let parameters = app.preview_parameters;
        let _ = app.update(Message::TrackTimeline(0, TimelineEvent::PanStart));
        let _ = app.update(Message::TrackTimeline(
            0,
            TimelineEvent::Pan {
                delta_x: -25.0,
                width: 100.0,
            },
        ));
        assert_eq!(app.playback_scroll, PlaybackScroll::Locked);
        assert!(app.timeline.is_middle_panning());
        let panned = app.timeline.start();
        app.timeline.seek(42 * crate::TICKS_PER_SECOND);
        assert!(!app.timeline.follow_playhead(app.playback_scroll));
        assert_eq!(app.timeline.start(), panned);
        let fraction = app.timeline.x_for_sample(app.timeline.playhead(), 100.0);
        let _ = app.update(Message::TrackTimeline(0, TimelineEvent::PanEnd));
        assert!(!app.timeline.is_middle_panning());
        assert_eq!(app.timeline.start(), panned);
        let _ = app.update(Message::Timeline(TimelineEvent::PanEnd));
        assert_eq!(app.timeline.start(), panned);
        app.timeline.seek(43 * crate::TICKS_PER_SECOND);
        app.timeline.follow_playhead(app.playback_scroll);
        assert!(
            (app.timeline.x_for_sample(app.timeline.playhead(), 100.0) - fraction).abs() < 0.001
        );
        assert_eq!(app.selected_example, Some(7));
        assert_eq!(app.selection_track, Some(0));
        assert_eq!(app.selected_gain_point, Some((0, 8, 0)));
        assert_eq!(app.timeline.selection(), selection);
        assert_eq!(app.preview_parameters, parameters);
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn middle_pan_cancellation_modes_and_explicit_unlock_are_authoritative() {
        use super::{Message, PlaybackScroll, TimelineEvent, TimelineViewport};
        for mode in [
            PlaybackScroll::Off,
            PlaybackScroll::Page,
            PlaybackScroll::Locked,
        ] {
            for cancel in [false, true] {
                let mut app = super::CleanSlateApp {
                    timeline: TimelineViewport::new(100 * crate::TICKS_PER_SECOND),
                    playback_scroll: mode,
                    ..Default::default()
                };
                app.timeline
                    .fit_range(30 * crate::TICKS_PER_SECOND, 50 * crate::TICKS_PER_SECOND);
                app.timeline.seek(40 * crate::TICKS_PER_SECOND);
                app.timeline.lock_playhead();
                let _ = app.update(Message::Timeline(TimelineEvent::PanStart));
                let _ = app.update(Message::Timeline(TimelineEvent::Pan {
                    delta_x: -25.0,
                    width: 100.0,
                }));
                let expected = mode;
                assert_eq!(app.playback_scroll, expected);
                if mode == PlaybackScroll::Locked && !cancel {
                    let _ = app.update(Message::LockPlayhead);
                    assert_eq!(app.playback_scroll, PlaybackScroll::Off);
                }
                let panned = app.timeline.start();
                app.timeline.seek(42 * crate::TICKS_PER_SECOND);
                app.app_dialog = Some(super::AppDialog::Settings);
                let event = if cancel {
                    TimelineEvent::PanCancel
                } else {
                    TimelineEvent::PanEnd
                };
                let _ = app.update(Message::Timeline(event));
                assert!(!app.timeline.is_middle_panning());
                if expected == PlaybackScroll::Locked && cancel {
                    assert_eq!(
                        app.timeline.x_for_sample(app.timeline.playhead(), 100.0),
                        50.0
                    );
                } else {
                    assert_eq!(
                        app.playback_scroll,
                        if mode == PlaybackScroll::Locked {
                            PlaybackScroll::Off
                        } else {
                            mode
                        }
                    );
                    assert_eq!(app.timeline.start(), panned);
                }
                assert_eq!(app.timeline.playhead(), 42 * crate::TICKS_PER_SECOND);
            }
        }
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
    fn shortcut_help_and_escape_respect_edit_owners_and_idle_gain_mode() {
        use super::Message;
        use iced::keyboard::{Key, Modifiers, key::Named};
        let mut app = super::CleanSlateApp::default();
        let before = (app.timeline.start(), app.timeline.visible_samples());
        let _ = app.update(Message::KeyPressed(
            Key::Character("f".into()),
            Modifiers::empty(),
        ));
        assert_eq!(
            (app.timeline.start(), app.timeline.visible_samples()),
            before
        );
        assert!(
            super::keyboard_message(
                Key::Character("/".into()),
                Modifiers::CTRL,
                iced::event::Status::Captured
            )
            .is_none()
        );
        app.timeline.set_selection(Some((1, 2)));
        app.gain_tool = true;
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Escape),
            Modifiers::empty(),
        ));
        assert_eq!(app.timeline.selection(), None);
        app.sound_typing = true;
        let _ = app.update(Message::KeyPressed(
            Key::Character("/".into()),
            Modifiers::CTRL,
        ));
        assert!(!app.shortcuts_open);
        app.sound_typing = false;
        app.track_name_draft = Some((0, "Typing".into()));
        let _ = app.update(Message::KeyPressed(
            Key::Character("/".into()),
            Modifiers::CTRL,
        ));
        assert!(!app.shortcuts_open);
        app.track_name_draft = None;
        app.import_dialog = true;
        let _ = app.update(Message::KeyPressed(
            Key::Character("/".into()),
            Modifiers::CTRL,
        ));
        assert!(!app.shortcuts_open);
        app.import_dialog = false;
        app.export_dialog = Some(super::ExportDialog {
            settings: crate::ExportSettings::default(),
            destination: None,
        });
        let _ = app.update(Message::KeyPressed(
            Key::Character("/".into()),
            Modifiers::CTRL,
        ));
        assert!(!app.shortcuts_open);
        app.export_dialog = None;
        app.reference_menu = Some((0, iced::Point::ORIGIN));
        app.timeline.set_selection(Some((1, 2)));
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Escape),
            Modifiers::empty(),
        ));
        assert!(app.reference_menu.is_none());
        assert_eq!(app.timeline.selection(), Some((1, 2)));
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Escape),
            Modifiers::empty(),
        ));
        assert_eq!(app.timeline.selection(), None);
        let _ = app.update(Message::KeyPressed(
            Key::Character("/".into()),
            Modifiers::CTRL,
        ));
        assert!(app.shortcuts_open);
        app.timeline.set_selection(Some((1, 2)));
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Escape),
            Modifiers::empty(),
        ));
        assert!(!app.shortcuts_open);
        assert_eq!(app.timeline.selection(), Some((1, 2)));
    }

    #[test]
    fn help_menu_is_keyboard_reachable_and_blocks_workspace_navigation() {
        use super::Message;
        use iced::keyboard::{Key, Modifiers, key::Named};
        let mut app = super::CleanSlateApp::default();
        let _ = app.update(Message::KeyPressed(
            Key::Character("h".into()),
            Modifiers::ALT,
        ));
        assert!(app.help_open);
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Tab),
            Modifiers::empty(),
        ));
        assert!(app.help_close_selected);
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::ArrowUp),
            Modifiers::empty(),
        ));
        assert!(!app.help_close_selected);
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Enter),
            Modifiers::empty(),
        ));
        assert!(app.shortcuts_open && !app.help_open);
        app.timeline.set_selection(Some((1, 2)));
        let _ = app.update(Message::Timeline(crate::timeline::TimelineEvent::Select(
            3, 4,
        )));
        let _ = app.update(Message::KeyPressed(
            Key::Character("f".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.timeline.selection(), Some((1, 2)));
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Enter),
            Modifiers::empty(),
        ));
        assert!(!app.shortcuts_open);
    }

    #[test]
    fn fit_keys_and_gain_alt_drag_preserve_session_decisions_and_playback() {
        use super::Message;
        use iced::{
            Point, Rectangle, Size,
            keyboard::{Key, Modifiers},
            mouse,
            widget::canvas::{self, Program},
        };
        let root = std::env::temp_dir().join(format!("cleanslate-fit-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("source.wav");
        write_wav(&source, &vec![0.0; 48000 * 100]);
        let mut app = super::CleanSlateApp {
            gain_tool: true,
            project: Some(
                synchronized_project(root.join("session.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        app.timeline
            .set_total_samples(100 * crate::TICKS_PER_SECOND);
        app.timeline.seek(8 * crate::TICKS_PER_SECOND);
        app.timeline.set_selection(Some((
            40 * crate::TICKS_PER_SECOND,
            50 * crate::TICKS_PER_SECOND,
        )));
        let _ = app.update(Message::GainAction(0, false));
        assert!(!app.project.as_ref().unwrap().manual_gain_edits().is_empty());
        app.selection_track = Some(0);
        app.selected_example = Some(77);
        app.audition_range = Some((2, 3));
        app.playback_scroll = crate::timeline::PlaybackScroll::Locked;
        let saved = std::fs::read(app.project.as_ref().unwrap().path()).unwrap();
        let _ = app.update(Message::KeyPressed(
            Key::Character("f".into()),
            Modifiers::SHIFT,
        ));
        assert_eq!(
            app.timeline.visible_samples(),
            12.0 * crate::TICKS_PER_SECOND as f64
        );
        assert_eq!(app.timeline.start(), 2.0 * crate::TICKS_PER_SECOND as f64);
        assert_eq!(app.playback_scroll, crate::timeline::PlaybackScroll::Locked);
        let _ = app.update(Message::KeyPressed(
            Key::Character("f".into()),
            Modifiers::empty(),
        ));
        assert_eq!(
            app.timeline.visible_samples(),
            100.0 * crate::TICKS_PER_SECOND as f64
        );
        let project = app.project.as_ref().unwrap();
        let constraints = project.gain_constraints();
        assert!(
            crate::hit_gain_point(
                &constraints,
                project.sources[0].track_id,
                &app.timeline,
                crate::GainPointer {
                    x: 40.0,
                    y: 11.0,
                    width: 100.0,
                    height: 20.0
                }
            )
            .is_some()
        );
        let gain = crate::gain_editor::GainCanvas {
            revision: 0,
            project,
            constraints: &constraints,
            track: project.sources[0].track_id,
            viewport: &app.timeline,
            automatic: None,
            effective: None,
            editing: true,
            selected_point: None,
            on_event: Box::new(|event| event),
        };
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(100.0, 20.0));
        let mut state = crate::gain_editor::GainCanvasState::default();
        let point = mouse::Cursor::Available(Point::new(40.0, 11.0));
        assert_eq!(
            gain.mouse_interaction(&state, bounds, point),
            mouse::Interaction::Grab
        );
        let hover = gain
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(40.0, 11.0),
                }),
                bounds,
                point,
            )
            .unwrap()
            .into_inner();
        assert!(hover.0.is_none());
        gain.update(
            &mut state,
            &canvas::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(Modifiers::ALT)),
            bounds,
            mouse::Cursor::Unavailable,
        );
        let start = gain
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(40.0, 11.0)),
            )
            .unwrap()
            .into_inner();
        assert!(start.0.is_none());
        let end = gain
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(50.0, 8.0)),
            )
            .unwrap()
            .into_inner();
        let Some(crate::GainEditorEvent::FitRange(start, end)) = end.0 else {
            panic!("Alt must fit instead of editing gain");
        };
        drop(gain);
        let selection = app.timeline.selection();
        let _ = app.update(Message::GainEvent(
            0,
            crate::GainEditorEvent::FitRange(start, end),
        ));
        let _ = app.update(Message::TrackTimeline(
            0,
            crate::timeline::TimelineEvent::FitRange(start, end),
        ));
        assert_eq!(app.timeline.selection(), selection);
        assert_eq!(app.selected_example, Some(77));
        assert_eq!(app.selection_track, Some(0));
        assert_eq!(app.timeline.playhead(), 8 * crate::TICKS_PER_SECOND);
        assert_eq!(app.audition_range, Some((2, 3)));
        assert_eq!(
            std::fs::read(app.project.as_ref().unwrap().path()).unwrap(),
            saved
        );
        app.audition_range = None;
        std::fs::remove_dir_all(root).unwrap();
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
        assert_eq!(app.drawing_revision, 2);
        let _ = app.update(super::Message::PlaybackFrame);
        assert_eq!(app.drawing_revision, 2);
    }

    fn drawn_waveform(
        app: &super::CleanSlateApp,
        state: &crate::waveform::TimelineCanvasState,
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
    ) -> std::sync::Arc<[iced_tiny_skia::Primitive]> {
        use iced::widget::canvas::{Geometry, Program};
        let (waveform, viewport) = app
            .source_loading
            .as_ref()
            .map_or((&app.waveforms[0], &app.timeline), |loading| {
                (&loading.waveforms[0], &loading.timeline)
            });
        let canvas = super::TimelineCanvas::waveform(
            waveform,
            viewport,
            super::Message::Timeline,
            app.preview.as_ref().map(|preview| (preview, 0)),
            false,
        )
        .with_display(app.waveform_display)
        .with_revision(app.drawing_revision);
        let geometry = canvas.draw(
            state,
            renderer,
            theme,
            bounds,
            iced::mouse::Cursor::Unavailable,
        );
        let Geometry::<iced::Renderer>::Secondary(iced_tiny_skia::geometry::Geometry::Cache(
            cached,
        )) = &geometry[0]
        else {
            panic!("expected cached software waveform geometry");
        };
        assert!(!cached.primitives.is_empty());
        cached.primitives.clone()
    }

    #[test]
    fn transport_buttons_keep_equal_sizes_and_center_at_compact_and_wide_widths() {
        fn collect(
            node: &iced::advanced::layout::Node,
            offset: iced::Vector,
            centers: &mut Vec<f32>,
        ) {
            let bounds = node.bounds();
            let origin = offset + iced::Vector::new(bounds.x, bounds.y);
            if bounds.size() == iced::Size::new(40.0, 36.0) {
                centers.push(origin.x + bounds.width / 2.0);
            }
            for child in node.children() {
                collect(child, origin, centers);
            }
        }
        let app = super::CleanSlateApp::default();
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
        ));
        for width in [1024.0, 1390.0, 1600.0, 2560.0] {
            let mut content = app.workspace(super::compact_workspace(width), 900.0, width);
            let mut tree = iced::advanced::widget::Tree::new(content.as_widget());
            let layout = content.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &iced::advanced::layout::Limits::new(
                    iced::Size::ZERO,
                    iced::Size::new(width, 900.0),
                ),
            );
            let mut centers = vec![];
            collect(&layout, iced::Vector::ZERO, &mut centers);
            centers.sort_by(f32::total_cmp);
            centers.dedup();
            assert_eq!(centers.len(), 3, "transport at width {width}: {centers:?}");
            assert!(
                (centers[1] - width / 2.0).abs() < 1.0,
                "width {width}: {centers:?}"
            );
            assert!((centers[1] - centers[0] - 48.0).abs() < 1.0);
            assert!((centers[2] - centers[1] - 48.0).abs() < 1.0);
        }
    }

    #[test]
    fn waveform_display_modes_change_geometry_without_changing_audio_or_reusing_stale_cache() {
        use crate::waveform::WaveformDisplay;
        let mut builder = crate::waveform::WaveformBuilder::default();
        builder.push(&[0.5; 4096]);
        let waveform = builder.finish(48_000);
        let mut app = super::CleanSlateApp {
            timeline: crate::timeline::TimelineViewport::new(waveform.duration()),
            waveforms: vec![waveform],
            preview: Some(crate::AnalysisPreview {
                stems: vec![],
                regions: vec![],
                protections: vec![],
                envelopes: vec![crate::GainEnvelope::constant_at_rate(4096, 48_000, 0.25).unwrap()],
                parameters: Default::default(),
            }),
            ..Default::default()
        };
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
        ));
        let state = crate::waveform::TimelineCanvasState::default();
        let bounds = iced::Rectangle::with_size(iced::Size::new(640.0, 120.0));
        let mut shapes = vec![];
        let audio_mode = app.audition_mode;
        let parameters = app.preview_parameters;
        for mode in [
            WaveformDisplay::Original,
            WaveformDisplay::Processed,
            WaveformDisplay::Both,
            WaveformDisplay::Original,
        ] {
            let _ = app.update(super::Message::WaveformDisplay(mode));
            let cached = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
            let fresh = drawn_waveform(
                &app,
                &Default::default(),
                &renderer,
                &super::editor_theme(),
                bounds,
            );
            assert_eq!(format!("{cached:?}"), format!("{fresh:?}"));
            shapes.push(format!("{cached:?}"));
            assert_eq!(app.audition_mode, audio_mode);
            assert_eq!(app.preview_parameters, parameters);
        }
        assert_ne!(shapes[0], shapes[1]);
        assert_ne!(shapes[1], shapes[2]);
        assert_ne!(shapes[0], shapes[2]);
        assert_eq!(shapes[0], shapes[3]);
        app.preview = None;
        app.drawing_revision += 1;
        let original = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
        app.waveform_display = WaveformDisplay::Processed;
        let unity = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
        assert_eq!(format!("{original:?}"), format!("{unity:?}"));
    }

    #[test]
    fn follow_canvases_rebuild_truthfully_across_themes_and_logical_scales() {
        use iced::widget::canvas::{Geometry, Program};
        use std::sync::Arc;
        let root =
            std::env::temp_dir().join(format!("cleanslate-follow-colors-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("voice.wav");
        write_wav(&source, &[0.25; 4096]);
        let project = synchronized_project(root.join("session.cleanslate"), vec![source]).unwrap();
        let envelope = crate::GainEnvelope::constant_at_rate(4096, 48_000, 0.25).unwrap();
        let mut builder = crate::waveform::WaveformBuilder::default();
        builder.push(&[0.25; 4096]);
        let waveform = builder.finish(48_000);
        let mut app = super::CleanSlateApp {
            timeline: crate::timeline::TimelineViewport::new(waveform.duration()),
            waveforms: vec![waveform],
            project: Some(project),
            ..Default::default()
        };
        app.preview = Some(crate::AnalysisPreview {
            stems: app.project.as_ref().unwrap().source_paths(),
            regions: vec![],
            envelopes: vec![envelope.clone()],
            protections: vec![],
            parameters: app.project.as_ref().unwrap().parameters(),
        });
        let project = app.project.as_ref().unwrap();
        let constraints = project.gain_constraints();
        let gain = crate::gain_editor::GainCanvas {
            revision: 0,
            project,
            constraints: &constraints,
            track: project.sources[0].track_id,
            viewport: &app.timeline,
            automatic: None,
            effective: Some(&envelope),
            editing: true,
            selected_point: None,
            on_event: Box::new(|_| ()),
        };
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
        ));
        let waveform_state = crate::waveform::TimelineCanvasState::default();
        let gain_state = crate::gain_editor::GainCanvasState::default();
        let mut previous_waveform = None;
        let mut previous_gain = None;
        for (appearance, scale) in [
            (crate::appearance::Appearance::Dark, 1.0),
            (crate::appearance::Appearance::Light, 1.0),
            (crate::appearance::Appearance::Light, 2.0),
            (crate::appearance::Appearance::Dark, 2.0),
            (crate::appearance::Appearance::Dark, 1.0),
        ] {
            let theme = appearance.theme();
            let bounds = iced::Rectangle::with_size(iced::Size::new(640.0 * scale, 120.0 * scale));
            let waveform = drawn_waveform(&app, &waveform_state, &renderer, &theme, bounds);
            let fresh = drawn_waveform(&app, &Default::default(), &renderer, &theme, bounds);
            assert_eq!(format!("{waveform:?}"), format!("{fresh:?}"));
            assert!(Arc::ptr_eq(
                &waveform,
                &drawn_waveform(&app, &waveform_state, &renderer, &theme, bounds)
            ));
            if let Some(previous) = previous_waveform.replace(waveform.clone()) {
                assert!(!Arc::ptr_eq(&previous, &waveform));
                assert_ne!(format!("{previous:?}"), format!("{waveform:?}"));
            }
            let bounds = iced::Rectangle::with_size(iced::Size::new(640.0 * scale, 64.0 * scale));
            let draw = |state| {
                gain.draw(
                    state,
                    &renderer,
                    &theme,
                    bounds,
                    iced::mouse::Cursor::Unavailable,
                )
            };
            let retained = draw(&gain_state);
            let fresh_state = Default::default();
            let fresh = draw(&fresh_state);
            assert_eq!(format!("{retained:?}"), format!("{fresh:?}"));
            let Geometry::<iced::Renderer>::Secondary(iced_tiny_skia::geometry::Geometry::Cache(
                cached,
            )) = &retained[0]
            else {
                panic!("expected cached gain canvas");
            };
            assert_eq!(cached.clip_bounds.size(), bounds.size());
            let repeated = draw(&gain_state);
            let Geometry::<iced::Renderer>::Secondary(iced_tiny_skia::geometry::Geometry::Cache(
                repeated,
            )) = &repeated[0]
            else {
                panic!("expected repeated gain canvas");
            };
            assert!(Arc::ptr_eq(&cached.primitives, &repeated.primitives));
            if let Some(previous) = previous_gain.replace(cached.primitives.clone()) {
                assert!(!Arc::ptr_eq(&previous, &cached.primitives));
                assert_ne!(format!("{previous:?}"), format!("{:?}", cached.primitives));
            }
            let curve = crate::visible_gain_curve(&app.timeline, bounds.width, &envelope);
            assert_eq!((curve[0].time, curve[0].gain), (0, 0.25));
            assert_eq!(
                (curve.last().unwrap().time, curve.last().unwrap().gain),
                (envelope.duration(), 1.0)
            );
        }
        drop(gain);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn visibility_observations_preserve_canvas_and_background_job_delivery() {
        let mut app = super::CleanSlateApp::default();
        let mut builder = crate::waveform::WaveformBuilder::default();
        builder.push(&[0.25; 4096]);
        app.waveforms.push(builder.finish(48_000));
        app.timeline.set_total_samples(app.waveforms[0].duration());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
        ));
        let state = crate::waveform::TimelineCanvasState::default();
        let bounds = iced::Rectangle::with_size(iced::Size::new(640.0, 120.0));
        let before = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
        let (sender, result) = std::sync::mpsc::channel();
        app.source_inspection = Some(super::SourceInspection {
            paths: vec![],
            result,
            cancelled: Default::default(),
        });
        // Unsupported state and real minimize/restore observations do not change source geometry.
        for minimized in [None, Some(true), Some(false)] {
            let _ = app.update(super::Message::WindowVisibilityChecked(0, minimized));
            let _ = app.update(super::Message::PlaybackFrame);
            let after = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
            assert!(std::sync::Arc::ptr_eq(&before, &after));
        }
        let _ = app.update(super::Message::WindowVisibilityChecked(0, Some(true)));
        sender.send(Err("hidden source failure".into())).unwrap();
        let _ = app.update(super::Message::Tick);
        assert!(app.source_inspection.is_none());
        assert!(
            app.messages
                .iter()
                .any(|message| message.contains("hidden source failure"))
        );
    }

    #[test]
    fn unchanged_worker_polls_retain_the_drawn_waveform() {
        let mut app = super::CleanSlateApp::default();
        let mut builder = crate::waveform::WaveformBuilder::default();
        builder.push(&[0.25; 4096]);
        app.waveforms.push(builder.finish(48_000));
        app.timeline.set_total_samples(app.waveforms[0].duration());
        let (_sender, result) = std::sync::mpsc::channel();
        app.source_inspection = Some(super::SourceInspection {
            paths: vec![],
            result,
            cancelled: Default::default(),
        });
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
        ));
        let state = crate::waveform::TimelineCanvasState::default();
        let bounds = iced::Rectangle::with_size(iced::Size::new(640.0, 120.0));
        let mut previous = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
        let started = std::time::Instant::now();
        let mut rebuilds = 0;
        for _ in 0..100 {
            let _ = app.update(super::Message::Tick);
            let current = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
            rebuilds += usize::from(!std::sync::Arc::ptr_eq(&previous, &current));
            previous = current;
        }
        eprintln!(
            "NOOP_POLL iterations=100 geometry_rebuilds={rebuilds} elapsed_us={}",
            started.elapsed().as_micros()
        );
        assert_eq!(
            rebuilds, 0,
            "unchanged worker polls must retain static drawing"
        );
        assert!(app.source_inspection.is_some());
    }

    #[test]
    fn polled_source_waveforms_replace_cached_drawing_and_failure_restores_it() {
        let mut app = super::CleanSlateApp::default();
        let mut builder = crate::waveform::WaveformBuilder::default();
        builder.push(&[0.25; 4096]);
        app.waveforms.push(builder.finish(48_000));
        app.timeline.set_total_samples(app.waveforms[0].duration());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
        ));
        let state = crate::waveform::TimelineCanvasState::default();
        let bounds = iced::Rectangle::with_size(iced::Size::new(640.0, 120.0));
        let previous = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
        let (sender, events) = std::sync::mpsc::channel();
        app.source_loading = Some(super::SourceLoading {
            paths: vec![],
            waveforms: vec![],
            timeline: app.timeline.clone(),
            stage: "Loading waveforms",
            events,
            publish: None,
            committing: false,
        });
        let mut builder = crate::waveform::WaveformBuilder::default();
        builder.push(&[-0.5; 4096]);
        sender
            .send(super::SourceEvent::Waveform(0, builder.finish(48_000)))
            .unwrap();
        let _ = app.update(super::Message::Tick);
        let loading = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
        assert!(!std::sync::Arc::ptr_eq(&previous, &loading));
        let fresh = drawn_waveform(
            &app,
            &Default::default(),
            &renderer,
            &super::editor_theme(),
            bounds,
        );
        assert_eq!(format!("{loading:?}"), format!("{fresh:?}"));
        assert_ne!(format!("{previous:?}"), format!("{loading:?}"));

        sender
            .send(super::SourceEvent::Stage("Checking sources"))
            .unwrap();
        let _ = app.update(super::Message::Tick);
        let status_only = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
        assert!(std::sync::Arc::ptr_eq(&loading, &status_only));

        sender
            .send(super::SourceEvent::Finished(Err(
                "fixture source failure".into()
            )))
            .unwrap();
        let _ = app.update(super::Message::Tick);
        assert!(app.source_loading.is_none());
        let restored = drawn_waveform(&app, &state, &renderer, &super::editor_theme(), bounds);
        assert!(!std::sync::Arc::ptr_eq(&loading, &restored));
        assert_eq!(format!("{previous:?}"), format!("{restored:?}"));
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
            "Speaker activity uncertain"
        );
        assert_eq!(
            super::ownership_summary(&speakers, &[Owned, UncertainUnity]),
            "Active: Speaker 3 · other speakers uncertain"
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
    fn gain_hints_follow_effective_bindings_and_unbinding() {
        use super::{CleanSlateApp, Message};
        use crate::shortcuts::Command;
        use iced::keyboard::{Key, Modifiers};
        let mut app = CleanSlateApp::default();
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::CaptureShortcut(Command::Earlier));
        let _ = app.update(Message::KeyPressed(
            Key::Character("j".into()),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::ApplyShortcut);
        let _ = app.update(Message::UnbindShortcut(Command::DeleteGain));
        assert_eq!(
            app.gain_key_hint(),
            "Drag pink points to adjust gain and fades.\nEarlier: J; later: Right; up: Up; down: Down; remove: Unbound."
        );
    }

    #[test]
    fn dialog_navigation_owns_a_remapped_reference_key() {
        use super::{CleanSlateApp, Message};
        use crate::shortcuts::Command;
        use iced::keyboard::{Key, Modifiers, key::Named};
        let mut app = CleanSlateApp::default();
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::CaptureShortcut(Command::Reference));
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::PageDown),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::ApplyShortcut);
        for dialog in [Message::Settings, Message::About, Message::Help] {
            let _ = app.update(Message::CloseHelp);
            let _ = app.update(dialog);
            let _ = app.update(Message::KeyPressed(
                Key::Named(Named::PageDown),
                Modifiers::empty(),
            ));
            assert!(
                !app.shortcuts_open,
                "Dialog scrolling must retain input ownership"
            );
            assert!(app.app_dialog.is_some() || app.help_open);
        }
        let _ = app.update(Message::CloseHelp);
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::PageDown),
            Modifiers::empty(),
        ));
        assert!(
            app.shortcuts_open,
            "The remapped binding still works in the workspace"
        );
    }

    #[test]
    fn plus_binding_has_an_unambiguous_display_name() {
        use super::{CleanSlateApp, Message};
        use crate::shortcuts::Command;
        use iced::keyboard::{Key, Modifiers};
        let mut app = CleanSlateApp::default();
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::CaptureShortcut(Command::SaveExample));
        let _ = app.update(Message::KeyPressed(
            Key::Character("+".into()),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::ApplyShortcut);
        assert_eq!(app.shortcuts.hint(Command::SaveExample), "Plus");
    }

    #[test]
    fn remapped_popup_save_uses_validation_for_room_tone_edits_and_enter() {
        use super::{CleanSlateApp, Message, TimelineEvent};
        use crate::shortcuts::Command;
        use iced::keyboard::{Key, Modifiers, key::Named};
        fn remap(app: &mut CleanSlateApp, command: Command, key: &str) {
            let _ = app.update(Message::Settings);
            let _ = app.update(Message::CaptureShortcut(command));
            let _ = app.update(Message::KeyPressed(
                Key::Character(key.into()),
                Modifiers::empty(),
            ));
            let _ = app.update(Message::ApplyShortcut);
            assert!(app.shortcut_capture.is_none(), "{:?}", app.app_feedback);
            let _ = app.update(Message::CloseHelp);
        }
        let root = crate::audio_io::multichannel_tests::root();
        let source = root.join("voice.wav");
        write_wav(&source, &vec![0.0; 2 * 48_000]);
        let mut app = CleanSlateApp {
            project: Some(
                synchronized_project(root.join("session.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        let t = crate::TICKS_PER_SECOND;
        app.timeline.set_total_samples(2 * t);
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::CaptureShortcut(Command::SaveExample));
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::ApplyShortcut);
        let _ = app.update(Message::CloseHelp);
        let _ = app.update(Message::CompleteTrackSelection(
            0,
            t / 2,
            t,
            iced::Point::ORIGIN,
        ));
        let _ = app.update(Message::KeyPressed(
            Key::Character("s".into()),
            Modifiers::empty(),
        ));
        assert!(app.project.as_ref().unwrap().reference_labels().is_empty());
        let _ = app.update(Message::KeyPressed(
            Key::Character("s".into()),
            if cfg!(target_os = "macos") {
                Modifiers::LOGO
            } else {
                Modifiers::CTRL
            },
        ));
        assert!(app.messages.iter().any(|m| m == "Editable project saved."));
        assert!(app.reference_menu.is_some());
        app.sound_typing = true;
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        assert!(app.project.as_ref().unwrap().reference_labels().is_empty());
        app.sound_typing = false;
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::CTRL,
        ));
        assert!(
            app.project.as_ref().unwrap().reference_labels().is_empty(),
            "Chords stay distinct"
        );
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        let id = app.project.as_ref().unwrap().reference_labels()[0].id;
        let _ = app.update(Message::OpenTimelineMenu(0, t / 2, iced::Point::ORIGIN));
        let _ = app.update(Message::KeyPressed(
            Key::Character("r".into()),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        let labels = app.project.as_ref().unwrap().reference_labels();
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].id, id);
        assert!(
            labels[0].active_speakers.is_empty(),
            "Editing can save room tone"
        );
        let _ = app.update(Message::CompleteTrackSelection(
            0,
            t,
            t + 1,
            iced::Point::ORIGIN,
        ));
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        assert!(app.reference_menu.is_some() && app.calibration_error.is_some());
        assert_eq!(app.project.as_ref().unwrap().reference_labels().len(), 1);
        let _ = app.update(Message::DismissReferenceMenu);
        let _ = app.update(Message::CompleteTrackSelection(
            0,
            t,
            t + t / 2,
            iced::Point::ORIGIN,
        ));
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Enter),
            Modifiers::empty(),
        ));
        assert_eq!(app.project.as_ref().unwrap().reference_labels().len(), 2);
        assert!(app.reference_menu.is_none());
        let _ = app.update(Message::KeyPressed(
            Key::Character("1".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.audition_mode, crate::AuditionMode::Original);
        let _ = app.update(Message::Timeline(TimelineEvent::Seek(0)));
        remap(&mut app, Command::UndoLabels, "u");
        let _ = app.update(Message::KeyPressed(
            Key::Character("u".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.project.as_ref().unwrap().reference_labels().len(), 1);
        let _ = app.update(Message::GainTool(true));
        let _ = app.update(Message::GainAction(0, false));
        assert_eq!(app.project.as_ref().unwrap().manual_gain_edits().len(), 1);
        remap(&mut app, Command::UndoGain, "u");
        let _ = app.update(Message::KeyPressed(
            Key::Character("u".into()),
            Modifiers::empty(),
        ));
        assert!(app.project.as_ref().unwrap().manual_gain_edits().is_empty());
        remap(&mut app, Command::FitRecording, "g");
        app.timeline.zoom_at(0.0, 1000.0, 1.0);
        let visible = app.timeline.visible_samples();
        let _ = app.update(Message::KeyPressed(
            Key::Character("f".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.timeline.visible_samples(), visible);
        let _ = app.update(Message::KeyPressed(
            Key::Character("g".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.timeline.visible_samples(), (2 * t) as f64);
        remap(&mut app, Command::Original, "o");
        app.audition_mode = crate::AuditionMode::Removed;
        let _ = app.update(Message::KeyPressed(
            Key::Character("1".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.audition_mode, crate::AuditionMode::Removed);
        let _ = app.update(Message::KeyPressed(
            Key::Character("o".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.audition_mode, crate::AuditionMode::Original);
        remap(&mut app, Command::Start, "h");
        app.timeline.set_selection(Some((t / 2, t)));
        app.timeline.seek(t + t / 2);
        let _ = app.update(Message::KeyPressed(
            Key::Character("h".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.timeline.playhead(), t / 2);
        app.stop_audition();
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repeated_or_captured_keyboard_events_cannot_toggle_commands() {
        use iced::keyboard::{
            Event, Key, Location, Modifiers,
            key::{Code, Physical},
        };
        let event = |repeat| Event::KeyPressed {
            key: Key::Character("l".into()),
            modified_key: Key::Character("l".into()),
            physical_key: Physical::Code(Code::KeyL),
            location: Location::Standard,
            modifiers: Modifiers::empty(),
            text: Some("l".into()),
            repeat,
        };
        let mut app = super::CleanSlateApp::default();
        if let Some(message) =
            super::application_keyboard_event(event(false), iced::event::Status::Ignored)
        {
            let _ = app.update(message);
        }
        assert_eq!(app.playback_scroll, super::PlaybackScroll::Locked);
        assert!(
            super::application_keyboard_event(event(true), iced::event::Status::Ignored).is_none()
        );
        assert!(
            super::application_keyboard_event(event(false), iced::event::Status::Captured)
                .is_none()
        );
        assert_eq!(app.playback_scroll, super::PlaybackScroll::Locked);
    }

    #[test]
    fn every_user_zoom_route_obeys_local_lock_policy_without_editing_the_session() {
        use super::{CleanSlateApp, Message, PlaybackScroll, TimelineEvent, TimelineViewport};
        use iced::keyboard::{Key, Modifiers};
        let root = crate::audio_io::multichannel_tests::root();
        let source = root.join("voice.wav");
        write_wav(&source, &vec![0.0; 4 * 48_000]);
        let path = root.join("session.cleanslate");
        synchronized_project(&path, vec![source]).unwrap();
        let saved = std::fs::read(&path).unwrap();
        let t = crate::TICKS_PER_SECOND;
        let zoom = TimelineEvent::Zoom {
            cursor_x: 250.0,
            width: 1000.0,
            direction: 1.0,
        };
        let routes = [
            Message::Timeline(zoom),
            Message::TrackTimeline(0, zoom),
            Message::Timeline(TimelineEvent::FitRange(t, 2 * t)),
            Message::TrackTimeline(0, TimelineEvent::FitRange(t, 2 * t)),
            Message::GainEvent(0, crate::GainEditorEvent::FitRange(t, 2 * t)),
            Message::KeyPressed(Key::Character("f".into()), Modifiers::empty()),
            Message::KeyPressed(Key::Character("f".into()), Modifiers::SHIFT),
        ];
        for unlock in [false, true] {
            for mode in [
                PlaybackScroll::Off,
                PlaybackScroll::Page,
                PlaybackScroll::Locked,
            ] {
                let locked = mode == PlaybackScroll::Locked;
                for (index, route) in routes.iter().enumerate() {
                    let mut app = CleanSlateApp {
                        project: Some(crate::Project::open(&path).unwrap()),
                        timeline: TimelineViewport::new(4 * t),
                        gain_tool: index == 4,
                        unlock_playhead_when_zooming: unlock,
                        audition_range: Some((t, 2 * t)),
                        playback_scroll: if locked { PlaybackScroll::Off } else { mode },
                        ..Default::default()
                    };
                    app.timeline.zoom_at(500.0, 1000.0, 1.0);
                    app.timeline.seek(2 * t);
                    app.timeline.set_selection(Some((t, 2 * t)));
                    if locked {
                        let _ = app.update(Message::LockPlayhead);
                    }
                    let _ = app.update(route.clone());
                    assert_eq!(
                        app.playback_scroll,
                        if locked && !unlock {
                            PlaybackScroll::Locked
                        } else if locked {
                            PlaybackScroll::Off
                        } else {
                            mode
                        },
                        "route {index}, unlock={unlock}, locked={locked}"
                    );
                    assert_eq!(app.timeline.playhead(), 2 * t);
                    assert_eq!(app.timeline.selection(), Some((t, 2 * t)));
                    assert_eq!(app.audition_range, Some((t, 2 * t)));
                    if locked && !unlock {
                        assert!(
                            (app.timeline.x_for_sample(2 * t, 1000.0) - 500.0).abs() < 0.01,
                            "route {index} retains lock anchor"
                        );
                    }
                }
            }
        }
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shortcut_preferences_persist_restore_and_preserve_unrelated_settings() {
        use super::{CleanSlateApp, Message};
        use crate::shortcuts::Command;
        use iced::keyboard::{Key, Modifiers};
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("preferences.json");
        std::fs::write(
            &path,
            r#"{"appearance":"Dark","future_setting":{"keep":true}}"#,
        )
        .unwrap();
        let mut app = CleanSlateApp {
            preferences_path: Some(path.clone()),
            ..Default::default()
        };
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::CaptureShortcut(Command::SaveExample));
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::ApplyShortcut);
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["shortcuts"]["SaveExample"]["key"], "x");
        let (reloaded, unlock, error) = crate::appearance::load_shortcuts(&path);
        assert_eq!(reloaded.hint(Command::SaveExample), "X");
        assert!(!unlock && error.is_none());
        app.reset_project(true);
        assert_eq!(app.shortcuts.hint(Command::SaveExample), "X");
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::Appearance(crate::appearance::Appearance::Light));
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["future_setting"]["keep"], true);
        assert_eq!(saved["shortcuts"]["SaveExample"]["key"], "x");
        let _ = app.update(Message::RestoreShortcut(Command::SaveExample));
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert!(saved["shortcuts"].get("SaveExample").is_none());
        let _ = app.update(Message::UnbindShortcut(Command::ClearSpeakers));
        let _ = app.update(Message::RestoreAllShortcuts);
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["shortcuts"], serde_json::json!({}));
        assert_eq!(saved["appearance"], "Light");
        let _ = app.update(Message::UnlockOnZoom(true));
        let (_, unlock, error) = crate::appearance::load_shortcuts(&path);
        assert!(unlock && error.is_none());
        app.reset_project(true);
        assert!(app.unlock_playhead_when_zooming);
        let invalid = r#"{"appearance":"Light","shortcuts":{"SaveExample":{"key":"Escape"},"ClearSpeakers":{"key":"x"},"Center":{"key":"f"},"Obsolete":{"key":"j"}},"unlock_playhead_when_zooming":"bad","future_setting":3}"#;
        std::fs::write(&path, invalid).unwrap();
        let (reloaded, unlock, error) = crate::appearance::load_shortcuts(&path);
        assert_eq!(reloaded.hint(Command::SaveExample), "S");
        assert_eq!(reloaded.hint(Command::ClearSpeakers), "X");
        assert_eq!(reloaded.hint(Command::Center), "C");
        assert!(!unlock);
        let error = error.unwrap();
        assert!(
            error.contains("SaveExample")
                && error.contains("Obsolete")
                && error.contains("zooming")
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), invalid);
        assert_eq!(
            crate::appearance::load(&path).0,
            crate::appearance::Appearance::Light
        );
        std::fs::write(&path, "{").unwrap();
        assert!(crate::appearance::save(&path, crate::appearance::Appearance::Dark).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shortcut_capture_previews_checks_conflicts_and_dispatches_effective_bindings() {
        use super::{CleanSlateApp, Message};
        use crate::shortcuts::Command;
        use iced::keyboard::{Key, Modifiers, key::Named};
        let mut app = CleanSlateApp::default();
        let _ = app.update(Message::Settings);
        let _ = app.update(Message::CaptureShortcut(Command::SaveExample));
        let _ = app.update(Message::KeyPressed(
            Key::Character("r".into()),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::ApplyShortcut);
        assert!(
            app.app_feedback
                .as_ref()
                .unwrap()
                .contains("Clear speakers")
        );
        assert_eq!(app.shortcuts.hint(Command::SaveExample), "S");
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        assert_eq!(
            app.shortcuts.hint(Command::SaveExample),
            "S",
            "Capture is a preview"
        );
        let _ = app.update(Message::ApplyShortcut);
        assert_eq!(app.shortcuts.hint(Command::SaveExample), "X");
        let _ = app.update(Message::CaptureShortcut(Command::Center));
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::ApplyShortcut);
        assert_eq!(
            app.shortcuts.hint(Command::Center),
            "X",
            "Popup and workspace are exclusive"
        );
        let _ = app.update(Message::CaptureShortcut(Command::Lock));
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        let _ = app.update(Message::ApplyShortcut);
        assert!(
            app.app_feedback
                .as_ref()
                .unwrap()
                .contains("Center playhead")
        );
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Tab),
            Modifiers::empty(),
        ));
        assert!(
            app.app_feedback
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("reserved")
        );
        let _ = app.update(Message::KeyPressed(
            Key::Character("s".into()),
            if cfg!(target_os = "macos") {
                Modifiers::LOGO
            } else {
                Modifiers::CTRL
            },
        ));
        let _ = app.update(Message::ApplyShortcut);
        assert_eq!(app.shortcuts.hint(Command::Lock), "L");
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::Escape),
            Modifiers::empty(),
        ));
        assert_eq!(app.shortcuts.hint(Command::Lock), "L");
        assert!(app.app_dialog.is_some(), "Escape cancels capture only");
        let _ = app.update(Message::CloseHelp);
        let t = crate::TICKS_PER_SECOND;
        app.timeline = super::TimelineViewport::new(100 * t);
        app.timeline.zoom_at(0.0, 1000.0, 1.0);
        app.timeline.seek(50 * t);
        app.timeline.set_selection(Some((t, 2 * t)));
        let _ = app.update(Message::KeyPressed(
            Key::Character("x".into()),
            Modifiers::empty(),
        ));
        assert!((app.timeline.x_for_sample(50 * t, 1000.0) - 500.0).abs() < 0.01);
        assert_eq!(app.timeline.playhead(), 50 * t);
        assert_eq!(app.timeline.selection(), Some((t, 2 * t)));
        assert_eq!(app.playback_scroll, super::PlaybackScroll::Page);
        let _ = app.update(Message::KeyPressed(
            Key::Character("l".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.playback_scroll, super::PlaybackScroll::Locked);
        app.sound_typing = true;
        let _ = app.update(Message::KeyPressed(
            Key::Character("l".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.playback_scroll, super::PlaybackScroll::Locked);
    }

    #[test]
    fn default_s_saves_one_popup_example_without_leaking_to_workspace() {
        use super::{CleanSlateApp, Message};
        use iced::keyboard::{Key, Modifiers};
        let root = crate::audio_io::multichannel_tests::root();
        let source = root.join("voice.wav");
        write_wav(&source, &vec![0.0; 48_000]);
        let mut app = CleanSlateApp {
            project: Some(
                synchronized_project(root.join("session.cleanslate"), vec![source]).unwrap(),
            ),
            ..Default::default()
        };
        app.timeline.set_total_samples(crate::TICKS_PER_SECOND);
        let _ = app.update(Message::CompleteTrackSelection(
            0,
            crate::TICKS_PER_SECOND / 4,
            crate::TICKS_PER_SECOND * 3 / 4,
            iced::Point::ORIGIN,
        ));
        let _ = app.update(Message::KeyPressed(
            Key::Character("s".into()),
            Modifiers::empty(),
        ));
        assert!(app.reference_menu.is_none());
        assert_eq!(app.project.as_ref().unwrap().reference_labels().len(), 1);
        let _ = app.update(Message::KeyPressed(
            Key::Character("s".into()),
            Modifiers::empty(),
        ));
        assert_eq!(app.project.as_ref().unwrap().reference_labels().len(), 1);
        assert!(app.messages.iter().all(|m| m != "Editable project saved."));
        app.stop_audition();
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
        app.audition_mode = crate::AuditionMode::Processed;
        app.solo_tracks = 5;
        let _ = app.update(Message::CompleteTrackSelection(0, start, end, point));
        assert_eq!(app.reference_menu, Some((0, point)));
        assert_eq!(app.audition_range, Some((start, end)));
        assert_eq!(app.audition_mode, crate::AuditionMode::Original);
        assert_eq!(app.solo_tracks, 0);
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
        assert!(app.audition_range.is_none() && app.audition.is_none());
        assert_eq!(app.audition_mode, crate::AuditionMode::Processed);
        assert_eq!(app.solo_tracks, 5);
        assert_eq!(app.timeline.selection(), Some((start, end)));
        assert!(app.reference_stale && app.preview_pending.is_none());
        assert_eq!(
            app.project.as_ref().unwrap().reference_labels()[0].active_speakers,
            vec![speakers[1]]
        );
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        let _ = app.update(Message::ContextReference(vec![], false));
        assert!(app.calibration_error.is_some() && app.reference_menu.is_some());
        assert_eq!(app.audition_range, Some((start, end)));
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
    fn popup_audition_restores_monitoring_and_rejects_obsolete_preparation() {
        use super::Message;
        let root =
            std::env::temp_dir().join(format!("cleanslate-popup-loop-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("voice.wav");
        write_wav(&source, &vec![0.1; 48_000]);
        let mut app = super::CleanSlateApp {
            project: Some(
                synchronized_project(root.join("session.cleanslate"), vec![source.clone()])
                    .unwrap(),
            ),
            audition_mode: crate::AuditionMode::Removed,
            solo_tracks: 1,
            monitor_db: -12.0,
            ..Default::default()
        };
        app.timeline.set_total_samples(crate::TICKS_PER_SECOND);
        let range = (crate::TICKS_PER_SECOND / 4, crate::TICKS_PER_SECOND / 2);
        let point = iced::Point::ORIGIN;
        let space = || {
            Message::KeyPressed(
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Space),
                iced::keyboard::Modifiers::empty(),
            )
        };
        let open = |app: &mut super::CleanSlateApp| {
            let _ = app.update(Message::CompleteTrackSelection(0, range.0, range.1, point));
        };
        let drain = |app: &mut super::CleanSlateApp| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while app.audition_preparation.is_busy() {
                app.poll_audition();
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
            assert!(app.audition.is_none());
        };
        open(&mut app);
        assert_eq!(app.audition_range, Some(range));
        assert_eq!(app.monitor_db, -12.0);
        let _ = app.update(Message::Removed);
        let _ = app.update(Message::SoloTrack(0));
        assert_eq!(app.audition_mode, crate::AuditionMode::Original);
        assert_eq!(app.solo_tracks, 0);
        let _ = app.update(space());
        assert!(app.audition_range.is_none() && app.reference_menu.is_some());
        assert!(app.reference_audition.as_ref().unwrap().paused);
        drain(&mut app);
        let _ = app.update(space());
        assert_eq!(app.audition_range, Some(range));
        assert!(!app.reference_audition.as_ref().unwrap().paused);
        let _ = app.update(Message::Stop);
        drain(&mut app);
        let _ = app.update(space());
        let _ = app.update(Message::OpenReferenceMenu(0, point));
        assert_eq!(app.audition_range, Some(range));
        let _ = app.update(Message::DismissReferenceMenu);
        drain(&mut app);
        assert_eq!(app.audition_mode, crate::AuditionMode::Removed);
        assert_eq!(app.solo_tracks, 1);
        assert_eq!(app.timeline.selection(), Some(range));
        open(&mut app);
        app.source_generation += 1;
        app.poll_audition();
        drain(&mut app);
        assert!(app.reference_menu.is_none());
        open(&mut app);
        app.timeline.set_selection(Some((range.0, range.1 + 1)));
        app.poll_audition();
        drain(&mut app);
        assert!(app.reference_menu.is_none());
        open(&mut app);
        app.install_waveforms(vec![]);
        drain(&mut app);
        assert!(app.reference_menu.is_none());
        assert_eq!(app.audition_mode, crate::AuditionMode::Removed);
        assert_eq!(
            app.solo_tracks, 1,
            "Invalidation restores the popup snapshot"
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
        assert_eq!(app.workspace_status(), "Add speaker and room-tone examples");
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
        let saved = std::fs::read(app.project.as_ref().unwrap().path()).unwrap();
        let _ = app.update(super::Message::GainEvent(
            0,
            crate::GainEditorEvent::Begin(id, 0),
        ));
        assert!(
            app.gain_drag.is_none(),
            "The hidden strip cannot receive gain input"
        );
        let _ = app.update(super::Message::GainTool(true));
        let _ = app.update(super::Message::GainTool(false));
        let _ = app.update(super::Message::GainTool(true));
        assert_eq!(app.project.as_ref().unwrap().gain_constraints(), before);
        assert_eq!(
            std::fs::read(app.project.as_ref().unwrap().path()).unwrap(),
            saved
        );
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
            feedback: Default::default(),
            cancelled: cancelled.clone(),
            generation: 0,
            source_generation: 0,
            sources: vec![],
            result,
        });
        let _ = app.update(super::Message::FileDropped(source));
        assert!(
            !cancelled.load(super::Ordering::Relaxed),
            "metadata inspection does not cancel accepted-track analysis"
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
    fn imported_media_directory_is_untouched_and_project_is_local() {
        let root = crate::audio_io::multichannel_tests::root();
        let source = root.join("read-only.wav");
        write_wav(&source, &[0.125; 4800]);
        let original = std::fs::read(&source).unwrap();
        let original_permissions = std::fs::metadata(&source).unwrap().permissions();
        let mut permissions = original_permissions.clone();
        permissions.set_readonly(true);
        std::fs::set_permissions(&source, permissions.clone()).unwrap();
        let mut app = super::CleanSlateApp::default();
        app.add_stems([source.clone()]);
        finish_import(&mut app);
        let project = app.project.as_ref().expect("read-only media must import");
        assert_eq!(project.sources[0].path, source);
        assert_ne!(project.path().parent(), Some(root.as_path()));
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        assert_eq!(std::fs::read(&source).unwrap(), original);
        app.audition_preparation.cancel();
        std::fs::set_permissions(&source, original_permissions).unwrap();
        std::fs::remove_dir_all(root).unwrap();
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
        assert!(app.project.is_some(), "{:?}", app.messages);
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
        for index in 0..app.import_choices.len() {
            let _ = app.update(super::Message::ImportChannel(index, false));
        }
        let _ = app.update(super::Message::ConfirmChannels);
        assert!(app.messages[0].contains("at most eight"));
        assert!(app.source_loading.is_none());
        for index in 0..app.import_choices.len() {
            let _ = app.update(super::Message::ImportChannel(index, true));
        }
        let _ = app.update(super::Message::ImportChannelName(renamed, "x".repeat(257)));
        let _ = app.update(super::Message::ConfirmChannels);
        assert!(app.messages[0].contains("1 to 256 bytes"));
        assert!(app.source_loading.is_none());
        let _ = app.import_settings_view(300.0);
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
        app.import_choices
            .extend(
                crate::wav_channel_sources(&poly)
                    .unwrap()
                    .into_iter()
                    .map(|source| super::ImportChoice {
                        source,
                        selected: true,
                    }),
            );
        for choice in &mut app.import_choices {
            choice.source.path = root.join("additional-poly.wav");
        }
        let _ = app.update(super::Message::ConfirmChannels);
        assert!(app.messages[0].contains("at most eight"));
        assert!(app.source_loading.is_none());
        assert_eq!(
            std::fs::read(app.project.as_ref().unwrap().path()).unwrap(),
            saved
        );
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

    #[test]
    fn progressive_rows_preserve_edits_order_readiness_and_retry_current_sources() {
        let root = crate::audio_io::multichannel_tests::root();
        let a = root.join("a.wav");
        let b = root.join("b.wav");
        let c = root.join("c.wav");
        write_wav(&a, &[0.0; 4800]);
        write_wav(&b, &[0.0; 16]);
        write_wav(&c, &[0.0; 4800]);
        let mut app = super::CleanSlateApp::default();
        app.begin_source_loading(super::SourceRequest::AddChannels(vec![
            crate::AudioSource::mono(a.clone()),
        ]));
        assert!(!app.import_dialog);
        assert_eq!(app.import_jobs.len(), 1);
        assert_eq!(app.import_jobs[0].waveforms[0].sample_count, 4800);
        assert!(app.project.is_none());
        finish_import(&mut app);
        let project_path = app.project.as_ref().unwrap().path().to_owned();
        app.timeline.set_selection(Some((1, 2)));
        app.timeline.seek(1);
        app.begin_source_loading(super::SourceRequest::AddChannels(vec![
            crate::AudioSource::mono(b.clone()),
            crate::AudioSource::mono(c.clone()),
        ]));
        assert_eq!(app.reserved_track_count(), 3);
        assert_eq!(app.timeline.selection(), Some((1, 2)));
        assert_eq!(app.timeline.playhead(), 1);
        app.project
            .as_mut()
            .unwrap()
            .rename_track(0, "Edited during import".into())
            .unwrap();
        let _ = app.update(super::Message::Attenuation(-3.0));
        finish_import(&mut app);
        assert_eq!(app.project.as_ref().unwrap().sources.len(), 2);
        assert_eq!(
            app.project.as_ref().unwrap().sources[0].display_name(),
            "Edited during import"
        );
        assert_eq!(
            app.project.as_ref().unwrap().parameters().attenuation_db,
            -3.0
        );
        assert!(app.import_jobs[1].error.is_some());
        assert!(app.import_jobs[2].accepted);
        let saved = std::fs::read(&project_path).unwrap();
        assert_eq!(
            crate::Project::open(&project_path).unwrap().source_paths(),
            [a, c]
        );
        let failed_id = app.import_jobs[1].id;
        write_wav(&b, &[0.25; 4800]);
        let _ = app.update(super::Message::RetryImport(failed_id));
        finish_import(&mut app);
        assert!(app.import_jobs[1].accepted);
        assert_eq!(app.project.as_ref().unwrap().sources.len(), 3);
        let rows = app.import_display_rows();
        assert_eq!(
            rows.iter().map(|r| r.0.unwrap()).collect::<Vec<_>>(),
            [0, 2, 1]
        );
        assert_eq!(app.export_source_order(), [0, 2, 1]);
        assert_ne!(std::fs::read(&project_path).unwrap(), saved);
        app.stop_audition();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provisional_reservations_limit_cancel_and_new_project_reject_late_results() {
        let root = crate::audio_io::multichannel_tests::root();
        let inputs = (0..8)
            .map(|index| {
                let path = root.join(format!("queued-{index}.wav"));
                write_wav(&path, &[0.1; 4800]);
                crate::AudioSource::mono(path)
            })
            .collect::<Vec<_>>();
        let mut app = super::CleanSlateApp::default();
        app.reserve_import(inputs.clone());
        assert_eq!(app.reserved_track_count(), 8);
        let flag = app.import_jobs[0].cancelled.clone();
        app.reserve_import(vec![inputs[0].clone()]);
        assert_eq!(app.import_jobs.len(), 8);
        assert!(app.messages[0].contains("eight-track"));
        let _ = app.update(super::Message::CancelPendingImports);
        assert!(flag.load(super::Ordering::Relaxed));
        assert!(app.import_jobs.is_empty());
        assert!(app.project.is_none());
        app.reserve_import(inputs);
        let flag = app.import_jobs[0].cancelled.clone();
        app.reset_project(true);
        assert!(flag.load(super::Ordering::Relaxed));
        app.poll_sources();
        assert!(app.project.is_none());
        assert!(app.import_jobs.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn overall_import_status_names_active_work_and_queued_recordings() {
        let root = crate::audio_io::multichannel_tests::root();
        let inputs = ["first.wav", "second.wav"].map(|name| {
            let path = root.join(name);
            write_wav(&path, &[0.1; 4800]);
            crate::AudioSource::mono(path)
        });
        let mut app = super::CleanSlateApp::default();
        app.reserve_import(inputs.to_vec());
        let status = app.workspace_status();
        assert!(status.contains("2 pending"));
        assert!(status.contains("1 queued"));
        assert!(status.contains("Sampling whole-recording preview"));
        app.cancel_pending_imports();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "measures local full recording; use an isolated LOCALAPPDATA cache root"]
    fn progressive_import_stage_benchmark() {
        let source = std::env::var_os("CLEANSLATE_IMPORT_BENCHMARK").expect("set recording path");
        let source = std::path::PathBuf::from(source);
        let root = crate::audio_io::multichannel_tests::root();
        let baseline = std::env::var_os("CLEANSLATE_IMPORT_BASELINE").is_some();
        for warm in [false, true] {
            let started = std::time::Instant::now();
            let inputs = crate::wav_channel_sources(&source).unwrap();
            let choices_ms = started.elapsed().as_secs_f64() * 1000.0;
            let mut preview_ms = None;
            let mut ready_ms = None;
            let mut stages = vec![];
            if baseline {
                let (sender, receiver) = super::mpsc::channel();
                let parameters = crate::AutomixParameters::default();
                let worker = std::thread::spawn(move || {
                    super::load_sources(
                        super::SourceRequest::AddChannels(inputs),
                        None,
                        parameters,
                        &sender,
                    )
                });
                for event in receiver {
                    let ms = started.elapsed().as_secs_f64() * 1000.0;
                    match event {
                        super::SourceEvent::Stage(stage) => stages.push((stage, ms)),
                        super::SourceEvent::Waveform(_, _) => {
                            preview_ms.get_or_insert(ms);
                        }
                        _ => {}
                    }
                }
                let loaded = worker.join().unwrap().unwrap();
                ready_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
                assert_eq!(loaded.project.sources.len(), 3);
            } else {
                let metadata = inputs.iter().map(|s| s.metadata(None).unwrap()).collect();
                let mut job = crate::import::Job::from_metadata(1, inputs, metadata);
                let skeleton_ms = started.elapsed().as_secs_f64() * 1000.0;
                stages.push(("Skeleton", skeleton_ms));
                job.start();
                let receiver = job.events.take().unwrap();
                for event in receiver {
                    let ms = started.elapsed().as_secs_f64() * 1000.0;
                    match event {
                        crate::import::Event::Preview(_) => {
                            preview_ms.get_or_insert(ms);
                        }
                        crate::import::Event::Ready(identities, _) => {
                            let project = crate::Project::accept_recording(
                                None,
                                root.join(format!("bench-{warm}.cleanslate")),
                                identities,
                                crate::AutomixParameters::default(),
                            )
                            .unwrap();
                            assert_eq!(project.sources.len(), 3);
                            ready_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
                        }
                        crate::import::Event::Finished(result) => result.unwrap(),
                        crate::import::Event::Progress(stage, _, _)
                            if stages.last().is_none_or(|(previous, _)| *previous != stage) =>
                        {
                            stages.push((stage, ms));
                        }
                        _ => {}
                    }
                }
            }
            println!(
                "IMPORT_BENCH {}",
                serde_json::json!({"baseline":baseline,"warm":warm,"choices_ms":choices_ms,"first_preview_ms":preview_ms,"ready_ms":ready_ms,"total_ms":started.elapsed().as_secs_f64()*1000.0,"stages":stages})
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "measures cancellation on the local owner recording"]
    fn progressive_import_cancellation_benchmark() {
        let path = std::path::PathBuf::from(
            std::env::var_os("CLEANSLATE_IMPORT_BENCHMARK").expect("set recording path"),
        );
        for phase in ["preview", "identity", "validation"] {
            let inputs = crate::wav_channel_sources(&path).unwrap();
            let metadata = inputs.iter().map(|s| s.metadata(None).unwrap()).collect();
            let mut job = crate::import::Job::from_metadata(1, inputs, metadata);
            job.start();
            let receiver = job.events.take().unwrap();
            let mut cancelled_at = None;
            for event in receiver {
                let cancel_now = match &event {
                    crate::import::Event::Preview(_) => phase == "preview",
                    crate::import::Event::Progress(stage, _, _) => match phase {
                        "identity" => stage.starts_with("Checking source identity"),
                        "validation" => stage.starts_with("Validating audio"),
                        _ => false,
                    },
                    _ => false,
                };
                if cancel_now && cancelled_at.is_none() {
                    cancelled_at = Some(std::time::Instant::now());
                    job.cancelled.store(true, super::Ordering::Relaxed);
                }
                assert!(!matches!(event, crate::import::Event::Ready(_, _)));
                if let crate::import::Event::Finished(result) = event {
                    assert!(result.is_err());
                    let elapsed = cancelled_at.expect("requested phase was reached").elapsed();
                    println!(
                        "IMPORT_CANCEL phase={phase} latency_ms={:.3}",
                        elapsed.as_secs_f64() * 1000.0
                    );
                }
            }
        }
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
            publish: None,
            committing: false,
        });
        sender
            .send(super::SourceEvent::Waveform(0, app.waveforms[0].clone()))
            .unwrap();
        app.poll_sources();
        assert_eq!(app.source_loading.as_ref().unwrap().waveforms.len(), 1);
        assert_eq!(app.project.as_ref().unwrap().sources.len(), 2);
        let _ = app.workspace(false, 900.0, 960.0);
        drop(sender);
        app.poll_sources();
        assert!(!app.sources_busy());
        assert!(app.messages[0].contains("stopped unexpectedly"));
        assert_eq!(app.waveforms.len(), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn prepared_import_does_not_publish_before_acceptance() {
        let root =
            std::env::temp_dir().join(format!("cleanslate-import-commit-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let a = root.join("a.wav");
        let b = root.join("b.wav");
        write_wav(&a, &[0.0; 16]);
        write_wav(&b, &[0.0; 16]);
        let project = crate::Project::create(root.join("prior.cleanslate"), vec![a]).unwrap();
        let saved = std::fs::read(project.path()).unwrap();
        let (sender, _) = super::mpsc::channel();
        let loaded = super::load_sources(
            super::SourceRequest::AddChannels(vec![crate::AudioSource::mono(b)]),
            Some(project.clone()),
            project.parameters(),
            &sender,
        )
        .unwrap();
        assert_eq!(loaded.project.sources.len(), 2);
        assert_eq!(
            std::fs::read(project.path()).unwrap(),
            saved,
            "Preparing a result that Cancel may discard must not publish it"
        );
        let mut app = super::CleanSlateApp {
            project: Some(project.clone()),
            ..Default::default()
        };
        let (late, events) = super::mpsc::channel();
        let (publish, accepted) = super::mpsc::channel();
        app.source_loading = Some(super::SourceLoading {
            paths: loaded.project.source_paths(),
            waveforms: vec![],
            timeline: Default::default(),
            stage: "Ready",
            events,
            publish: Some(publish),
            committing: false,
        });
        let _ = app.update(super::Message::CancelChannels);
        assert!(accepted.recv().is_err());
        assert!(
            late.send(super::SourceEvent::Finished(Ok(Box::new(loaded))))
                .is_err()
        );
        app.poll_sources();
        assert_eq!(app.project.as_ref().unwrap().sources.len(), 1);
        assert_eq!(
            crate::Project::open(project.path()).unwrap().sources.len(),
            1
        );
        assert_eq!(std::fs::read(project.path()).unwrap(), saved);
        app.add_stems([root.join("b.wav")]);
        finish_import(&mut app);
        assert_eq!(app.project.as_ref().unwrap().sources.len(), 2);
        assert_eq!(
            crate::Project::open(project.path()).unwrap().sources.len(),
            2
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepted_import_finishes_atomically_and_reports_save_failure_in_modal() {
        let root = std::env::temp_dir().join(format!(
            "cleanslate-import-save-error-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("source.wav");
        write_wav(&source, &[0.0; 16]);
        let project = crate::Project::create(root.join("prior.cleanslate"), vec![source]).unwrap();
        let saved = std::fs::read(project.path()).unwrap();
        let mut app = super::CleanSlateApp {
            import_dialog: true,
            project: Some(project.clone()),
            ..Default::default()
        };
        let (sender, events) = super::mpsc::channel();
        let (publish, accepted) = super::mpsc::channel();
        app.source_loading = Some(super::SourceLoading {
            paths: vec![],
            waveforms: vec![],
            timeline: Default::default(),
            stage: "Ready",
            events,
            publish: Some(publish),
            committing: false,
        });
        sender.send(super::SourceEvent::Prepared).unwrap();
        app.poll_sources();
        accepted.try_recv().unwrap();
        let _ = app.update(super::Message::CancelChannels);
        assert!(app.source_loading.as_ref().unwrap().committing);
        sender
            .send(super::SourceEvent::Finished(Err(
                "Project save failed: access denied".into(),
            )))
            .unwrap();
        app.poll_sources();
        assert_eq!(
            app.project.as_ref().unwrap().source_paths(),
            project.source_paths()
        );
        assert_eq!(std::fs::read(project.path()).unwrap(), saved);
        assert!(app.import_dialog);
        assert!(app.messages[0].contains("access denied"));
        let _ = app.import_settings_view(300.0);
        let _ = app.update(super::Message::CancelChannels);
        assert!(!app.import_dialog);
        assert_eq!(app.project.as_ref().unwrap().sources.len(), 1);
        app.audition_preparation.cancel();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn import_modal_blocks_workspace_and_escape_cancels_pending_work() {
        let mut app = super::CleanSlateApp::default();
        app.add_stems([std::path::PathBuf::from("pending.wav")]);
        let parameters = app.preview_parameters;
        let _ = app.update(super::Message::Attenuation(-6.0));
        assert_eq!(app.preview_parameters, parameters);
        let _ = app.update(super::Message::Timeline(
            crate::timeline::TimelineEvent::Select(1, 2),
        ));
        assert!(app.timeline.selection().is_none());
        let _ = app.update(super::Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::empty(),
        ));
        assert!(
            !app.sources_busy(),
            "Escape cancels queued import as well as channel choices"
        );
        assert!(app.import_due.is_none());
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
    fn typed_sound_drafts_commit_cancel_validate_and_reset_individually() {
        use super::{Message, SoundParameter};
        let directory =
            std::env::temp_dir().join(format!("cleanslate-sound-ui-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("voice.wav");
        write_wav(&source, &[0.0; 100]);
        let project =
            synchronized_project(directory.join("episode.cleanslate"), vec![source]).unwrap();
        let mut app = super::CleanSlateApp {
            project: Some(project),
            ..Default::default()
        };
        for (parameter, value) in [
            (SoundParameter::Attenuation, "-120"),
            (SoundParameter::Open, "0"),
            (SoundParameter::Close, "60000"),
            (SoundParameter::Gap, "12.125"),
        ] {
            let _ = app.update(Message::EditSound(parameter));
            let _ = app.update(Message::SoundText(value.into()));
            assert_ne!(
                parameter
                    .value(app.project.as_ref().unwrap().parameters())
                    .to_string(),
                value
            );
            let _ = app.update(Message::CommitSound);
            assert_eq!(
                parameter.value(app.project.as_ref().unwrap().parameters()),
                value.parse::<f32>().unwrap()
            );
            assert!(app.sound_draft.is_none());
        }
        let accepted = app.preview_parameters;
        let _ = app.update(Message::EditSound(SoundParameter::Open));
        let _ = app.update(Message::SoundText("42".into()));
        let _ = app.update(Message::CancelSound);
        assert_eq!(app.preview_parameters, accepted);
        for invalid in ["NaN", "inf", "abc", "-1", "60001"] {
            let _ = app.update(Message::EditSound(SoundParameter::Open));
            let _ = app.update(Message::SoundText(invalid.into()));
            let _ = app.update(Message::SoundTab(false));
            assert!(app.sound_error.is_some() && app.sound_draft.is_some());
            assert!(!app.sound_typing);
            assert_eq!(app.preview_parameters, accepted);
        }
        let _ = app.update(Message::SoundText("123.25".into()));
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Character("3".into()),
            iced::keyboard::Modifiers::default(),
        ));
        assert_eq!(app.audition_mode, super::AuditionMode::Original);
        let _ = app.update(Message::SoundTab(false));
        assert_eq!(app.preview_parameters.open_ms, 123.25);
        let _ = app.update(Message::Mute);
        assert!(app.project.as_ref().unwrap().parameters().mute);
        let _ = app.workspace(false, 900.0, 1400.0);
        assert_eq!(app.preview_parameters.attenuation_db, -120.0);
        let _ = app.update(Message::ResetSound(SoundParameter::Attenuation));
        assert_eq!(app.preview_parameters.attenuation_db, -18.0);
        assert!(!app.preview_parameters.mute);
        assert_eq!(app.preview_parameters.open_ms, 123.25);
        let _ = app.update(Message::SoundModifiers(iced::keyboard::Modifiers::CTRL));
        let _ = app.update(Message::EditSound(SoundParameter::Open));
        assert_eq!(app.preview_parameters.open_ms, 50.0);
        assert!(!app.reference_analyze_requested && app.preview_processing.is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn inspector_stacks_only_in_compact_windows() {
        assert!(!super::compact_workspace(1280.0));
        assert!(super::compact_workspace(1000.0));
        assert!(!super::compact_workspace(1600.0));
    }

    #[test]
    fn waveform_rows_fill_height_without_becoming_unusable() {}

    #[test]
    fn file_dialog_starts_in_an_existing_local_directory() {
        assert!(super::dialog_start_directory().is_dir());
    }

    #[test]
    fn export_folders_never_overwrite_existing_downloads() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-path-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("CleanSlate export"), b"existing").unwrap();
        assert_eq!(
            super::available_export_path(&directory),
            directory.join("CleanSlate export 2")
        );
        assert_eq!(
            std::fs::read(directory.join("CleanSlate export")).unwrap(),
            b"existing"
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
        assert!(app.messages[0].starts_with("Prepare WAV files"));
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

#[cfg(test)]
mod app_menu_tests {
    use super::*;

    #[test]
    fn version_copy_is_allowlisted_and_failures_stay_actionable() {
        assert_eq!(
            version_info("1.2.3", None, "windows", "x86_64"),
            "CleanSlate 1.2.3\nPlatform: windows (x86_64)"
        );
        assert_eq!(
            version_info("1.2.3", Some("abc123"), "linux", "aarch64"),
            "CleanSlate 1.2.3\nBuild: abc123\nPlatform: linux (aarch64)"
        );
        let mut app = CleanSlateApp::default();
        let _ = app.update(Message::About);
        let _ = app.update(Message::CopyVersion);
        let request = app.copy_request.as_ref().unwrap().clone();
        let _ = app.update(Message::VersionInfoReady(
            request.clone(),
            "Observed report".into(),
        ));
        let _ = app.update(Message::VersionCopied(request, false));
        assert!(app.app_feedback.as_ref().unwrap().contains("manually"));
        assert!(app.copy_request.is_none());
        let _ = app.update(Message::CopyVersion);
        let request = app.copy_request.as_ref().unwrap().clone();
        let _ = app.update(Message::VersionCopied(request, true));
        assert_eq!(
            app.app_feedback.as_deref(),
            Some("Version information copied.")
        );
        let _ = app.update(Message::CloseHelp);
        let _ = app.update(Message::Menu);
        let _ = app.update(Message::LinkOpened(
            AppLink::ReportBug,
            Err("No browser".into()),
        ));
        assert!(
            app.app_feedback
                .as_ref()
                .unwrap()
                .contains("https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose")
        );
    }

    #[test]
    fn closing_about_before_copy_finishes_does_not_disable_future_copy() {
        let mut app = CleanSlateApp::default();
        let _ = app.update(Message::About);
        let _ = app.update(Message::CopyVersion);
        let request = app.copy_request.as_ref().unwrap().clone();
        let _ = app.update(Message::CloseHelp);
        let _ = app.update(Message::KeyboardShortcuts);
        let _ = app.update(Message::VersionCopied(request, false));
        assert!(app.copy_request.is_none());
        assert!(app.shortcuts_open);
    }

    #[test]
    fn copied_version_report_includes_observed_graphics_and_explicit_unavailable_fields() {
        let report = version_info_with_graphics(
            version_info("1.2.3", None, "windows", "x86_64"),
            "Direct3D12\r\n",
            "Example Adapter",
            Some(1.25),
        );
        assert_eq!(
            report,
            "CleanSlate 1.2.3\nPlatform: windows (x86_64)\nGraphics backend: Direct3D12\nGraphics adapter: Example Adapter\nWindow scale: 1.25x\nDisplay refresh: unavailable (current GUI API)"
        );
        let unavailable = version_info_with_graphics(
            version_info("1.2.3", None, "windows", "x86_64"),
            "",
            "C:/Users/private/session.wav",
            Some(f32::NAN),
        );
        assert!(unavailable.contains("Graphics backend: unavailable"));
        assert!(unavailable.contains("Graphics adapter: unavailable"));
        assert!(unavailable.contains("Window scale: unavailable"));
        assert!(!unavailable.contains("private"));
        assert!(!unavailable.contains("session.wav"));
    }

    #[test]
    fn late_clipboard_reply_cannot_finish_a_reopened_about_copy() {
        let mut app = CleanSlateApp::default();
        let _ = app.update(Message::About);
        let _ = app.update(Message::CopyVersion);
        let first = app.copy_request.as_ref().unwrap().clone();
        let _ = app.update(Message::CloseHelp);
        let _ = app.update(Message::About);
        let _ = app.update(Message::CopyVersion);
        let second = app.copy_request.as_ref().unwrap().clone();
        let stale = app.update(Message::VersionInfoReady(
            first.clone(),
            "Old report".into(),
        ));
        assert_eq!(
            stale.units(),
            0,
            "a stale report attempted a clipboard write"
        );
        let _ = app.update(Message::VersionCopied(first, true));
        assert!(
            app.copy_request
                .as_ref()
                .is_some_and(|request| Arc::ptr_eq(request, &second))
        );
        assert!(app.version_details.is_none());
        let _ = app.update(Message::VersionInfoReady(
            second.clone(),
            "Current observed report".into(),
        ));
        assert_eq!(
            app.version_details.as_deref(),
            Some("Current observed report")
        );
        let _ = app.update(Message::VersionCopied(second, true));
        assert!(app.copy_request.is_none());
        assert_eq!(
            app.app_feedback.as_deref(),
            Some("Version information copied.")
        );
    }

    #[test]
    fn copied_hardware_fields_cannot_hide_paths_or_exceed_the_report_bound() {
        let report = version_info_with_graphics(
            "CleanSlate test".into(),
            "C:\u{0}/Users/private/session.wav",
            &"δ".repeat(400),
            None,
        );
        assert_eq!(
            report,
            format!(
                "CleanSlate test\nGraphics backend: unavailable\nGraphics adapter: {}\nWindow scale: unavailable\nDisplay refresh: unavailable (current GUI API)",
                "δ".repeat(160),
            ),
        );
    }

    #[test]
    fn menu_reuses_reference_and_about_keyboard_owns_navigation() {
        let mut app = CleanSlateApp::default();
        let _ = app.update(Message::KeyPressed(
            keyboard::Key::Character("m".into()),
            keyboard::Modifiers::ALT,
        ));
        assert!(app.menu_open);
        let _ = app.update(Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::ArrowDown),
            keyboard::Modifiers::empty(),
        ));
        let _ = app.update(Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Enter),
            keyboard::Modifiers::empty(),
        ));
        assert!(app.shortcuts_open && !app.menu_open);
        let _ = app.update(Message::CloseHelp);
        let _ = app.update(Message::About);
        app.timeline.set_selection(Some((1, 2)));
        let _ = app.update(Message::KeyPressed(
            keyboard::Key::Character("f".into()),
            keyboard::Modifiers::empty(),
        ));
        assert_eq!(app.timeline.selection(), Some((1, 2)));
        let _ = app.update(Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Tab),
            keyboard::Modifiers::empty(),
        ));
        let _ = app.update(Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Enter),
            keyboard::Modifiers::empty(),
        ));
        assert!(app.app_dialog.is_none());
        assert_eq!(
            AppLink::Help.url(),
            "https://github.com/Today20092/podcast-bleed-reducer/blob/main/docs/editor-workflow.md"
        );
        assert_eq!(
            AppLink::GitHub.url(),
            "https://github.com/Today20092/podcast-bleed-reducer"
        );
    }

    #[test]
    fn menu_routes_settings_and_restores_workspace_without_edits() {
        let mut app = CleanSlateApp::default();
        app.timeline.set_selection(Some((1, 2)));
        let _ = app.update(Message::Menu);
        let _ = app.update(Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Enter),
            keyboard::Modifiers::empty(),
        ));
        assert_eq!(app.app_dialog, Some(AppDialog::Settings));
        let _ = app.update(Message::Timeline(TimelineEvent::Select(3, 4)));
        assert_eq!(app.timeline.selection(), Some((1, 2)));
        let _ = app.update(Message::CloseHelp);
        assert!(app.app_dialog.is_none());
        assert_eq!(app.timeline.selection(), Some((1, 2)));
    }

    #[test]
    fn menu_outside_click_dismisses_without_editing_the_workspace() {
        let mut app = CleanSlateApp::default();
        app.timeline.set_selection(Some((1, 2)));
        let _ = app.update(Message::Menu);
        let messages = {
            let mut content = app.view();
            let mut tree = iced::advanced::widget::Tree::new(content.as_widget());
            let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                iced::Font::DEFAULT,
                iced::Pixels(16.0),
            ));
            let viewport = iced::Rectangle::with_size(iced::Size::new(1024.0, 768.0));
            let layout = content.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &iced::advanced::layout::Limits::new(iced::Size::ZERO, viewport.size()),
            );
            let mut messages = Vec::new();
            content.as_widget_mut().update(
                &mut tree,
                &iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                iced::advanced::Layout::new(&layout),
                iced::mouse::Cursor::Available(iced::Point::new(4.0, 4.0)),
                &renderer,
                &mut iced::advanced::clipboard::Null,
                &mut iced::advanced::Shell::new(&mut messages),
                &viewport,
            );
            messages
        };
        assert_eq!(
            messages.len(),
            1,
            "outside click must only dismiss the menu"
        );
        assert!(matches!(messages[0], Message::CloseHelp));
        for message in messages {
            let _ = app.update(message);
        }
        assert!(!app.menu_open);
        assert_eq!(app.timeline.selection(), Some((1, 2)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppDialog {
    Settings,
    About,
}

#[derive(Debug, Clone, Copy)]
enum AppLink {
    Help,
    GitHub,
    ReportBug,
}

impl AppLink {
    fn url(self) -> &'static str {
        match self {
            Self::Help => {
                "https://github.com/Today20092/podcast-bleed-reducer/blob/main/docs/editor-workflow.md"
            }
            Self::GitHub => "https://github.com/Today20092/podcast-bleed-reducer",
            Self::ReportBug => {
                "https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose"
            }
        }
    }
}

fn menu_action(index: usize) -> Message {
    crate::button_action::message(legacy_menu_action(index))
}

fn version_info(version: &str, build: Option<&str>, os: &str, arch: &str) -> String {
    let mut info = format!("CleanSlate {version}");
    if let Some(build) = build {
        info.push_str(&format!("\nBuild: {build}"));
    }
    info.push_str(&format!("\nPlatform: {os} ({arch})"));
    info
}

fn version_info_with_graphics(
    mut info: String,
    backend: &str,
    adapter: &str,
    scale: Option<f32>,
) -> String {
    fn hardware_field(value: &str) -> String {
        let value: String = value
            .chars()
            .filter(|c| !c.is_control())
            .take(160)
            .collect();
        let value = value.trim();
        if value.is_empty()
            || value.contains(":\\")
            || value.contains(":/")
            || value.starts_with('/')
            || value.starts_with("\\\\")
        {
            return "unavailable".into();
        }
        value.to_owned()
    }
    let scale = scale
        .filter(|scale| scale.is_finite() && *scale > 0.0)
        .map_or_else(|| "unavailable".into(), |scale| format!("{scale:.2}x"));
    info.push_str(&format!(
        "\nGraphics backend: {}\nGraphics adapter: {}\nWindow scale: {scale}\nDisplay refresh: unavailable (current GUI API)",
        hardware_field(backend),
        hardware_field(adapter),
    ));
    info
}

fn open_app_link(link: AppLink) -> Result<(), String> {
    use std::process::{Command, Stdio};
    #[cfg(target_os = "windows")]
    let mut command = {
        use std::os::windows::process::CommandExt;
        let mut command = Command::new("powershell.exe");
        // Only the fixed HTTPS allowlist enters this script; no project text.
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "try {{ Start-Process -FilePath '{}' -ErrorAction Stop }} catch {{ exit 1 }}",
                link.url()
            ),
        ]);
        command.creation_flags(0x08000000);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("/usr/bin/open");
        command.arg(link.url());
        command
    };
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(link.url());
        command
    };
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("The default browser launcher failed. Check your browser association.".into())
    }
}

struct ReferenceAudition {
    mode: AuditionMode,
    solo: u64,
    range: TimeRange,
    source_generation: u64,
    paused: bool,
}

pub struct CleanSlateApp {
    shortcuts: Shortcuts,
    shortcut_capture: Option<(Command, Option<Binding>)>,
    shortcut_hint_generation: u64,
    unlock_playhead_when_zooming: bool,
    appearance: Appearance,
    preferences_path: Option<PathBuf>,
    appearance_error: Option<String>,
    settings_selected: usize,
    menu_open: bool,
    menu_selected: usize,
    app_dialog: Option<AppDialog>,
    about_copy_selected: bool,
    copy_request: Option<Arc<()>>,
    version_details: Option<String>,
    app_feedback: Option<String>,
    help_open: bool,
    shortcuts_open: bool,
    help_close_selected: bool,
    sound_draft: Option<(SoundParameter, String)>,
    sound_error: Option<String>,
    sound_typing: bool,
    sound_menu: Option<SoundParameter>,
    sound_modifiers: keyboard::Modifiers,
    reference_audition: Option<ReferenceAudition>,
    project: Option<Project>,
    pending_imports: Vec<PathBuf>,
    source_inspection: Option<SourceInspection>,
    import_choices: Vec<ImportChoice>,
    import_dialog: bool,
    track_name_draft: Option<(usize, String)>,
    import_due: Option<Instant>,
    source_loading: Option<SourceLoading>,
    import_jobs: Vec<crate::import::Job>,
    next_import_id: u64,
    import_metadata: Vec<(crate::AudioSource, crate::SourceMetadata)>,
    import_metadata_failed: Vec<PathBuf>,
    messages: Vec<String>,
    processing: Option<Processing>,
    exporting: Option<ExportProcessing>,
    export_dialog: Option<ExportDialog>,
    pending_export: Option<(PathBuf, crate::ExportSettings)>,
    render_generation: Arc<crate::RenderGeneration>,
    review: Option<ReviewWorkspace>,
    audition: Option<LiveAudition>,
    idle_audition: Option<LiveAudition>,
    audition_output: Option<std::rc::Rc<crate::audition::AuditionOutput>>,
    audition_requested_at: Option<Instant>,
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
    setup_details: bool,
    analysis_details: Option<[(&'static str, f64); 6]>,
    analysis_details_open: bool,
    reference_stale: bool,
    reference_analyze_requested: bool,
    waveforms: Vec<WaveformCache>,
    timeline: TimelineViewport,
    playback_scroll: PlaybackScroll,
    meters_at_playhead: bool,
    waveform_display: waveform::WaveformDisplay,
    solo_tracks: u64,
    drawing_revision: u64,
    window_minimized: bool,
    window_visibility_generation: u64,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SoundParameter {
    Attenuation,
    Open,
    Close,
    Gap,
}

impl SoundParameter {
    fn control_id(self) -> &'static str {
        match self {
            Self::Attenuation => "sound-attenuation",
            Self::Open => "sound-open",
            Self::Close => "sound-close",
            Self::Gap => "sound-gap",
        }
    }

    fn value(self, p: AutomixParameters) -> f32 {
        match self {
            Self::Attenuation => p.attenuation_db,
            Self::Open => p.open_ms,
            Self::Close => p.close_ms,
            Self::Gap => p.minimum_gap_ms,
        }
    }
    fn set(self, p: &mut AutomixParameters, value: f32) {
        match self {
            Self::Attenuation => {
                p.attenuation_db = value;
                p.mute = false;
            }
            Self::Open => p.open_ms = value,
            Self::Close => p.close_ms = value,
            Self::Gap => p.minimum_gap_ms = value,
        }
    }
    fn parse(self, value: &str) -> Result<f32, &'static str> {
        let bounds = if self == Self::Attenuation {
            -120.0..=0.0
        } else {
            0.0..=60000.0
        };
        value
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && bounds.contains(v))
            .ok_or(if self == Self::Attenuation {
                "Enter a finite value from −120 to 0 dB."
            } else {
                "Enter a finite value from 0 to 60000 ms."
            })
    }
}

impl Default for CleanSlateApp {
    fn default() -> Self {
        Self {
            appearance: Appearance::Dark,
            shortcuts: Shortcuts::default(),
            shortcut_capture: None,
            shortcut_hint_generation: 0,
            unlock_playhead_when_zooming: false,
            preferences_path: None,
            appearance_error: None,
            settings_selected: 0,
            reference_audition: None,
            project: None,
            pending_imports: vec![],
            source_inspection: None,
            import_choices: vec![],
            import_dialog: false,
            help_open: false,
            menu_open: false,
            menu_selected: 0,
            app_dialog: None,
            about_copy_selected: true,
            copy_request: None,
            version_details: None,
            app_feedback: None,
            shortcuts_open: false,
            help_close_selected: false,
            track_name_draft: None,
            sound_draft: None,
            sound_error: None,
            sound_typing: false,
            sound_menu: None,
            sound_modifiers: keyboard::Modifiers::default(),
            import_due: None,
            source_loading: None,
            import_jobs: vec![],
            next_import_id: 0,
            import_metadata: vec![],
            import_metadata_failed: vec![],
            messages: vec![],
            processing: None,
            exporting: None,
            render_generation: Arc::new(crate::RenderGeneration::default()),
            export_dialog: None,
            pending_export: None,
            review: None,
            audition: None,
            idle_audition: None,
            audition_output: None,
            audition_requested_at: None,
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
            setup_details: false,
            analysis_details: None,
            analysis_details_open: false,
            reference_stale: false,
            reference_analyze_requested: false,
            waveforms: vec![],
            timeline: TimelineViewport::default(),
            playback_scroll: PlaybackScroll::Page,
            meters_at_playhead: true,
            waveform_display: waveform::WaveformDisplay::Both,
            solo_tracks: 0,
            drawing_revision: 0,
            window_minimized: false,
            window_visibility_generation: 0,
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

type InspectedSource = (crate::AudioSource, Option<crate::SourceMetadata>);
type ImportDisplayRow = (Option<usize>, Option<(usize, usize)>);

struct SourceInspection {
    paths: Vec<PathBuf>,
    result: Receiver<Result<Vec<InspectedSource>, String>>,
    cancelled: Arc<AtomicBool>,
}

impl Drop for SourceInspection {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
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
    Prepared,
    Finished(Result<Box<LoadedSession>, String>),
}

struct SourceLoading {
    paths: Vec<PathBuf>,
    waveforms: Vec<WaveformCache>,
    timeline: TimelineViewport,
    stage: &'static str,
    events: Receiver<SourceEvent>,
    publish: Option<mpsc::Sender<()>>,
    committing: bool,
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
    if !opening && let Some(project) = &mut project {
        project.defer_save();
    }
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
    let mut local_project_notice = None;
    if let SourceRequest::Open(path) = &request {
        let mut opened = Project::open(path).map_err(|error| error.to_string())?;
        if opened
            .localize_for_editing()
            .map_err(|error| error.to_string())?
        {
            local_project_notice = Some(format!(
                "Working project saved locally at {}. Original media and project were left untouched.",
                opened.path().display()
            ));
        }
        project = Some(opened);
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
    let project_path = project
        .as_ref()
        .map_or_else(crate::storage::new_project, |project| {
            Ok(project.path().to_owned())
        })
        .map_err(|error| error.to_string())?;
    let _ = sender.send(SourceEvent::Stage("Loading waveforms"));
    let waveform_result = waveform::load_source_waveforms(&inputs, |index, waveform| {
        let _ = sender.send(SourceEvent::Waveform(index, waveform.clone()));
    });
    let mut warnings: Vec<_> = local_project_notice.into_iter().collect();
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
                let mut created = Project::prepare_from_sources(project_path, additions)
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

struct ExportDialog {
    settings: crate::ExportSettings,
    destination: Option<PathBuf>,
}
impl Drop for ExportProcessing {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

struct PreviewProcessing {
    started_at: Instant,
    feedback: Arc<Mutex<String>>,
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
    InvokeAction(Command),
    CaptureShortcut(Command),
    ApplyShortcut,
    RestoreShortcut(Command),
    RestoreAllShortcuts,
    UnbindShortcut(Command),
    ShortcutHintInput,
    UnlockOnZoom(bool),
    Appearance(Appearance),
    Menu,
    Settings,
    About,
    OpenLink(AppLink),
    LinkOpened(AppLink, Result<(), String>),
    CopyVersion,
    CopyProjectPath,
    VersionInfoReady(Arc<()>, String),
    VersionCopied(Arc<()>, bool),
    Help,
    KeyboardShortcuts,
    CloseHelp,
    ImportChannel(usize, bool),
    ImportChannelName(usize, String),
    ConfirmChannels,
    CancelChannels,
    RemoveImport(u64),
    RetryImport(u64),
    CancelPendingImports,
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
    SuppliedAlignment(bool),
    ImportStems,
    NewProject,
    OpenProject,
    FileDropped(PathBuf),
    Relink(usize),
    Validate,
    #[cfg(test)]
    Process,
    Cancel,
    Tick,
    PlaybackFrame,
    CheckWindowVisibility,
    WindowVisibilityChecked(u64, Option<bool>),
    LockPlayhead,
    CenterPlayhead,
    ToggleMeterPlacement,
    WaveformDisplay(waveform::WaveformDisplay),
    SoloTrack(usize),
    SetupDetails(bool),
    AnalysisDetails(bool),
    ToggleCatchUp,
    #[cfg(test)]
    SpeechSensitivity(f32),
    Attenuation(f32),
    EditSound(SoundParameter),
    SoundText(String),
    CommitSound,
    CancelSound,
    CheckSoundFocus,
    SoundFocused(bool),
    SoundTab(bool),
    SoundModifiers(keyboard::Modifiers),
    SoundMenu(SoundParameter),
    ResetSound(SoundParameter),
    Mute,
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
    ExportLayout(crate::ExportLayout),
    ExportEncoding(crate::ExportEncoding),
    ExportDestination,
    CancelExportDialog,
    SubmitExport,
    Timeline(TimelineEvent),
}

// Individually audited exceptions; new actions belong in shortcuts::commands.
macro_rules! legacy_buttons {
    ($($variant:ident $(($($arg:ident: $ty:ty),*))? => $reason:literal),* $(,)?) => {
        #[derive(Clone)]
        enum LegacyButton { $($variant $(($($ty),*))?),* }
        impl crate::button_action::ButtonAction<Message> for LegacyButton {
            fn into_message(self) -> Message {
                match self { $(Self::$variant $(($($arg),*))? => {
                    let _reason = $reason;
                    Message::$variant $(($($arg),*))?
                }),* }
            }
        }
    };
}
legacy_buttons! {
    About => "About: Transient dialog, capture or menu navigation.",
    AnalysisDetails(a: bool) => "AnalysisDetails: Text/value editor or disclosure with local input ownership.",
    AnalyzeReferences => "AnalyzeReferences: Legacy command pending migration; existing availability and keyboard binding retained.",
    Appearance(a: Appearance) => "Appearance: Text/value editor or disclosure with local input ownership.",
    ApplyShortcut => "ApplyShortcut: Transient dialog, capture or menu navigation.",
    Audition => "Audition: Legacy command pending migration; existing availability and keyboard binding retained.",
    CalibrationTiming(a: bool) => "CalibrationTiming: Text/value editor or disclosure with local input ownership.",
    Cancel => "Cancel: Legacy command pending migration; existing availability and keyboard binding retained.",
    CancelAnalysis => "CancelAnalysis: Legacy command pending migration; existing availability and keyboard binding retained.",
    CancelChannels => "CancelChannels: Transient dialog, capture or menu navigation.",
    CancelExportDialog => "CancelExportDialog: Transient dialog, capture or menu navigation.",
    CancelPendingImports => "CancelPendingImports: Legacy command pending migration; existing availability and keyboard binding retained.",
    CancelTrackName => "CancelTrackName: Text/value editor or disclosure with local input ownership.",
    CaptureShortcut(a: Command) => "CaptureShortcut: Transient dialog, capture or menu navigation.",
    CloseHelp => "CloseHelp: Transient dialog, capture or menu navigation.",
    ConfirmChannels => "ConfirmChannels: Transient dialog, capture or menu navigation.",
    ContextReference(a: Vec<SpeakerId>, b: bool) => "ContextReference: Explicit item arguments; current source, range and selection validation retained.",
    CopyProjectPath => "CopyProjectPath: Legacy command pending migration; existing availability and keyboard binding retained.",
    CopyVersion => "CopyVersion: Transient dialog, capture or menu navigation.",
    DeleteGain => "DeleteGain: Legacy command pending migration; existing availability and keyboard binding retained.",
    DeleteRegion(a: usize, b: u64) => "DeleteRegion: Explicit item arguments; current source, range and selection validation retained.",
    DismissReferenceMenu => "DismissReferenceMenu: Transient dialog, capture or menu navigation.",
    EditSound(a: SoundParameter) => "EditSound: Text/value editor or disclosure with local input ownership.",
    EditTrackName(a: usize) => "EditTrackName: Text/value editor or disclosure with local input ownership.",
    Export => "Export: Legacy command pending migration; existing availability and keyboard binding retained.",
    ExportDestination => "ExportDestination: Transient dialog, capture or menu navigation.",
    GainAction(a: usize, b: bool) => "GainAction: Explicit item arguments; current source, range and selection validation retained.",
    ImportStems => "ImportStems: Legacy command pending migration; existing availability and keyboard binding retained.",
    InsertGainPoint => "InsertGainPoint: Explicit item arguments; current source, range and selection validation retained.",
    KeyboardShortcuts => "KeyboardShortcuts: Transient dialog, capture or menu navigation.",
    Menu => "Menu: Transient dialog, capture or menu navigation.",
    Mute => "Mute: Text/value editor or disclosure with local input ownership.",
    NewProject => "NewProject: Legacy command pending migration; existing availability and keyboard binding retained.",
    OpenLink(a: AppLink) => "OpenLink: Transient dialog, capture or menu navigation.",
    OpenProject => "OpenProject: Legacy command pending migration; existing availability and keyboard binding retained.",
    Original => "Original: Legacy command pending migration; existing availability and keyboard binding retained.",
    PlayExample(a: u64) => "PlayExample: Explicit item arguments; current source, range and selection validation retained.",
    Previous => "Previous: Legacy command pending migration; existing availability and keyboard binding retained.",
    Processed => "Processed: Legacy command pending migration; existing availability and keyboard binding retained.",
    ProtectSelection => "ProtectSelection: Explicit item arguments; current source, range and selection validation retained.",
    RebindSource(a: usize) => "RebindSource: Explicit item arguments; current source, range and selection validation retained.",
    RedoGain => "RedoGain: Legacy command pending migration; existing availability and keyboard binding retained.",
    RedoReferences => "RedoReferences: Legacy command pending migration; existing availability and keyboard binding retained.",
    Relink(a: usize) => "Relink: Explicit item arguments; current source, range and selection validation retained.",
    RemoveImport(a: u64) => "RemoveImport: Explicit item arguments; current source, range and selection validation retained.",
    RemoveReference(a: u64) => "RemoveReference: Explicit item arguments; current source, range and selection validation retained.",
    Removed => "Removed: Legacy command pending migration; existing availability and keyboard binding retained.",
    ResetSound(a: SoundParameter) => "ResetSound: Text/value editor or disclosure with local input ownership.",
    RestoreAllShortcuts => "RestoreAllShortcuts: Transient dialog, capture or menu navigation.",
    RestoreShortcut(a: Command) => "RestoreShortcut: Transient dialog, capture or menu navigation.",
    RetryImport(a: u64) => "RetryImport: Explicit item arguments; current source, range and selection validation retained.",
    SaveProject => "SaveProject: Legacy command pending migration; existing availability and keyboard binding retained.",
    SaveTrackName => "SaveTrackName: Text/value editor or disclosure with local input ownership.",
    Settings => "Settings: Transient dialog, capture or menu navigation.",
    SetupDetails(a: bool) => "SetupDetails: Text/value editor or disclosure with local input ownership.",
    ShowExample(a: u64) => "ShowExample: Explicit item arguments; current source, range and selection validation retained.",
    SoloTrack(a: usize) => "SoloTrack: Explicit item arguments; current source, range and selection validation retained.",
    Stop => "Stop: Legacy command pending migration; existing availability and keyboard binding retained.",
    SubmitExport => "SubmitExport: Transient dialog, capture or menu navigation.",
    UnbindShortcut(a: Command) => "UnbindShortcut: Transient dialog, capture or menu navigation.",
    UndoGain => "UndoGain: Legacy command pending migration; existing availability and keyboard binding retained.",
    UndoReferences => "UndoReferences: Legacy command pending migration; existing availability and keyboard binding retained.",
    Validate => "Validate: Legacy command pending migration; existing availability and keyboard binding retained.",
    WaveformDisplay(a: waveform::WaveformDisplay) => "WaveformDisplay: Text/value editor or disclosure with local input ownership.",
}

struct RegisteredButton(Command);
impl crate::button_action::ButtonAction<Message> for RegisteredButton {
    fn into_message(self) -> Message {
        Message::InvokeAction(self.0)
    }
}

fn legacy_menu_action(index: usize) -> LegacyButton {
    match index {
        0 => LegacyButton::Settings,
        1 => LegacyButton::KeyboardShortcuts,
        2 => LegacyButton::OpenLink(AppLink::Help),
        3 => LegacyButton::OpenLink(AppLink::GitHub),
        4 => LegacyButton::OpenLink(AppLink::ReportBug),
        5 => LegacyButton::About,
        _ => LegacyButton::CloseHelp,
    }
}

pub fn run() -> iced::Result {
    crate::performance::start_from_environment();
    let result = iced::application(
        || {
            #[cfg(debug_assertions)]
            let project = std::env::var_os("CLEANSLATE_DEV_PROJECT").map(PathBuf::from);
            #[cfg(not(debug_assertions))]
            let project = None;
            startup(project)
        },
        |app: &mut CleanSlateApp, message| {
            let _update = crate::performance::measure(crate::performance::Metric::AppUpdate);
            let _poll = matches!(message, Message::Tick)
                .then(|| crate::performance::measure(crate::performance::Metric::WorkerPoll));
            app.update(message)
        },
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
    .default_font(iced::Font::with_name("Segoe UI"))
    .subscription(|app: &CleanSlateApp| {
        let observations = if crate::performance::is_active() {
            iced::event::listen_raw(|event, _, _| {
                crate::performance::observe_event(&event);
                None::<Message>
            })
        } else {
            Subscription::none()
        };
        Subscription::batch([app.subscription(), observations])
    })
    .window(window::Settings {
        icon: Some(
            window::icon::from_rgba(include_bytes!("../assets/cleanslate.rgba").to_vec(), 64, 64)
                .expect("embedded CleanSlate icon is 64 by 64 RGBA"),
        ),
        maximized: true,
        min_size: Some(iced::Size::new(960.0, 900.0)),
        ..window::Settings::default()
    })
    .run();
    crate::performance::finish();
    result
}

fn startup(project: Option<PathBuf>) -> CleanSlateApp {
    let mut app = CleanSlateApp::default();
    match appearance::preference_path() {
        Ok(path) => {
            (app.appearance, app.appearance_error) = appearance::load(&path);
            let (shortcuts, unlock, error) = appearance::load_shortcuts(&path);
            app.shortcuts = shortcuts;
            app.unlock_playhead_when_zooming = unlock;
            app.appearance_error = app.appearance_error.or(error);
            app.preferences_path = Some(path);
        }
        Err(error) => app.appearance_error = Some(error),
    }
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

fn application_keyboard_event(
    event: keyboard::Event,
    status: iced::event::Status,
) -> Option<Message> {
    match event {
        keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        } => keyboard_message(key, modifiers, status),
        _ => None,
    }
}

fn application_input_event(
    event: iced::Event,
    status: iced::event::Status,
    sound_owns_keys: bool,
) -> Option<Message> {
    match event {
        iced::Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Tab | keyboard::key::Named::Escape),
            ..
        }) if sound_owns_keys => None,
        iced::Event::Keyboard(event @ keyboard::Event::KeyPressed { .. }) => {
            application_keyboard_event(event, status)
        }
        iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            Some(Message::SoundModifiers(modifiers))
        }
        _ => None,
    }
}

fn compact_workspace(width: f32) -> bool {
    width < 1100.0
}

fn selected_button(theme: &Theme, status: button::Status) -> button::Style {
    let c = colors(theme);
    let mut style = secondary_button(theme, status);
    if status != button::Status::Disabled {
        style.border.color = c.focus;
        style.border.width = 2.0;
        if status == button::Status::Active {
            style.background = Some(c.selection.into());
        }
    }
    style
}

fn quiet_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = secondary_button(theme, status);
    if status == button::Status::Active {
        style.background = None;
        style.border.width = 0.0;
    }
    style
}

fn flat_button(theme: &Theme, status: button::Status, selected: bool) -> button::Style {
    let c = colors(theme);
    button::Style {
        background: match status {
            button::Status::Hovered => Some(c.hover.into()),
            button::Status::Pressed => Some(c.pressed.into()),
            button::Status::Active if selected => Some(c.selection.into()),
            _ => None,
        },
        text_color: if status == button::Status::Disabled {
            c.disabled
        } else {
            c.text
        },
        border: iced::Border {
            radius: 5.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        ..button::Style::default()
    }
}

fn toolbar_label(icon: ToolbarIcon, label: &str) -> Element<'_, Message> {
    row![icon.view(None), text(label).size(13)]
        .spacing(6)
        .align_y(iced::Center)
        .into()
}

impl CleanSlateApp {
    fn commit_sound(&mut self) {
        let Some((parameter, draft)) = &self.sound_draft else {
            return;
        };
        match parameter.parse(draft) {
            Ok(value) => {
                parameter.set(&mut self.preview_parameters, value);
                self.sound_draft = None;
                self.sound_error = None;
                self.sound_typing = false;
            }
            Err(error) => self.sound_error = Some(error.into()),
        }
    }

    fn sound_value(&self, parameter: SoundParameter, label: &'static str) -> Element<'_, Message> {
        let unit = if parameter == SoundParameter::Attenuation {
            "dB"
        } else {
            "ms"
        };
        let value = if parameter == SoundParameter::Attenuation && self.preview_parameters.mute {
            "Mute".into()
        } else {
            format!("{} {unit}", parameter.value(self.preview_parameters))
        };
        let control: Element<'_, Message> = if let Some((active, draft)) = &self.sound_draft
            && *active == parameter
        {
            row![
                text_input("Value", draft)
                    .font(iced::Font::MONOSPACE)
                    .id("sound-value")
                    .on_input(Message::SoundText)
                    .on_submit(Message::CommitSound)
                    .width(78),
                text(unit).size(14)
            ]
            .spacing(6)
            .into()
        } else {
            mouse_area(
                button(text(value).size(15).font(iced::Font::MONOSPACE))
                    .id(parameter.control_id())
                    .padding([6, 10])
                    .width(108)
                    .style(sound_button)
                    .on_press(LegacyButton::EditSound(parameter)),
            )
            .on_right_press(Message::SoundMenu(parameter))
            .into()
        };
        let mut content = if label.is_empty() {
            column![control]
        } else {
            column![
                row![text(label).size(14), Space::new().width(Fill), control]
                    .spacing(6)
                    .align_y(iced::Center)
            ]
        }
        .spacing(4);
        if self
            .sound_draft
            .as_ref()
            .is_some_and(|(active, _)| *active == parameter)
            && let Some(error) = &self.sound_error
        {
            content = content.push(text(error).size(12).color(self.appearance.colors().error));
        }
        if self.sound_menu == Some(parameter) {
            content = content.push(
                button("Reset to default")
                    .style(secondary_button)
                    .on_press(LegacyButton::ResetSound(parameter)),
            );
        }
        content.into()
    }

    fn sound_dial(&self, parameter: SoundParameter, size: f32) -> Element<'_, Message> {
        use crate::parameter_dial::{DialKind, ParameterDial};
        let (kind, on_change): (DialKind, fn(f32) -> Message) = match parameter {
            SoundParameter::Attenuation => (DialKind::Attenuation, Message::Attenuation),
            SoundParameter::Open => (DialKind::Open, Message::OpenTime),
            SoundParameter::Close => (DialKind::Close, Message::CloseTime),
            SoundParameter::Gap => (DialKind::Gap, Message::MinimumGap),
        };
        canvas(ParameterDial {
            kind,
            value: parameter.value(self.preview_parameters),
            muted: parameter == SoundParameter::Attenuation && self.preview_parameters.mute,
            enabled: self.workspace_input_active(),
            keyboard_enabled: self.sound_draft.is_none() && self.track_name_draft.is_none(),
            on_change,
            on_release: Message::RefreshPreview,
            on_reset: Message::ResetSound(parameter),
            on_edit: Message::EditSound(parameter),
        })
        .width(size)
        .height(size)
        .into()
    }

    fn timing_control(
        &self,
        parameter: SoundParameter,
        label: &'static str,
    ) -> Element<'_, Message> {
        row![
            self.sound_dial(parameter, 56.0),
            column![text(label).size(14), self.sound_value(parameter, "")]
                .spacing(4)
                .width(Fill),
        ]
        .spacing(12)
        .align_y(iced::Center)
        .into()
    }
    fn install_waveforms(&mut self, waveforms: Vec<WaveformCache>) {
        self.close_reference_menu();
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
        self.idle_audition = None;
        self.audition_output = None;
        self.audition_requested_at = None;
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
                vec!["Wait for WAV preparation to finish before importing tracks.".into()];
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
        self.import_dialog = true;
        self.pending_imports
            .sort_by_key(|path| path.to_string_lossy().to_lowercase());
        // A desktop multi-file drop arrives as separate events. Collect the burst
        // before starting one source pass, without delaying placeholder rows.
        self.import_due = Some(Instant::now() + Duration::from_millis(150));
        self.messages.clear();
    }

    fn reserved_track_count(&self) -> usize {
        self.project.as_ref().map_or(0, |p| p.sources.len())
            + self
                .import_jobs
                .iter()
                .filter(|j| !j.accepted)
                .map(|j| j.sources.len())
                .sum::<usize>()
    }

    fn reserve_import(&mut self, additions: Vec<crate::AudioSource>) {
        if self.reserved_track_count() + additions.len() > 8 || additions.is_empty() {
            self.messages = vec!["Import blocked: accepted and pending tracks together must fit the eight-track limit.".into()];
            return;
        }
        if additions.iter().any(|s| {
            self.import_jobs.iter().any(|j| {
                j.sources
                    .iter()
                    .any(|old| old.path == s.path && old.channel == s.channel)
            })
        }) {
            self.messages = vec!["These channels are already accepted or reserved. Retry or remove their existing rows.".into()];
            return;
        }
        let mut recordings: Vec<Vec<crate::AudioSource>> = vec![];
        for source in additions {
            if let Some(recording) = recordings.iter_mut().find(|r| r[0].path == source.path) {
                recording.push(source);
            } else {
                recordings.push(vec![source]);
            }
        }
        for recording in recordings {
            self.next_import_id += 1;
            let metadata = recording.iter().map(|s| {
                if self.import_metadata_failed.contains(&s.path) { return Err(crate::RenderBlocker("Source metadata could not be read. Check the file, then Retry or Remove.".into())); }
                self.import_metadata.iter().rev().find(|(source, _)| source.path == s.path && source.channel == s.channel)
                .map(|(_, metadata)| *metadata).map_or_else(|| s.metadata(None), Ok)
            }).collect::<Result<Vec<_>, _>>();
            match metadata.map(|metadata| {
                crate::import::Job::from_metadata(self.next_import_id, recording.clone(), metadata)
            }) {
                Ok(job) => {
                    if let Some(waveform) = job.waveforms.first() {
                        if self.project.is_none() && self.import_jobs.is_empty() {
                            self.timeline.set_total_samples(waveform.duration());
                        } else {
                            self.timeline.extend_total_samples(waveform.duration());
                        }
                    }
                    self.import_jobs.push(job);
                }
                Err(error) => self.import_jobs.push(crate::import::Job::failed(
                    self.next_import_id,
                    recording,
                    error.to_string(),
                )),
            }
        }
        self.import_dialog = false;
        self.start_next_import();
    }

    fn start_next_import(&mut self) {
        if self.import_jobs.iter().any(|job| job.events.is_some()) {
            return;
        }
        if let Some(job) = self
            .import_jobs
            .iter_mut()
            .find(|job| !job.accepted && job.error.is_none())
        {
            job.start();
        }
    }

    fn cancel_pending_imports(&mut self) {
        self.import_jobs.retain(|job| job.accepted);
        for job in &mut self.import_jobs {
            job.cancelled.store(true, Ordering::Relaxed);
            job.events = None;
            job.status = "Ready".into();
        }
        self.pending_imports.clear();
        self.source_inspection = None;
        self.import_due = None;
        self.import_choices.clear();
        self.import_dialog = false;
    }

    fn poll_import_jobs(&mut self) {
        // Bounded event work keeps navigation responsive even when a worker is faster than drawing.
        for _ in 0..4 {
            let Some(index) = self.import_jobs.iter().position(|job| job.events.is_some()) else {
                break;
            };
            let event = self.import_jobs[index].events.as_ref().unwrap().try_recv();
            if matches!(
                &event,
                Ok(crate::import::Event::Metadata(_)
                    | crate::import::Event::Preview(_)
                    | crate::import::Event::Ready(_, _))
            ) {
                self.drawing_revision = self.drawing_revision.wrapping_add(1);
            }
            match event {
                Ok(crate::import::Event::Metadata(waveforms)) => {
                    if let Some(waveform) = waveforms.first() {
                        self.timeline.extend_total_samples(waveform.duration());
                    }
                    self.import_jobs[index].waveforms = waveforms;
                }
                Ok(crate::import::Event::Preview(waveforms)) => {
                    self.import_jobs[index].waveforms = waveforms;
                    self.import_jobs[index].status = "Coarse preview · not yet validated".into();
                }
                Ok(crate::import::Event::Progress(stage, done, total)) => {
                    self.import_jobs[index].status = if total == 0 {
                        stage.into()
                    } else {
                        format!("{stage} · {:.0}%", 100.0 * done as f64 / total as f64)
                    };
                }
                Ok(crate::import::Event::Ready(identities, waveforms)) => {
                    let path = self
                        .project
                        .as_ref()
                        .map_or_else(crate::storage::new_project, |p| Ok(p.path().to_owned()));
                    self.import_jobs[index].status = "Finishing · atomic project save".into();
                    let first = self.project.as_ref().map_or(0, |p| p.sources.len());
                    match path.and_then(|path| {
                        Project::accept_recording(
                            self.project.as_ref(),
                            path,
                            identities,
                            self.preview_parameters,
                        )
                    }) {
                        Ok(project) => {
                            self.import_jobs[index].track_ids = project.sources[first..]
                                .iter()
                                .map(|s| s.track_id)
                                .collect();
                            self.project = Some(project);
                            self.waveforms.extend(waveforms.clone());
                            self.import_jobs[index].waveforms = waveforms;
                            self.import_jobs[index].accepted = true;
                            self.import_jobs[index].status = "Ready · saving waveform cache".into();
                            self.source_generation += 1;
                            if let Some(snapshot) = &mut self.reference_audition {
                                snapshot.source_generation = self.source_generation;
                            }
                            self.preview_generation += 1;
                            if let Some(processing) = self.preview_processing.take() {
                                processing.cancelled.store(true, Ordering::Relaxed);
                            }
                            self.preview_pending = None;
                            self.preview = None;
                            self.supervised_result = None;
                            self.effective_result = None;
                            self.reference_stale = true;
                            self.gain_constraints =
                                self.project.as_ref().unwrap().gain_constraints();
                            self.audition_preparation.invalidate();
                            self.audition_ready = false;
                            if self.audition.is_none() {
                                self.audition_preparation.request_sources(
                                    self.project.as_ref().unwrap().audio_sources(),
                                );
                            } else {
                                self.messages.push("New recording is ready. It joins the mix the next time playback starts.".into());
                            }
                            self.render_generation.invalidate();
                        }
                        Err(error) => {
                            self.import_jobs[index].error = Some(error.to_string());
                            self.messages.push(format!("Import blocked: {error}"));
                            self.import_jobs[index].status = "Failed · Retry or Remove".into();
                            self.import_jobs[index]
                                .cancelled
                                .store(true, Ordering::Relaxed);
                            self.import_jobs[index].events = None;
                        }
                    }
                }
                Ok(crate::import::Event::Finished(result)) => {
                    let job = &mut self.import_jobs[index];
                    job.events = None;
                    if job.accepted {
                        job.status = "Ready".into();
                    } else {
                        job.error =
                            Some(result.err().unwrap_or_else(|| {
                                "Import finished without validated audio".into()
                            }));
                        job.status = "Failed · Retry or Remove".into();
                        self.messages
                            .push(format!("Import blocked: {}", job.error.as_deref().unwrap()));
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    let job = &mut self.import_jobs[index];
                    job.events = None;
                    if !job.accepted {
                        job.error = Some("Import worker stopped unexpectedly; Retry.".into());
                    }
                }
            }
        }
        self.start_next_import();
    }

    fn sources_busy(&self) -> bool {
        self.source_loading.is_some()
            || self
                .import_jobs
                .iter()
                .any(|job| job.events.is_some() || (!job.accepted && job.error.is_none()))
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
        for job in self.import_jobs.iter().filter(|job| !job.accepted) {
            paths.extend(job.sources.iter().map(|source| source.path.clone()));
        }
        paths.extend(self.pending_imports.iter().cloned());
        paths
    }

    fn begin_source_loading(&mut self, request: SourceRequest) {
        if let SourceRequest::AddChannels(additions) = request {
            self.reserve_import(additions);
            return;
        }
        self.cancel_pending_imports();
        if self.source_loading.is_some() || self.processing.is_some() {
            return;
        }
        self.close_reference_menu();
        self.import_dialog = true;
        self.stop_audition();
        self.track_name_draft = None;
        self.sound_draft = None;
        self.sound_error = None;
        self.sound_typing = false;
        self.sound_menu = None;
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
        let (publish, accepted) = mpsc::channel();
        std::thread::spawn(move || {
            let result =
                load_sources(request, project, parameters, &sender).and_then(|mut loaded| {
                    // Preparation cannot change the saved project. Only the window's
                    // acceptance starts the short, noncancelable atomic publication.
                    sender
                        .send(SourceEvent::Prepared)
                        .map_err(|_| "Import cancelled".to_string())?;
                    accepted
                        .recv()
                        .map_err(|_| "Import cancelled".to_string())?;
                    loaded
                        .project
                        .publish_prepared()
                        .map_err(|error| error.to_string())?;
                    Ok(Box::new(loaded))
                });
            let _ = sender.send(SourceEvent::Finished(result));
        });
        self.source_loading = Some(SourceLoading {
            paths,
            waveforms: vec![],
            timeline: TimelineViewport::default(),
            stage: "Opening audio",
            events,
            publish: Some(publish),
            committing: false,
        });
    }

    fn poll_sources(&mut self) {
        self.poll_import_jobs();
        if let Some(inspection) = &self.source_inspection {
            let result = match inspection.result.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Source inspection stopped unexpectedly.".into()))
                }
            };
            if let Some(result) = result {
                self.drawing_revision = self.drawing_revision.wrapping_add(1);
                self.source_inspection = None;
                let result = result.map(|inputs| {
                    inputs
                        .into_iter()
                        .map(|(source, metadata)| {
                            if let Some(metadata) = metadata {
                                self.import_metadata_failed.retain(|p| *p != source.path);
                                self.import_metadata.push((source.clone(), metadata));
                            } else {
                                self.import_metadata_failed.push(source.path.clone());
                            }
                            source
                        })
                        .collect::<Vec<_>>()
                });
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
                Ok(SourceEvent::Prepared) => {
                    loading.committing = true;
                    loading.stage = "Finishing import · saving project";
                    if let Some(publish) = loading.publish.take() {
                        let _ = publish.send(());
                    }
                }
                Ok(SourceEvent::Waveform(index, waveform)) => {
                    if index == loading.waveforms.len() {
                        if index == 0 {
                            loading.timeline.set_total_samples(waveform.duration());
                        }
                        loading.waveforms.push(waveform);
                        self.drawing_revision = self.drawing_revision.wrapping_add(1);
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
                    self.drawing_revision = self.drawing_revision.wrapping_add(1);
                    let loading = self.source_loading.take().unwrap();
                    match result {
                        Ok(loaded) => {
                            self.import_dialog = !self.pending_imports.is_empty();
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
                let cancelled = Arc::new(AtomicBool::new(false));
                let worker_cancelled = cancelled.clone();
                let mut paths = self
                    .project
                    .as_ref()
                    .map_or_else(Vec::new, Project::source_paths);
                paths.extend(additions.iter().cloned());
                std::thread::spawn(move || {
                    let inputs = additions
                        .into_iter()
                        .map(|path| {
                            let Ok(metadata) = crate::SourceMetadata::read_cancellable(
                                &path,
                                Some(&worker_cancelled),
                            ) else {
                                return Ok(vec![(crate::AudioSource::mono(path), None)]);
                            };
                            if metadata.channels > 1 {
                                crate::wav_channel_sources(&path)
                                    .map(|sources| {
                                        sources
                                            .into_iter()
                                            .map(|source| {
                                                let mut selected = metadata;
                                                selected.channels = 1;
                                                (source, Some(selected))
                                            })
                                            .collect()
                                    })
                                    .map_err(|e| e.to_string())
                            } else {
                                Ok(vec![(crate::AudioSource::mono(path), Some(metadata))])
                            }
                        })
                        .collect::<Result<Vec<_>, String>>()
                        .map(|inputs| inputs.into_iter().flatten().collect());
                    let _ = sender.send(inputs);
                });
                self.source_inspection = Some(SourceInspection {
                    paths,
                    result,
                    cancelled,
                });
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
        self.appearance.theme()
    }

    fn export_current(&mut self) {
        if !self.can_prepare_export() {
            return;
        }
        self.export_dialog = Some(ExportDialog {
            settings: crate::ExportSettings {
                layout: crate::ExportLayout::SeparateMono,
                encoding: self.project.as_ref().unwrap().export_encoding(),
            },
            destination: None,
        });
    }

    fn can_prepare_export(&self) -> bool {
        self.source_loading.is_none()
            && self.processing.is_none()
            && self.exporting.is_none()
            && !self.reference_stale
            && self.gain_drag.is_none()
            && self.preview_pending.is_none()
            && self.preview_processing.is_none()
            && self
                .project
                .as_ref()
                .zip(self.supervised_result.as_ref())
                .zip(self.effective_result.as_ref())
                .is_some_and(|((project, analysis), effective)| {
                    project.parameters() == self.preview_parameters
                        && crate::SupervisedRenderSnapshot::capture(project, analysis, effective)
                            .is_ok()
                })
    }

    fn submit_export(&mut self) {
        if !self.can_prepare_export() {
            return;
        }
        let Some(dialog) = &self.export_dialog else {
            return;
        };
        let Some(destination) = dialog.destination.clone() else {
            return;
        };
        let settings = dialog.settings;
        let project = self.project.as_mut().unwrap();
        if let Err(error) = settings.validate_layout(project) {
            self.messages = vec![error.to_string()];
            return;
        }
        if let Err(error) = project.set_export_encoding(settings.encoding) {
            self.messages = vec![error.to_string()];
            return;
        }
        self.export_dialog = None;
        self.pending_export = Some((destination, settings));
        self.start_processing();
        if self.processing.is_none() {
            self.pending_export = None;
        }
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
        if self.reference_stale
            || self.gain_drag.is_some()
            || self.preview_processing.is_some()
            || self.preview_pending.is_some()
        {
            self.messages =
                vec!["Finish editing and run Analyze before preparing WAV files.".into()];
            return;
        }
        let snapshot = match crate::SupervisedRenderSnapshot::capture(project, analysis, effective)
        {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.messages = vec![error.to_string()];
                return;
            }
        };
        let project = project.clone();
        let analysis = analysis.clone();
        let effective = effective.clone();
        let generation = self.render_generation.current();
        let gate = self.render_generation.clone();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let working = match crate::storage::working_directory(project.path()) {
            Ok(path) => path,
            Err(error) => {
                self.messages = vec![error.to_string()];
                return;
            }
        };
        let destination = working
            .join("renders")
            .join(format!("{}-{generation}-{nonce}", std::process::id()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU8::new(0));
        let (sender, result) = mpsc::channel();
        let worker_cancel = cancelled.clone();
        let worker_progress = progress.clone();
        let worker_snapshot = snapshot.clone();
        std::thread::spawn(move || {
            let completed = (|| {
                let private = crate::prepare_supervised_float_render(
                    &project,
                    &analysis,
                    &effective,
                    &worker_snapshot,
                    &destination,
                    &worker_progress,
                    Some(&worker_cancel),
                )?;
                private.publish(&project, &gate, generation, Some(&worker_cancel))
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
        self.messages = vec!["Preparing processed WAV files at the original sample rates.".into()];
    }

    #[cfg(test)]
    fn export_to(&mut self, destination: PathBuf) {
        self.export_to_settings(destination, crate::ExportSettings::default());
    }

    fn export_to_settings(&mut self, destination: PathBuf, settings: crate::ExportSettings) {
        if !self.export_ready() {
            self.messages =
                vec!["Prepare WAV files with the current settings before saving.".into()];
            return;
        }
        let Some(project) = &self.project else { return };
        let source_order = self.export_source_order();
        let project = project.clone();
        let gate = self.render_generation.clone();
        let generation = gate.current();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancelled.clone();
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let completed = (|| {
                let private = crate::prepare_export_in_order(
                    &project,
                    &destination,
                    settings,
                    &source_order,
                    Some(&worker_cancel),
                )?;
                private.publish(&project, &gate, generation, Some(&worker_cancel))?;
                Ok(destination)
            })()
            .map_err(|error: crate::RenderBlocker| error.to_string());
            let _ = sender.send(completed);
        });
        self.exporting = Some(ExportProcessing { cancelled, result });
        self.messages = vec!["Checking and saving WAV files in the background.".into()];
    }

    fn poll_exporting(&mut self) {
        let Some(exporting) = &self.exporting else {
            return;
        };
        let completed = match exporting.result.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("Saving WAV files stopped unexpectedly".into())
            }
        };
        self.exporting = None;
        self.messages = vec![match completed {
            Ok(destination) => format!("WAV files saved to {}", destination.display()),
            Err(error) => format!("WAV files were not saved: {error}"),
        }];
    }
    fn export_ready(&self) -> bool {
        self.source_loading.is_none()
            && !self.reference_stale
            && self.gain_drag.is_none()
            && self.preview_pending.is_none()
            && self.preview_processing.is_none()
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
    fn can_start_new_project(&self) -> bool {
        self.source_loading.is_none() && self.processing.is_none() && self.exporting.is_none()
    }

    fn reset_project(&mut self, confirmed: bool) {
        if !confirmed || !self.can_start_new_project() {
            return;
        }
        self.stop_audition();
        self.cancel_pending_imports();
        self.render_generation.invalidate();
        *self = Self {
            appearance: self.appearance,
            shortcuts: self.shortcuts.clone(),
            unlock_playhead_when_zooming: self.unlock_playhead_when_zooming,
            preferences_path: self.preferences_path.clone(),
            appearance_error: self.appearance_error.clone(),
            drawing_revision: self.drawing_revision.wrapping_add(1),
            window_visibility_generation: self.window_visibility_generation.wrapping_add(1),
            messages: vec!["New project. Import tracks to begin.".into()],
            ..Self::default()
        };
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
        let cache_directory = match crate::storage::working_directory(project.path()) {
            Ok(path) => path.join("analysis-cache"),
            Err(error) => {
                self.preview_error = Some(error.to_string());
                return;
            }
        };
        let progress = Arc::new(AtomicU8::new(0));
        let worker_progress = progress.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        let mut worker = std::mem::take(&mut self.supervised_worker);
        let feedback = Arc::new(Mutex::new("Starting analysis".into()));
        let worker_feedback = feedback.clone();
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let completed = worker
                .analyze_with_feedback(
                    &source_project,
                    &cache_directory,
                    &worker_progress,
                    &worker_cancelled,
                    &|message| {
                        *worker_feedback.lock().unwrap_or_else(|e| e.into_inner()) = message.into();
                    },
                )
                .map_err(|e| {
                    format!(
                        "Analysis failed (working folder '{}'): {e}",
                        cache_directory.display()
                    )
                });
            let _ = sender.send((completed, worker));
        });
        self.analysis_details = None;
        self.analysis_details_open = false;
        self.preview_processing = Some(PreviewProcessing {
            started_at: Instant::now(),
            feedback,
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
                self.reference_stale = !effective.analysis_current;
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
                self.drawing_revision = self.drawing_revision.wrapping_add(1);
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
        if let Some(playing) = self.audition.take() {
            playing.stop();
            self.idle_audition = Some(playing);
        }
        self.audition_requested_at = Some(Instant::now());
        if let Some(prepared) = self
            .audition_preparation
            .ready_sources(&project.audio_sources())
        {
            self.audition_range = None;
            self.audition_preparation.cancel();
            if let Err(error) =
                self.start_prepared_audition(prepared, envelopes, TimeRange { start, end })
            {
                self.messages = vec![error.to_string()];
            }
            return;
        }
        self.audition_range = Some((start, end));
        if !self.audition_preparation.is_requested() {
            self.audition_preparation
                .request_sources(project.audio_sources());
        }
    }

    fn start_prepared_audition(
        &mut self,
        prepared: crate::audition::PreparedAudio,
        envelopes: Vec<crate::GainEnvelope>,
        range: TimeRange,
    ) -> Result<(), crate::RenderBlocker> {
        let monitor = 10.0_f32.powf(self.monitor_db / 20.0);
        if let Some(idle) = self.idle_audition.take()
            && idle.restart(
                &prepared,
                envelopes.clone(),
                self.audition_mode,
                monitor,
                range,
                self.solo_tracks,
                self.reference_audition.is_some(),
            )?
        {
            self.audition = Some(idle);
            return Ok(());
        }
        self.audition = Some(LiveAudition::start_on_output(
            &mut self.audition_output,
            prepared,
            envelopes,
            self.audition_mode,
            monitor,
            range,
            self.solo_tracks,
            self.reference_audition.is_some(),
        )?);
        Ok(())
    }

    fn audition_status(&self) -> Option<&'static str> {
        if !self
            .audition_requested_at
            .is_some_and(|started| started.elapsed() >= Duration::from_millis(150))
        {
            return None;
        }
        if self.audition_range.is_some() && self.audition_preparation.is_requested() {
            Some("Preparing audio…")
        } else if self
            .audition
            .as_ref()
            .is_some_and(LiveAudition::is_buffering)
        {
            Some("Buffering audio…")
        } else {
            None
        }
    }

    fn begin_reference_audition(&mut self) {
        let Some((start, end)) = self.calibration_selection else {
            return;
        };
        let Ok(range) = TimeRange::new(start, end) else {
            return;
        };
        if end > self.timeline.total_samples() {
            return;
        }
        self.reference_audition = Some(ReferenceAudition {
            mode: self.audition_mode,
            solo: self.solo_tracks,
            range,
            source_generation: self.source_generation,
            paused: false,
        });
        self.audition_mode = AuditionMode::Original;
        self.solo_tracks = 0;
        self.audition(true);
    }

    fn close_reference_menu(&mut self) {
        self.reference_menu = None;
        if let Some(snapshot) = self.reference_audition.take() {
            self.stop_audition();
            self.audition_mode = snapshot.mode;
            self.solo_tracks = snapshot.solo;
        }
    }

    fn toggle_reference_audition(&mut self) {
        if let Some(audition) = &self.audition {
            audition.toggle_pause();
            if let Some(snapshot) = &mut self.reference_audition {
                snapshot.paused = !snapshot.paused;
            }
        } else if self.audition_range.is_some() {
            self.stop_audition();
        } else if let Some(snapshot) = &mut self.reference_audition {
            snapshot.paused = false;
            self.audition(true);
        }
    }

    fn gain_event(&mut self, track: usize, event: crate::GainEditorEvent) {
        if let crate::GainEditorEvent::FitRange(start, end) = event {
            let _ = self.update(Message::Timeline(TimelineEvent::FitRange(start, end)));
            return;
        }
        self.render_generation.invalidate();
        match event {
            crate::GainEditorEvent::FitRange(_, _) => unreachable!(),
            crate::GainEditorEvent::Begin(id, point) => {
                if let Some(project) = &self.project
                    && let Some(source) = project.sources.get(track)
                {
                    match crate::GainPointDrag::begin(project, source.track_id, id, point) {
                        Ok(drag) => {
                            self.gain_drag = Some(drag);
                            self.selected_example = None;
                            self.example_draft = None;
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
        use keyboard::{Key, key::Named};
        if self.sound_menu.is_some() || self.gain_menu.is_some() {
            if *key == Key::Named(Named::Escape) {
                self.sound_menu = None;
                self.gain_menu = None;
            }
            return true;
        }
        let popup = self.reference_menu.is_some();
        if *key == Key::Named(Named::Escape) {
            if popup {
                self.close_reference_menu();
                return true;
            }
            if self.gain_tool && self.gain_drag.is_some() {
                self.gain_drag = None;
                self.rebuild_effective();
                return true;
            }
            return false;
        }
        if popup && *key == Key::Named(Named::Enter) && modifiers.is_empty() {
            self.dispatch_shortcut(Command::SaveExample);
            return true;
        }
        let context = if popup {
            Context::Popup
        } else if self.gain_tool {
            Context::Gain
        } else {
            Context::Labels
        };
        if let Some(command) = self.shortcuts.resolve(key, modifiers, context) {
            self.dispatch_shortcut(command);
            return true;
        }
        popup
    }

    fn dispatch_shortcut(&mut self, command: Command) {
        use Command::*;
        if !self.action_available(command) {
            return;
        }
        let message = match command {
            SaveExample => Some(Message::ContextReference(
                self.reference_speakers.clone(),
                false,
            )),
            ClearSpeakers => {
                self.reference_speakers.clear();
                None
            }
            Loop => {
                self.toggle_reference_audition();
                None
            }
            Speaker1 | Speaker2 | Speaker3 | Speaker4 | Speaker5 | Speaker6 | Speaker7
            | Speaker8 => {
                let index = (0..8)
                    .find(|&index| Command::speaker(index) == Some(command))
                    .unwrap();
                self.project
                    .as_ref()
                    .and_then(|p| p.sources.get(index))
                    .map(|source| {
                        Message::ReferenceSpeaker(
                            source.speaker_id,
                            !self.reference_speakers.contains(&source.speaker_id),
                        )
                    })
            }
            Play => Some(
                if self.audition.is_some() || self.audition_range.is_some() {
                    Message::Stop
                } else {
                    Message::Audition
                },
            ),
            Start => {
                self.seek_to_selection_start();
                None
            }
            Original => Some(Message::Original),
            Processed => Some(Message::Processed),
            Removed => Some(Message::Removed),
            Center => Some(Message::CenterPlayhead),
            Lock => Some(Message::LockPlayhead),
            CatchUp => Some(Message::ToggleCatchUp),
            Meters => Some(Message::ToggleMeterPlacement),
            Help => Some(Message::Help),
            Menu => Some(Message::Menu),
            Reference => Some(Message::KeyboardShortcuts),
            SaveProject => Some(Message::SaveProject),
            UndoLabels => Some(Message::UndoReferences),
            RedoLabels => Some(Message::RedoReferences),
            UndoGain => Some(Message::UndoGain),
            RedoGain => Some(Message::RedoGain),
            DeleteGain => Some(Message::DeleteGain),
            FitRecording | FitSelection => {
                if self.project.is_some() {
                    if command == FitRecording {
                        self.timeline.fit_full();
                        self.user_zoomed();
                    } else if let Some((start, end)) = self.timeline.selection()
                        && start < end
                    {
                        self.timeline.fit_range(start, end);
                        self.user_zoomed();
                    }
                }
                None
            }
            Earlier | Later | Louder | Quieter => {
                let (time_delta, gain_delta) = match command {
                    Earlier => (-1_i64, 0.0),
                    Later => (1, 0.0),
                    Louder => (0, 0.01),
                    _ => (0, -0.01),
                };
                if let (Some((track, id, point)), Some(project)) =
                    (self.selected_gain_point, &self.project)
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
                }
                None
            }
        };
        if let Some(message) = message {
            let _ = self.update(message);
        }
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
                            match analysis.install(project, &mut self.effective_cache, None) {
                                Ok(effective) => {
                                    let timings = &analysis.timings;
                                    self.messages.clear();
                                    self.analysis_details = Some([
                                        (
                                            "Total analysis",
                                            processing.started_at.elapsed().as_secs_f64(),
                                        ),
                                        ("Source checks", timings.source_validation_ms / 1000.0),
                                        (
                                            "Audio preparation",
                                            timings.audio_preparation_ms / 1000.0,
                                        ),
                                        ("Examples", timings.reference_readiness_ms / 1000.0),
                                        ("Speaker analysis", timings.ownership_ms / 1000.0),
                                        (
                                            "Timeline",
                                            (timings.temporal_ms + timings.overview_ms) / 1000.0,
                                        ),
                                    ]);
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
                                    self.drawing_revision = self.drawing_revision.wrapping_add(1);
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
                Err("WAV preparation stopped unexpectedly".into())
            }
        };
        let processing = self.processing.take().unwrap();
        let current = !processing.cancelled.load(Ordering::Relaxed)
            && self.render_generation.current() == processing.generation
            && self
                .project
                .as_ref()
                .is_some_and(|project| processing.snapshot.matches(project));
        let mut accepted = false;
        match completed {
            Ok(artifact) if current => {
                let project = self.project.as_mut().unwrap();
                match artifact.accept(project) {
                    Ok(()) => {
                        accepted = true;
                        self.messages = vec!["WAV files prepared · ready to save".into()];
                        self.refresh_export_readiness();
                    }
                    Err(error) => {
                        self.messages = vec![format!("WAV preparation failed validation: {error}")]
                    }
                }
            }
            Ok(_) => {
                self.messages = vec![
                    "Settings changed during WAV preparation. Previous output is still available."
                        .into(),
                ]
            }
            Err(error) => self.messages = vec![format!("WAV files could not be prepared: {error}")],
        }
        if let Some((destination, settings)) = self.pending_export.take()
            && accepted
        {
            self.export_to_settings(destination, settings);
        }
    }
    fn poll_audition(&mut self) {
        if self.reference_audition.as_ref().is_some_and(|snapshot| {
            self.reference_menu.is_none()
                || snapshot.source_generation != self.source_generation
                || self.timeline.selection() != Some((snapshot.range.start, snapshot.range.end))
                || self.calibration_selection != Some((snapshot.range.start, snapshot.range.end))
        }) {
            self.close_reference_menu();
        }
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
                                    match self.start_prepared_audition(
                                        prepared,
                                        envelopes,
                                        TimeRange { start, end },
                                    ) {
                                        Ok(()) => {}
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
        if audition.failed() {
            self.timeline.cancel_catch_up();
            self.messages =
                vec!["Audio output stopped. Check the device and press Play to retry.".into()];
            self.audition = None;
            self.idle_audition = None;
            self.audition_output = None;
            self.audition_preparation.invalidate();
            self.audition_ready = false;
            return;
        }
        // Audio and completion checks continue while optional playhead/meter drawing sleeps.
        if self.window_minimized && !audition.is_finished() {
            return;
        }
        self.timeline.advance_playhead(audition.position());
        if self.timeline.follow_playhead(self.playback_scroll) {
            self.drawing_revision = self.drawing_revision.wrapping_add(1);
        }
        if let Some(review) = &mut self.review {
            review.seek(self.timeline.playhead());
        }
        if audition.is_finished()
            && let Some(finished) = self.audition.take()
        {
            self.timeline.cancel_catch_up();
            finished.stop();
            self.idle_audition = Some(finished);
        }
    }
    fn set_audition_mode(&mut self, mode: AuditionMode) {
        if self.reference_audition.is_some() {
            return;
        }
        if mode != AuditionMode::Original && self.preview.is_none() {
            return;
        }
        self.audition_mode = mode;
        if let Some(audition) = &self.audition {
            audition.set_mode(mode);
        }
    }

    fn stop_audition(&mut self) {
        self.timeline.cancel_catch_up();
        self.window_visibility_generation = self.window_visibility_generation.wrapping_add(1);
        self.window_minimized = false;
        if self.timeline.is_middle_panning() {
            self.update_playback_position();
        }
        if let Some(snapshot) = &mut self.reference_audition {
            snapshot.paused = true;
        }
        self.audition_range = None;
        self.audition_preparation.cancel();
        if let Some(audition) = self.audition.take() {
            audition.stop();
            self.idle_audition = Some(audition);
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
            self.calibration_error = Some("Enable speaker analysis before adding examples.".into());
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
        self.close_reference_menu();
        self.example_draft = None;
        self.selected_gain_point = None;
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
    fn app_support_open(&self) -> bool {
        self.help_open || self.shortcuts_open || self.menu_open || self.app_dialog.is_some()
    }

    fn workspace_input_active(&self) -> bool {
        !self.app_support_open()
            && !self.import_dialog
            && self.export_dialog.is_none()
            && self.reference_menu.is_none()
            && self.gain_menu.is_none()
    }

    fn sound_input_owns_keyboard(&self) -> bool {
        self.sound_draft.is_some() && self.workspace_input_active()
    }

    fn editing_track(&self) -> Option<usize> {
        if self.gain_tool
            && let Some((track, _, _)) = self.selected_gain_point
        {
            Some(track)
        } else if self.selected_example.is_some() {
            None
        } else {
            self.selection_track
        }
    }

    fn command_name(&self, command: Command) -> &'static str {
        match command {
            Command::Lock if self.playback_scroll == PlaybackScroll::Locked => "Unlock playhead",
            Command::Lock => "Lock playhead",
            Command::SaveExample if self.selected_example.is_some() => "Save changes",
            _ => command.definition().name,
        }
    }

    fn persist_shortcuts(&mut self) {
        self.appearance_error = self
            .preferences_path
            .as_ref()
            .ok_or_else(|| "No valid user configuration directory.".to_owned())
            .and_then(|path| {
                appearance::save_shortcuts(path, &self.shortcuts, self.unlock_playhead_when_zooming)
            })
            .err()
            .map(|error| format!("Preferences changed, but could not be saved. {error}"));
    }

    fn user_zoomed(&mut self) {
        if self.unlock_playhead_when_zooming && self.playback_scroll == PlaybackScroll::Locked {
            let _ = self.update(Message::LockPlayhead);
        }
        if self.playback_scroll == PlaybackScroll::Locked {
            self.timeline.follow_playhead(self.playback_scroll);
        }
    }

    fn gain_key_hint(&self) -> String {
        format!(
            "Drag pink points to adjust gain and fades.\nEarlier: {}; later: {}; up: {}; down: {}; remove: {}.",
            self.shortcuts.hint(Command::Earlier),
            self.shortcuts.hint(Command::Later),
            self.shortcuts.hint(Command::Louder),
            self.shortcuts.hint(Command::Quieter),
            self.shortcuts.hint(Command::DeleteGain)
        )
    }

    fn action_hint(&self, command: Command) -> String {
        let binding = self.shortcuts.hint(command);
        if binding == "Unbound" {
            format!("{} · Unbound", self.command_name(command))
        } else {
            format!("{} · {}", self.command_name(command), binding)
        }
    }

    fn action_available(&self, command: Command) -> bool {
        if matches!(
            command,
            Command::Center | Command::Lock | Command::CatchUp | Command::Meters
        ) {
            self.workspace_input_active()
                && self.sound_draft.is_none()
                && self.track_name_draft.is_none()
                && self.shortcut_capture.is_none()
                && self.sound_menu.is_none()
        } else {
            true
        }
    }

    fn action_control<'a>(
        &'a self,
        command: Command,
        control: crate::gui_controls::ControlButton<'a, Message>,
    ) -> Element<'a, Message> {
        self.shortcut_control(
            command,
            control.on_press_maybe(
                self.action_available(command)
                    .then_some(RegisteredButton(command)),
            ),
        )
    }

    fn shortcut_control<'a>(
        &'a self,
        command: Command,
        control: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let label = self.action_hint(command);
        let hint = iced::widget::tooltip(
            control,
            container(text(label).size(13))
                .padding(8)
                .style(editor_panel),
            iced::widget::tooltip::Position::Bottom,
        )
        .delay(Duration::from_millis(600))
        .snap_within_viewport(true);
        crate::gui_controls::shortcut_hint(hint, self.shortcut_hint_generation)
    }

    fn settings_action(&self) -> Message {
        match self.settings_selected {
            0 => Message::Appearance(Appearance::Dark),
            1 => Message::Appearance(Appearance::Light),
            2 => Message::UnlockOnZoom(!self.unlock_playhead_when_zooming),
            3 => Message::RestoreAllShortcuts,
            index => {
                let offset = index - 4;
                match DEFINITIONS.get(offset / 3) {
                    Some(definition) => match offset % 3 {
                        0 => Message::CaptureShortcut(definition.command),
                        1 => Message::RestoreShortcut(definition.command),
                        _ => Message::UnbindShortcut(definition.command),
                    },
                    None => Message::CloseHelp,
                }
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InvokeAction(command) => {
                self.dispatch_shortcut(command);
                return Task::none();
            }
            Message::CheckWindowVisibility => {
                self.update_playback_position();
                self.window_visibility_generation =
                    self.window_visibility_generation.wrapping_add(1);
                let generation = self.window_visibility_generation;
                return window::latest()
                    .and_then(window::is_minimized)
                    .map(move |minimized| Message::WindowVisibilityChecked(generation, minimized));
            }
            Message::WindowVisibilityChecked(generation, minimized) => {
                if generation == self.window_visibility_generation {
                    self.window_minimized = minimized.unwrap_or(false);
                    self.update_playback_position();
                }
                return Task::none();
            }
            _ => {}
        }
        let pan_end = match &message {
            Message::Timeline(TimelineEvent::PanEnd)
            | Message::TrackTimeline(_, TimelineEvent::PanEnd) => Some(false),
            Message::Timeline(TimelineEvent::PanCancel)
            | Message::TrackTimeline(_, TimelineEvent::PanCancel) => Some(true),
            _ => None,
        };
        if let Some(cancel) = pan_end {
            if self.timeline.is_middle_panning() {
                // Read the audio position while follow is suspended, before capturing the new anchor.
                self.update_playback_position();
                let moved = if cancel {
                    self.timeline.cancel_middle_pan(self.playback_scroll)
                } else {
                    self.timeline.finish_middle_pan(self.playback_scroll)
                };
                if moved {
                    self.drawing_revision = self.drawing_revision.wrapping_add(1);
                }
            }
            return Task::none();
        }
        if matches!(message, Message::ShortcutHintInput) {
            self.shortcut_hint_generation = self.shortcut_hint_generation.wrapping_add(1);
            return Task::none();
        }
        if let Some((command, ref proposed)) = self.shortcut_capture
            && let Message::KeyPressed(key, modifiers) = &message
        {
            if *key == keyboard::Key::Named(keyboard::key::Named::Enter)
                && modifiers.is_empty()
                && proposed.is_some()
            {
                return self.update(Message::ApplyShortcut);
            }
            if *key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                self.shortcut_capture = None;
                self.app_feedback = None;
            } else {
                match Binding::capture(key, *modifiers) {
                    Ok(binding) => {
                        self.app_feedback = self.shortcuts.validate(command, Some(&binding)).err();
                        self.shortcut_capture = Some((command, Some(binding)));
                    }
                    Err(error) => {
                        self.shortcut_capture = Some((command, None));
                        self.app_feedback = Some(error);
                    }
                }
            }
            return Task::none();
        }
        match &message {
            Message::UnlockOnZoom(unlock) => {
                self.unlock_playhead_when_zooming = *unlock;
                self.persist_shortcuts();
                return Task::none();
            }
            Message::CaptureShortcut(command) if self.app_dialog == Some(AppDialog::Settings) => {
                self.shortcut_capture = Some((*command, None));
                self.app_feedback = None;
                return iced::widget::operation::snap_to(
                    "app-dialog",
                    iced::widget::operation::RelativeOffset::START,
                );
            }
            Message::ApplyShortcut => {
                if let Some((command, Some(binding))) = self.shortcut_capture.clone() {
                    match self.shortcuts.assign(command, Some(binding)) {
                        Ok(()) => {
                            self.shortcut_capture = None;
                            self.app_feedback = None;
                            self.persist_shortcuts();
                        }
                        Err(error) => self.app_feedback = Some(error),
                    }
                }
                return Task::none();
            }
            Message::RestoreShortcut(command) | Message::UnbindShortcut(command) => {
                self.shortcut_capture = None;
                let result = if matches!(message, Message::RestoreShortcut(_)) {
                    self.shortcuts.restore(*command)
                } else {
                    self.shortcuts.assign(*command, None)
                };
                if result.is_ok() {
                    self.persist_shortcuts();
                }
                self.app_feedback = result.err();
                return Task::none();
            }
            Message::RestoreAllShortcuts => {
                self.shortcuts = Shortcuts::default();
                self.shortcut_capture = None;
                self.app_feedback = None;
                self.persist_shortcuts();
                return Task::none();
            }
            _ => {}
        }
        if let Message::Appearance(appearance) = message {
            if self.appearance != appearance {
                self.appearance = appearance;
                self.drawing_revision = self.drawing_revision.wrapping_add(1);
            }
            self.appearance_error = self
                .preferences_path
                .as_ref()
                .ok_or_else(|| "No valid user configuration directory.".to_string())
                .and_then(|path| appearance::save(path, appearance))
                .err()
                .map(|error| format!("Appearance changed, but could not be saved. {error}"));
            return Task::none();
        }
        let copy_reply = match &message {
            Message::VersionInfoReady(request, _) | Message::VersionCopied(request, _) => {
                Some(request)
            }
            _ => None,
        };
        if let Some(request) = copy_reply
            && (self.app_dialog != Some(AppDialog::About)
                || !self
                    .copy_request
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, request)))
        {
            return Task::none();
        }
        if let Message::VersionInfoReady(request, expected) = &message {
            self.version_details = Some(expected.clone());
            self.app_feedback = Some("Copying version information…".into());
            let expected = expected.clone();
            let request = request.clone();
            return iced::clipboard::write::<Message>(expected.clone()).chain(
                iced::clipboard::read().map(move |actual| {
                    Message::VersionCopied(
                        request.clone(),
                        actual.as_deref() == Some(expected.as_str()),
                    )
                }),
            );
        }
        if let Message::VersionCopied(_, copied) = &message {
            self.copy_request = None;
            self.app_feedback = Some(if *copied { "Version information copied." } else { "Could not confirm the copy. Try again or copy the displayed version information manually." }.into());
            return Task::none();
        }
        if self.menu_open || self.app_dialog.is_some() {
            match &message {
                Message::CloseHelp
                | Message::KeyPressed(keyboard::Key::Named(keyboard::key::Named::Escape), _) => {
                    self.menu_open = false;
                    self.app_dialog = None;
                    self.copy_request = None;
                    self.shortcut_capture = None;
                    return Task::none();
                }
                Message::KeyPressed(keyboard::Key::Named(key), modifiers) => {
                    use keyboard::key::Named;
                    match key {
                        Named::Tab | Named::ArrowDown | Named::ArrowUp if self.menu_open => {
                            self.menu_selected = if *key == Named::ArrowUp
                                || (*key == Named::Tab && modifiers.shift())
                            {
                                (self.menu_selected + 6) % 7
                            } else {
                                (self.menu_selected + 1) % 7
                            };
                        }
                        Named::Tab if self.app_dialog == Some(AppDialog::About) => {
                            self.about_copy_selected = !self.about_copy_selected
                        }
                        Named::Tab | Named::ArrowDown | Named::ArrowUp
                            if self.app_dialog == Some(AppDialog::Settings) =>
                        {
                            self.settings_selected = if *key == Named::ArrowUp
                                || (*key == Named::Tab && modifiers.shift())
                            {
                                (self.settings_selected + 4 + DEFINITIONS.len() * 3)
                                    % (5 + DEFINITIONS.len() * 3)
                            } else {
                                (self.settings_selected + 1) % (5 + DEFINITIONS.len() * 3)
                            };
                            return iced::widget::operation::snap_to(
                                "app-dialog",
                                iced::widget::operation::RelativeOffset {
                                    x: 0.0,
                                    y: self.settings_selected.saturating_sub(4) as f32
                                        / (DEFINITIONS.len() * 3) as f32,
                                },
                            );
                        }
                        Named::Enter if self.app_dialog == Some(AppDialog::Settings) => {
                            return self.update(self.settings_action());
                        }
                        Named::Enter if self.menu_open => {
                            return self.update(menu_action(self.menu_selected));
                        }
                        Named::Enter
                            if self.app_dialog == Some(AppDialog::About)
                                && self.about_copy_selected =>
                        {
                            return self.update(Message::CopyVersion);
                        }
                        Named::Enter => return self.update(Message::CloseHelp),
                        Named::PageDown | Named::ArrowDown | Named::PageUp | Named::ArrowUp => {
                            return iced::widget::operation::scroll_by(
                                "app-dialog",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: if matches!(key, Named::PageUp | Named::ArrowUp) {
                                        -400.0
                                    } else {
                                        400.0
                                    },
                                },
                            );
                        }
                        Named::Home => {
                            return iced::widget::operation::snap_to(
                                "app-dialog",
                                iced::widget::operation::RelativeOffset::START,
                            );
                        }
                        Named::End => return iced::widget::operation::snap_to_end("app-dialog"),
                        _ => {}
                    }
                    return Task::none();
                }
                Message::KeyPressed(key, modifiers)
                    if self.shortcuts.resolve(key, *modifiers, Context::Labels)
                        == Some(Command::Reference) =>
                {
                    return self.update(Message::KeyboardShortcuts);
                }
                Message::Settings
                | Message::Appearance(_)
                | Message::About
                | Message::KeyboardShortcuts
                | Message::OpenLink(_)
                | Message::LinkOpened(_, _)
                | Message::CopyVersion
                | Message::VersionInfoReady(..)
                | Message::VersionCopied(..)
                | Message::Tick
                | Message::PlaybackFrame => {}
                _ => return Task::none(),
            }
        }
        if self.help_open || self.shortcuts_open {
            match &message {
                Message::KeyPressed(keyboard::Key::Named(keyboard::key::Named::Escape), _)
                | Message::CloseHelp => {
                    self.help_open = false;
                    self.shortcuts_open = false;
                    return Task::none();
                }
                Message::KeyPressed(keyboard::Key::Named(key), _) => {
                    use keyboard::key::Named;
                    let operation = iced::widget::operation::scroll_by;
                    return match key {
                        Named::Tab | Named::ArrowDown | Named::ArrowUp if self.help_open => {
                            self.help_close_selected = !self.help_close_selected;
                            Task::none()
                        }
                        Named::Enter => {
                            return self.update(
                                if self.shortcuts_open || self.help_close_selected {
                                    Message::CloseHelp
                                } else {
                                    Message::KeyboardShortcuts
                                },
                            );
                        }
                        Named::Tab => Task::none(),
                        Named::PageDown | Named::ArrowDown => operation(
                            "shortcut-reference",
                            iced::widget::operation::AbsoluteOffset {
                                x: 0.0,
                                y: if *key == Named::PageDown { 400.0 } else { 40.0 },
                            },
                        ),
                        Named::PageUp | Named::ArrowUp => operation(
                            "shortcut-reference",
                            iced::widget::operation::AbsoluteOffset {
                                x: 0.0,
                                y: if *key == Named::PageUp { -400.0 } else { -40.0 },
                            },
                        ),
                        Named::Home => iced::widget::operation::snap_to(
                            "shortcut-reference",
                            iced::widget::operation::RelativeOffset::START,
                        ),
                        Named::End => iced::widget::operation::snap_to_end("shortcut-reference"),
                        _ => Task::none(),
                    };
                }
                Message::KeyPressed(key, modifiers)
                    if self.help_open
                        && self.shortcuts.resolve(key, *modifiers, Context::Labels)
                            == Some(Command::Reference) =>
                {
                    return self.update(Message::KeyboardShortcuts);
                }
                Message::KeyboardShortcuts | Message::Tick | Message::PlaybackFrame => {}
                _ => return Task::none(),
            }
        }
        if self.import_dialog
            && matches!(
                &message,
                Message::KeyPressed(keyboard::Key::Named(keyboard::key::Named::Escape), _)
            )
        {
            return self.update(Message::CancelChannels);
        }
        if let Message::KeyPressed(keyboard::Key::Named(keyboard::key::Named::Tab), modifiers) =
            &message
            && !self.app_support_open()
            && self.reference_menu.is_none()
            && self.gain_menu.is_none()
            && !self.sound_input_owns_keyboard()
            && (self.track_name_draft.is_none()
                || self.import_dialog
                || self.export_dialog.is_some())
        {
            return crate::gui_controls::focus_next(modifiers.shift());
        }
        if self.export_dialog.is_some() {
            if matches!(
                &message,
                Message::KeyPressed(keyboard::Key::Named(keyboard::key::Named::Escape), _)
            ) {
                self.export_dialog = None;
                return Task::none();
            }
            if !matches!(
                &message,
                Message::ExportLayout(_)
                    | Message::ExportEncoding(_)
                    | Message::ExportDestination
                    | Message::CancelExportDialog
                    | Message::SubmitExport
                    | Message::Tick
                    | Message::PlaybackFrame
            ) {
                return Task::none();
            }
        }
        if matches!(message, Message::PlaybackFrame) {
            self.update_playback_position();
            return Task::none();
        }
        // Polls invalidate geometry only when they install changed timeline data.
        if !matches!(message, Message::Tick) {
            self.drawing_revision = self.drawing_revision.wrapping_add(1);
        }
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
        if self.import_dialog
            && !matches!(
                message,
                Message::FileDropped(_)
                    | Message::ImportChannel(_, _)
                    | Message::ImportChannelName(_, _)
                    | Message::ConfirmChannels
                    | Message::CancelChannels
                    | Message::ImportStems
                    | Message::CancelPendingImports
                    | Message::RemoveImport(_)
                    | Message::RetryImport(_)
                    | Message::Tick
                    | Message::SoundModifiers(_)
            )
        {
            return Task::none();
        }
        if matches!(
            &message,
            Message::AnalyzeReferences
                | Message::Cancel
                | Message::CancelAnalysis
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
            Message::InvokeAction(_) => unreachable!("actions dispatch before input routing"),
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
                let count = self.reserved_track_count() + additions.len();
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
                if self
                    .source_loading
                    .as_ref()
                    .is_some_and(|loading| loading.committing)
                {
                    return Task::none();
                }
                self.source_loading = None;
                self.source_inspection = None;
                self.pending_imports.clear();
                self.import_due = None;
                self.import_choices.clear();
                self.import_dialog = false;
                self.messages.clear();
            }
            Message::CancelPendingImports => self.cancel_pending_imports(),
            Message::RemoveImport(id) => {
                self.import_jobs.retain(|job| job.id != id || job.accepted);
                self.start_next_import();
            }
            Message::RetryImport(id) => {
                if let Some(index) = self
                    .import_jobs
                    .iter()
                    .position(|job| job.id == id && !job.accepted && job.error.is_some())
                {
                    self.import_jobs[index].retry();
                    self.start_next_import();
                }
            }
            Message::EditTrackName(index) => {
                if let Some(source) = self.project.as_ref().and_then(|p| p.sources.get(index)) {
                    self.track_name_draft = Some((index, source.display_name()));
                    self.messages.clear();
                    return iced::widget::operation::focus("track-name")
                        .chain(iced::widget::operation::select_all("track-name"));
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
                            self.messages.clear();
                            self.render_generation.invalidate();
                            self.mark_reference_edit();
                        }
                        Err(error) => self.messages = vec![error.to_string()],
                    }
                }
            }
            Message::CancelTrackName => {
                self.track_name_draft = None;
                self.messages.clear();
            }
            Message::ShowExample(id) | Message::PlayExample(id) => {
                let play = matches!(message, Message::PlayExample(_));
                self.stop_audition();
                self.select_example(id);
                if self.selected_example == Some(id) {
                    if let Some((start, end)) = self.timeline.selection() {
                        self.timeline.show_range(start, end);
                        if self.playback_scroll == PlaybackScroll::Locked {
                            self.playback_scroll = PlaybackScroll::Off;
                        }
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
                            if self.playback_scroll == PlaybackScroll::Locked {
                                self.playback_scroll = PlaybackScroll::Off;
                            }
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
                if matches!(
                    event,
                    TimelineEvent::FitRange(_, _) | TimelineEvent::PanStart
                ) || (matches!(event, TimelineEvent::Pan { .. })
                    && self.timeline.is_middle_panning())
                {
                    return self.update(Message::Timeline(event));
                }
                self.close_reference_menu();
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
                    self.selected_gain_point = None;
                    self.selection_track = Some(track);
                }
                return task;
            }
            Message::OpenReferenceMenu(track, point) => {
                self.close_reference_menu();
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
                    self.begin_reference_audition();
                }
            }
            Message::OpenTimelineMenu(track, time, point) => {
                self.close_reference_menu();
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
                    self.begin_reference_audition();
                    return Task::none();
                }
                return self.update(Message::OpenGainMenu(track, time, point));
            }
            Message::OpenGainMenu(track, time, point) => {
                self.close_reference_menu();
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
                self.close_reference_menu();
                self.gain_menu = None;
            }
            Message::SetupDetails(show) => self.setup_details = show,
            Message::AnalysisDetails(show) => self.analysis_details_open = show,
            Message::ToggleCatchUp => {
                self.timeline.override_middle_pan_anchor();
                self.timeline.cancel_catch_up();
                self.playback_scroll = if self.playback_scroll == PlaybackScroll::Page {
                    PlaybackScroll::Off
                } else {
                    PlaybackScroll::Page
                };
            }
            Message::LockPlayhead => {
                self.timeline.override_middle_pan_anchor();
                if self.playback_scroll == PlaybackScroll::Locked {
                    self.timeline.cancel_catch_up();
                    self.playback_scroll = PlaybackScroll::Off;
                } else {
                    self.timeline.lock_playhead();
                    self.playback_scroll = PlaybackScroll::Locked;
                    self.timeline.follow_playhead(self.playback_scroll);
                }
            }
            Message::CenterPlayhead => self.timeline.center_playhead(),
            Message::ToggleMeterPlacement => self.meters_at_playhead = !self.meters_at_playhead,
            Message::WaveformDisplay(display) => self.waveform_display = display,
            Message::SoloTrack(track) => {
                if self.reference_audition.is_some() {
                    return Task::none();
                }
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
            Message::ContextReference(speakers, replace) => {
                if let Some((track, _)) = self.reference_menu
                    && self.selection_track == Some(track)
                {
                    self.reference_speakers = speakers;
                    let _ = self.update(Message::SaveReference(replace));
                    if self.calibration_error.is_none() {
                        self.close_reference_menu();
                        self.calibrating = false;
                    }
                }
            }
            Message::ImportStems => self.import_stems(),
            Message::NewProject => {
                if self.can_start_new_project() {
                    let confirmed = self.project.is_none()
                        || rfd::MessageDialog::new()
                            .set_title("Start a new project?")
                            .set_description("This clears the current workspace. Unsaved changes will be lost. Choose No and save your project first if you want to keep them. Saved projects and source audio stay on disk.")
                            .set_level(rfd::MessageLevel::Warning)
                            .set_buttons(rfd::MessageButtons::YesNo)
                            .show()
                            == rfd::MessageDialogResult::Yes;
                    self.reset_project(confirmed);
                }
                return Task::none();
            }
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
            #[cfg(test)]
            Message::Process => self.start_processing(),
            Message::Cancel => {
                self.pending_export = None;
                if let Some(processing) = &self.processing {
                    processing.cancelled.store(true, Ordering::Relaxed);
                }
                if let Some(exporting) = &self.exporting {
                    exporting.cancelled.store(true, Ordering::Relaxed);
                }
            }
            Message::Tick => {
                self.poll_sources();
                if self.source_loading.is_none() {
                    self.poll_processing();
                }
                self.poll_audition();
                self.poll_exporting();
            }
            Message::PlaybackFrame
            | Message::CheckWindowVisibility
            | Message::WindowVisibilityChecked(_, _)
            | Message::Appearance(_)
            | Message::ShortcutHintInput
            | Message::UnlockOnZoom(_) => unreachable!(),
            Message::CaptureShortcut(_)
            | Message::ApplyShortcut
            | Message::RestoreShortcut(_)
            | Message::RestoreAllShortcuts
            | Message::UnbindShortcut(_) => {}
            #[cfg(test)]
            Message::SpeechSensitivity(value) => {
                self.preview_parameters.speech_sensitivity = 1.0 - value
            }
            Message::EditSound(parameter) => {
                if self.sound_modifiers.control() {
                    return self.update(Message::ResetSound(parameter));
                }
                let _ = self.update(Message::CommitSound);
                self.sound_draft = Some((
                    parameter,
                    parameter.value(self.preview_parameters).to_string(),
                ));
                self.sound_error = None;
                self.sound_typing = true;
                self.sound_menu = None;
                return iced::widget::operation::focus("sound-value")
                    .chain(iced::widget::operation::select_all("sound-value"));
            }
            Message::SoundText(value) => {
                if let Some((_, draft)) = &mut self.sound_draft {
                    *draft = value;
                }
                self.sound_typing = true;
                self.sound_error = None;
            }
            Message::CommitSound => self.commit_sound(),
            Message::CancelSound => {
                self.sound_draft = None;
                self.sound_error = None;
                self.sound_typing = false;
            }
            Message::CheckSoundFocus => {
                if self.sound_draft.is_some() {
                    return iced::widget::operation::is_focused("sound-value")
                        .map(Message::SoundFocused);
                }
            }
            Message::SoundFocused(focused) => {
                self.sound_typing = focused;
                if !focused {
                    self.commit_sound();
                }
            }
            Message::SoundTab(reverse) => {
                let origin = self.sound_draft.as_ref().map(|(parameter, _)| *parameter);
                let _ = self.update(Message::CommitSound);
                self.sound_typing = false;
                let next = crate::gui_controls::focus_next(reverse);
                // Only restore a disappearing input; invalid drafts keep their focus tree.
                return if let Some(parameter) = origin
                    && self.sound_draft.is_none()
                {
                    iced::widget::operation::focus(parameter.control_id()).chain(next)
                } else {
                    next
                };
            }
            Message::SoundModifiers(modifiers) => {
                self.sound_modifiers = modifiers;
                self.selecting_range = modifiers.shift();
            }
            Message::SoundMenu(parameter) => self.sound_menu = Some(parameter),
            Message::ResetSound(parameter) => {
                self.sound_draft = None;
                self.sound_error = None;
                self.sound_typing = false;
                self.sound_menu = None;
                parameter.set(
                    &mut self.preview_parameters,
                    parameter.value(AutomixParameters::default()),
                );
            }
            Message::Mute => {
                self.sound_draft = None;
                self.sound_error = None;
                self.sound_typing = false;
                self.preview_parameters.mute = !self.preview_parameters.mute;
            }
            Message::Attenuation(value) => {
                SoundParameter::Attenuation.set(&mut self.preview_parameters, value)
            }
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
            Message::Audition => {
                if self.reference_audition.is_some() {
                    self.toggle_reference_audition();
                } else {
                    self.audition(false);
                }
            }
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
                if self.sound_typing {
                    return Task::none();
                }
                if self.track_name_draft.is_some() {
                    if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                        return self.update(Message::CancelTrackName);
                    }
                    return Task::none();
                }
                if self.handle_edit_key(&key, modifiers) {
                    return Task::none();
                }
                if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                    self.selected_gain_point = None;
                    self.timeline.set_selection(None);
                    self.selection_track = None;
                    self.calibration_selection = None;
                    self.selected_example = None;
                    self.example_draft = None;
                    if let Some(review) = &mut self.review {
                        review.clear_selection();
                    }
                }
            }
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
                            self.close_reference_menu();
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
                self.gain_menu = None;
                self.rebuild_effective();
            }
            Message::GainEvent(track, event) => {
                if self.gain_tool {
                    self.gain_event(track, event);
                }
            }
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
                self.close_reference_menu();
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
            Message::ExportLayout(layout) => {
                if let Some(dialog) = &mut self.export_dialog {
                    dialog.settings.layout = layout;
                }
            }
            Message::ExportEncoding(encoding) => {
                if let Some(dialog) = &mut self.export_dialog {
                    dialog.settings.encoding = encoding;
                }
            }
            Message::ExportDestination => {
                let destination = rfd::FileDialog::new()
                    .set_directory(dialog_start_directory())
                    .pick_folder()
                    .map(|path| available_export_path(&path));
                if let Some(dialog) = &mut self.export_dialog {
                    dialog.destination = destination;
                }
            }
            Message::CancelExportDialog => self.export_dialog = None,
            Message::Help => {
                self.help_open = true;
                self.help_close_selected = false;
            }
            Message::KeyboardShortcuts => {
                self.shortcut_capture = None;
                self.menu_open = false;
                self.app_dialog = None;
                self.help_open = false;
                self.shortcuts_open = true;
            }
            Message::CloseHelp => {
                self.shortcut_capture = None;
                self.menu_open = false;
                self.app_dialog = None;
                self.copy_request = None;
                self.help_open = false;
                self.shortcuts_open = false;
            }
            Message::Menu => {
                self.menu_open = true;
                self.menu_selected = 0;
                self.app_feedback = None;
            }
            Message::Settings | Message::About => {
                self.copy_request = None;
                self.version_details = None;
                self.settings_selected = if self.appearance == Appearance::Dark {
                    0
                } else {
                    1
                };
                self.menu_open = false;
                self.app_dialog = Some(if matches!(message, Message::Settings) {
                    AppDialog::Settings
                } else {
                    AppDialog::About
                });
                self.about_copy_selected = true;
                self.app_feedback = None;
            }
            Message::OpenLink(link) => {
                self.app_feedback = Some("Opening in your browser…".into());
                let (sender, receiver) = iced::futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let _ = sender.send(open_app_link(link));
                });
                return Task::perform(
                    async move {
                        receiver
                            .await
                            .unwrap_or_else(|_| Err("The link launcher stopped.".into()))
                    },
                    move |result| Message::LinkOpened(link, result),
                );
            }
            Message::LinkOpened(link, result) => {
                self.app_feedback = Some(match result {
                    Ok(()) => "Opened in your browser.".into(),
                    Err(error) => format!(
                        "Could not open the link: {error}\nOpen {} manually.",
                        link.url()
                    ),
                })
            }
            Message::CopyVersion => {
                if self.app_dialog == Some(AppDialog::About) && self.copy_request.is_none() {
                    // A retained token cannot be reused by a reopened dialog's newer request.
                    let request = Arc::new(());
                    self.copy_request = Some(request.clone());
                    self.app_feedback = Some("Collecting graphics information…".into());
                    return iced::system::information().then(move |information| {
                        let request = request.clone();
                        window::latest().then(move |id| {
                            let backend = information.graphics_backend.clone();
                            let adapter = information.graphics_adapter.clone();
                            let request = request.clone();
                            let complete = move |scale| {
                                Message::VersionInfoReady(
                                    request.clone(),
                                    version_info_with_graphics(
                                        version_info(
                                            env!("CARGO_PKG_VERSION"),
                                            None,
                                            std::env::consts::OS,
                                            std::env::consts::ARCH,
                                        ),
                                        &backend,
                                        &adapter,
                                        scale,
                                    ),
                                )
                            };
                            match id {
                                Some(id) => {
                                    window::scale_factor(id).map(move |scale| complete(Some(scale)))
                                }
                                None => Task::done(complete(None)),
                            }
                        })
                    });
                }
            }
            Message::CopyProjectPath => {
                if let Some(project) = &self.project {
                    return iced::clipboard::write::<Message>(project.path().display().to_string());
                }
            }
            Message::VersionInfoReady(..) | Message::VersionCopied(..) => {}
            Message::SubmitExport => self.submit_export(),
            Message::Timeline(event) => match event {
                TimelineEvent::FitRange(start, end) => {
                    if self.project.is_some() && start < end {
                        self.timeline.fit_range(start, end);
                        self.user_zoomed();
                    }
                }
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
                    if direction != 0.0 {
                        self.user_zoomed();
                    }
                }
                TimelineEvent::Pan { delta_x, width } => {
                    if !self.timeline.is_middle_panning()
                        && self.playback_scroll == PlaybackScroll::Locked
                    {
                        self.playback_scroll = PlaybackScroll::Off;
                    }
                    self.timeline.pan_by_pixels(delta_x, width)
                }
                TimelineEvent::PanStart => {
                    self.timeline.begin_middle_pan(self.playback_scroll);
                }
                TimelineEvent::PanEnd | TimelineEvent::PanCancel => {
                    unreachable!("Pan cleanup runs before modal input guards")
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
        let pan_cleanup = if self.timeline.is_middle_panning() {
            iced::event::listen_with(|event, _, _| match event {
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Middle,
                )) => Some(Message::Timeline(TimelineEvent::PanEnd)),
                iced::Event::Window(window::Event::Unfocused)
                | iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => Some(Message::Timeline(TimelineEvent::PanCancel)),
                _ => None,
            })
        } else {
            Subscription::none()
        };
        let shortcut_hints = iced::event::listen_with(|event, _, _| match event {
            // Reset on release so rebuilding the hint never loses a button's pending click.
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(_))
            | iced::Event::Keyboard(keyboard::Event::KeyPressed { .. })
            | iced::Event::Window(window::Event::Unfocused) => Some(Message::ShortcutHintInput),
            _ => None,
        });
        let drops = iced::event::listen_with(if self.sound_input_owns_keyboard() {
            |event, status, _| application_input_event(event, status, true)
        } else {
            |event, status, _| application_input_event(event, status, false)
        });
        let processing = if self.processing.is_some()
            || self.exporting.is_some()
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
        let playback = if self.audition.is_some() && !self.window_minimized {
            window::frames().map(|_| Message::PlaybackFrame)
        } else {
            Subscription::none()
        };
        // Iced has no minimized event. Query only during playback, at two checks per second.
        let visibility = if self.audition.is_some() {
            iced::time::every(Duration::from_millis(500)).map(|_| Message::CheckWindowVisibility)
        } else {
            Subscription::none()
        };
        let window_changes = iced::event::listen_with(|event, _, _| match event {
            iced::Event::Window(
                window::Event::Focused | window::Event::Unfocused | window::Event::Resized(_),
            ) => Some(Message::CheckWindowVisibility),
            _ => None,
        });
        let rename_keys = if self.track_name_draft.is_some() && self.workspace_input_active() {
            // Text inputs capture Escape, so cancellation must also see captured events.
            iced::event::listen_with(|event, _, _| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => Some(Message::CancelTrackName),
                _ => None,
            })
        } else {
            Subscription::none()
        };
        let sound_keys = if self.sound_input_owns_keyboard() {
            iced::event::listen_with(|event, _, _| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => Some(Message::CancelSound),
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Tab),
                    modifiers,
                    ..
                }) => Some(Message::SoundTab(modifiers.shift())),
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(_))
                | iced::Event::Keyboard(keyboard::Event::KeyReleased { .. }) => {
                    Some(Message::CheckSoundFocus)
                }
                iced::Event::Window(window::Event::Unfocused) => Some(Message::SoundFocused(false)),
                _ => None,
            })
        } else {
            Subscription::none()
        };
        let import_keys = if self.import_dialog {
            iced::event::listen_with(|event, _, _| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => Some(Message::CancelChannels),
                _ => None,
            })
        } else {
            Subscription::none()
        };
        let help_keys = if self.app_support_open() {
            iced::event::listen_with(|event, status, _| match event {
                iced::Event::Keyboard(event @ keyboard::Event::KeyPressed { .. })
                    if status == iced::event::Status::Captured =>
                {
                    application_keyboard_event(event, iced::event::Status::Ignored)
                }
                _ => None,
            })
        } else {
            Subscription::none()
        };
        Subscription::batch([
            pan_cleanup,
            shortcut_hints,
            drops,
            processing,
            playback,
            visibility,
            window_changes,
            rename_keys,
            sound_keys,
            import_keys,
            help_keys,
        ])
    }

    fn shortcut_reference(&self) -> Element<'_, Message> {
        let mut content = column![
            row![text("Keyboard shortcuts").size(24), Space::new().width(Fill),
                button("Close    Enter / Esc").on_press(LegacyButton::CloseHelp)].spacing(16),
            text("Effective application bindings are shown below. Text fields, dialogs and speaker popups own input before workspace commands. Change bindings in Settings.").size(14),
        ].spacing(12).width(Fill).padding([0, 12]);
        let commands = DEFINITIONS
            .iter()
            .map(|definition| {
                (
                    self.command_name(definition.command).to_owned(),
                    self.shortcuts.hint(definition.command),
                    format!(
                        "{}{}",
                        definition.context.name(),
                        if definition.command == Command::SaveExample {
                            " · Enter also confirms."
                        } else {
                            ""
                        }
                    ),
                )
            })
            .collect::<Vec<_>>();
        let fixed = [
            (
                "Cancel interaction or clear selection",
                "Escape",
                "Cancel the active edit first; otherwise clear selection.",
            ),
            ("Seek", "Click waveform", ""),
            ("Select range and label speakers", "Drag waveform", ""),
            (
                "Fit temporary range",
                "Alt+drag",
                "Keeps selection, labels and gain. Zoom lock preference applies.",
            ),
            (
                "Zoom around pointer",
                "Wheel | Ctrl+wheel",
                "Preserves lock unless Unlock playhead when zooming is enabled.",
            ),
            (
                "Pan",
                "Middle-button drag",
                "Repositions an existing playhead lock on release.",
            ),
            (
                "Adjust sound parameter",
                "Drag dial | Wheel",
                "Hold Shift for fine adjustment.",
            ),
            (
                "Type exact value",
                "Double-click dial | Click value",
                "Enter saves; Escape cancels.",
            ),
            (
                "Select or resize example",
                "Click label strip | Drag selected handles",
                "Escape cancels resizing.",
            ),
            (
                "Open actions",
                "Right-click waveform | Right-click label | Right-click gain",
                "",
            ),
            (
                "Commit value / track name",
                "Enter",
                "Text entry owns its keys.",
            ),
            (
                "Cancel value / track name",
                "Escape",
                "Keeps the last valid value.",
            ),
            (
                "Focus next / previous",
                "Tab | Shift+Tab",
                "Standard widget focus navigation is reserved.",
            ),
            (
                "Reset sound parameter",
                if cfg!(target_os = "macos") {
                    "Command+click | Right-click value"
                } else {
                    "Ctrl+click | Right-click value"
                },
                "",
            ),
            (
                "Confirm / close dialog",
                "Enter | Escape",
                "Dialogs own confirmation and cancellation.",
            ),
            (
                "Scroll reference / credits",
                "PageUp | PageDown | Home | End",
                "",
            ),
        ]
        .into_iter()
        .map(|(action, binding, condition)| {
            (action.to_owned(), binding.to_owned(), condition.to_owned())
        })
        .collect::<Vec<_>>();
        for (heading, entries) in [
            ("Application commands", commands),
            ("Mouse, text & dialogs", fixed),
        ] {
            content = content
                .push(Space::new().height(16))
                .push(text(heading).size(18));
            for (action, binding, condition) in entries {
                content = content.push(
                    responsive(move |size| {
                        let mut description = column![text(action.clone()).size(14)]
                            .spacing(4)
                            .width(Fill);
                        if !condition.is_empty() {
                            description = description.push(text(condition.clone()).size(13).style(
                                |theme: &Theme| text::Style {
                                    color: Some(colors(theme).muted),
                                },
                            ));
                        }
                        let mut bindings = column![].spacing(5);
                        for (alternative_index, alternative) in binding.split(" | ").enumerate() {
                            if alternative_index > 0 {
                                bindings =
                                    bindings.push(text("or").size(12).style(|theme: &Theme| {
                                        text::Style {
                                            color: Some(colors(theme).muted),
                                        }
                                    }));
                            }
                            let visual: Element<'_, Message> =
                                if alternative.to_lowercase().contains("click")
                                    || alternative.to_lowercase().contains("drag")
                                    || alternative.to_lowercase().contains("wheel")
                                    || alternative == "Typing"
                                {
                                    text(alternative.to_owned()).size(13).into()
                                } else {
                                    let mut chord = row![].spacing(4).align_y(iced::Center);
                                    for (index, key) in alternative.split('+').enumerate() {
                                        if index > 0 {
                                            chord = chord.push(text("+").size(12));
                                        }
                                        chord = chord.push(
                                            container(text(key.to_owned()).size(13))
                                                .padding([3, 6])
                                                .style(|theme: &Theme| {
                                                    let c = colors(theme);
                                                    container::Style {
                                                        text_color: Some(c.text),
                                                        background: Some(c.control.into()),
                                                        border: iced::Border {
                                                            color: c.border,
                                                            width: 1.0,
                                                            radius: 3.0.into(),
                                                        },
                                                        ..Default::default()
                                                    }
                                                }),
                                        );
                                    }
                                    chord.wrap().into()
                                };
                            bindings = bindings.push(visual);
                        }
                        let body: Element<'_, Message> = if size.width < 560.0 {
                            column![description, bindings].spacing(8).into()
                        } else {
                            row![description, bindings.width(220)].spacing(20).into()
                        };
                        column![
                            container(body).padding([10, 0]).width(Fill),
                            container(Space::new().height(1))
                                .width(Fill)
                                .style(|theme: &Theme| container::Style {
                                    background: Some(colors(theme).panel_border.into()),
                                    ..Default::default()
                                }),
                        ]
                        .width(Fill)
                        .into()
                    })
                    .height(Length::Shrink),
                );
            }
        }
        content.into()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.menu_open || self.app_dialog.is_some() {
            return responsive(|size| {
                let content: Element<'_, Message> = if self.menu_open {
                    let mut items = column![container(text("Menu").size(20)).padding([4, 12])].spacing(4).width(Fill);
                    for (index, label) in ["Settings", "Keyboard shortcuts", "Help", "GitHub", "Report a bug", "About", "Close"].into_iter().enumerate() {
                        if matches!(index, 2 | 5 | 6) {
                            items = items.push(container(iced::widget::rule::horizontal(1)).padding([6, 12]));
                        }
                        let hint = match index {
                            1 => self.shortcuts.hint(Command::Reference),
                            6 => "Esc".into(),
                            _ => String::new(),
                        };
                        let selected = index == self.menu_selected;
                        items = items.push(button(row![text(label).width(Fill), text(hint).size(14)].spacing(16).align_y(iced::Center))
                            .width(Fill).padding([10, 12]).keyboard_enabled(false).selected(selected)
                            .style(move |theme, status| flat_button(theme, status, selected))
                            .on_press(legacy_menu_action(index)));
                    }
                    if let Some(feedback) = &self.app_feedback { items = items.push(container(text(feedback).size(14)).padding([6, 12])); }
                    container(scrollable(items).id("app-dialog")).max_height((size.height - 80.0).max(120.0)).into()
                } else {
                    let about = self.app_dialog == Some(AppDialog::About);
                    let mut body = column![text(if about { "About CleanSlate" } else { "Settings" }).size(24)].spacing(14).width(Fill).padding([0, 12]);
                    if about {
                        body = body.push(text(self.version_details.clone().unwrap_or_else(|| version_info(env!("CARGO_PKG_VERSION"), None, std::env::consts::OS, std::env::consts::ARCH))))
                            .push(text("License: MIT · Copyright © 2026 CleanSlate contributors"))
                            .push(text("CleanSlate is a local, non-destructive podcast automixer."))
                            .push(text("Credits & references").size(18))
                            .push(text("Interface: Iced. Audio playback: Rodio. Decoding: Symphonia and bwavfile. Resampling: Rubato. Ownership models: Linfa, nalgebra and ndarray. Serialization: Serde. Integrity: SHA-2."))
                            .push(text("Speech detection: embedded Silero VAD, used through ONNX Runtime (ort). Silero VAD and ONNX Runtime carry MIT licenses. Bundled runtime notices and each component's licenses remain authoritative; CleanSlate's MIT license does not relicense dependencies."))
                            .push(text("See the distribution's THIRD-PARTY-NOTICES.txt and licenses directory. Model references and runtime notices are recorded in docs/licenses/embedded-model-runtime.txt in the repository."))
                            .push(text("Outline icons: Lucide Icons and Contributors; ISC, with MIT notices for Feather-derived icons."))
                            .push(text(include_str!("../assets/lucide/LICENSE")).size(12))
                            .push(text(include_str!("../LICENSE")).size(12));
                    } else {
                        body = body.push(text("Appearance").size(18));
                        for (index, appearance, label) in [(0, Appearance::Dark, "Dark"), (1, Appearance::Light, "Light")] {
                            let label = format!("{}{}{}", if self.settings_selected == index { "› " } else { "" }, label, if self.appearance == appearance { " · Selected" } else { "" });
                            body = body.push(button(text(label)).keyboard_enabled(false).selected(self.appearance == appearance).style(if self.appearance == appearance { selected_button } else { secondary_button }).on_press(LegacyButton::Appearance(appearance)));
                        }
                        if let Some(error) = &self.appearance_error { body = body.push(text(error).color(self.appearance.colors().error)); }
                        body = body.push(text("Timeline").size(18))
                            .push(iced::widget::checkbox(self.unlock_playhead_when_zooming)
                                .label(if self.settings_selected == 2 { "› Unlock playhead when zooming" } else { "Unlock playhead when zooming" })
                                .on_toggle(Message::UnlockOnZoom));
                        body = body.push(text("Keyboard shortcuts").size(18))
                            .push(text("Select Change and press a key or chord. Review it, then Enter or Apply confirms; Escape cancels. Text editing, Tab, Enter, Escape and operating-system chords are reserved. Ctrl/Command+S always saves the project. Popup Enter remains a save alternative.").size(14))
                            .push(button(if self.settings_selected == 3 { "› Restore all defaults" } else { "Restore all defaults" }).style(secondary_button).on_press(LegacyButton::RestoreAllShortcuts));
                        if let Some((command, proposed)) = &self.shortcut_capture {
                            body = body.push(text(format!("Changing {} · {}", self.command_name(*command), proposed.as_ref().map_or_else(|| "Press a key…".into(), Binding::label))))
                                .push(button("Apply").on_press_maybe(proposed.as_ref().filter(|binding| self.shortcuts.validate(*command, Some(binding)).is_ok()).map(|_| LegacyButton::ApplyShortcut)));
                        }
                        for (index, definition) in DEFINITIONS.iter().enumerate() {
                            let command = definition.command;
                            let selected = self.settings_selected;
                            let item = column![
                                text(format!("{} · {}", self.command_name(command), self.shortcuts.hint(command))).size(15),
                                text(definition.context.name()).size(13).color(self.appearance.colors().muted),
                                row![
                                    button(if selected == 4 + index * 3 { "› Change" } else { "Change" }).style(secondary_button).on_press(LegacyButton::CaptureShortcut(command)),
                                    button(if selected == 5 + index * 3 { "› Restore default" } else { "Restore default" }).style(secondary_button).on_press(LegacyButton::RestoreShortcut(command)),
                                    button(text(format!("{}{}", if selected == 6 + index * 3 { "› " } else { "" }, if command == Command::SaveProject { "Remove extra" } else { "Unbind" }))).style(secondary_button).on_press(LegacyButton::UnbindShortcut(command)),
                                ].spacing(8).wrap(),
                            ].spacing(5);
                            body = body.push(item);
                        }
                    }
                    let mut actions = row![].spacing(12);
                    if about { actions = actions.push(button(if self.about_copy_selected { "› Copy version info" } else { "Copy version info" }).on_press_maybe(self.copy_request.is_none().then_some(LegacyButton::CopyVersion))); }
                    actions = actions.push(button(if (about && !self.about_copy_selected) || (!about && self.settings_selected == 4 + DEFINITIONS.len() * 3) { "› Close    Esc" } else { "Close    Esc" }).on_press(LegacyButton::CloseHelp));
                    let mut panel = column![scrollable(body).id("app-dialog").width(Fill).height((size.height - 220.0).clamp(140.0, 560.0)), actions].spacing(16).width(Fill);
                    if let Some(feedback) = &self.app_feedback { panel = panel.push(text(feedback).size(14)); }
                    panel.into()
                };
                let panel = container(crate::gui_controls::focus_scope(content, false)).padding(if self.menu_open { 12 } else { 24 }).width(Fill).max_width(if self.menu_open { 420 } else { 720 }).style(container::rounded_box);
                if self.menu_open {
                    stack![
                        self.workspace(compact_workspace(size.width), size.height, size.width),
                        opaque(mouse_area(container(Space::new()).width(Fill).height(Fill).style(|theme| container::Style { background: Some(colors(theme).scrim.into()), ..Default::default() })).on_press(Message::CloseHelp)),
                        container(opaque(panel)).padding(16).center_x(Fill).center_y(Fill),
                    ].into()
                } else {
                    stack![self.workspace(compact_workspace(size.width), size.height, size.width), opaque(container(panel).center_x(Fill).center_y(Fill).style(|theme| container::Style { background: Some(colors(theme).scrim.into()), ..Default::default() }))].into()
                }
            }).into();
        }
        if self.help_open || self.shortcuts_open {
            return responsive(|size| {
                let content = if self.shortcuts_open {
                    self.shortcut_reference()
                } else {
                    column![
                        text("Help").size(24),
                        text("Tab or Up/Down to select · Enter to activate · Esc to close")
                            .size(14),
                        self.shortcut_control(
                            Command::Reference,
                            button(text(format!(
                                "{}Keyboard shortcuts    {}",
                                if self.help_close_selected { "" } else { "› " },
                                self.shortcuts.hint(Command::Reference)
                            )))
                            .on_press(LegacyButton::KeyboardShortcuts)
                        ),
                        button(if self.help_close_selected {
                            "› Close"
                        } else {
                            "Close"
                        })
                        .on_press(LegacyButton::CloseHelp)
                    ]
                    .spacing(16)
                    .into()
                };
                stack![
                    self.workspace(compact_workspace(size.width), size.height, size.width),
                    opaque(
                        container(
                            container(
                                scrollable(crate::gui_controls::focus_scope(content, false))
                                    .id("shortcut-reference")
                                    .width(Fill)
                                    .height(
                                        (size.height - 120.0).clamp(
                                            160.0,
                                            if self.help_open { 180.0 } else { 620.0 }
                                        )
                                    )
                            )
                            .padding(24)
                            .width(Fill)
                            .max_width(if self.help_open { 460 } else { 760 })
                            .style(container::rounded_box)
                        )
                        .center_x(Fill)
                        .center_y(Fill)
                        .style(|theme| container::Style {
                            background: Some(colors(theme).scrim.into()),
                            ..Default::default()
                        })
                    )
                ]
                .into()
            })
            .into();
        }
        if let Some(dialog) = &self.export_dialog {
            return stack![
                responsive(|size| self.workspace(
                    compact_workspace(size.width),
                    size.height,
                    size.width
                )),
                opaque(
                    container(
                        container(scrollable(self.export_settings_view(dialog)))
                            .padding(24)
                            .max_width(680)
                            .style(container::rounded_box)
                    )
                    .center_x(Fill)
                    .center_y(Fill)
                    .style(|theme| container::Style {
                        background: Some(colors(theme).scrim.into()),
                        ..Default::default()
                    })
                ),
            ]
            .into();
        }
        if self.import_dialog {
            return responsive(|size| {
                stack![
                    self.workspace(compact_workspace(size.width), size.height, size.width),
                    opaque(
                        container(
                            self.import_settings_view((size.height - 48.0).clamp(160.0, 600.0))
                        )
                        .padding(24)
                        .center_x(Fill)
                        .center_y(Fill)
                        .style(|theme| container::Style {
                            background: Some(colors(theme).scrim.into()),
                            ..Default::default()
                        })
                    )
                ]
                .into()
            })
            .into();
        }
        responsive(|size| {
            let count = self
                .project
                .as_ref()
                .map_or(0, |project| project.sources.len());
            self.workspace(
                compact_workspace(size.width) && count <= 3,
                size.height,
                size.width,
            )
        })
        .into()
    }

    fn import_settings_view(&self, height: f32) -> Element<'_, Message> {
        let mut choices = column![text(
            "Choose the microphone channels to edit. Their original timing stays aligned."
        )]
        .spacing(12);
        choices = choices.push(
            text(format!(
                "{} selected · {} accepted or reserved tracks · maximum 8",
                self.import_choices.iter().filter(|c| c.selected).count(),
                self.reserved_track_count()
            ))
            .size(14),
        );
        for (index, choice) in self.import_choices.iter().enumerate() {
            choices = choices.push(
                column![
                    iced::widget::checkbox(choice.selected)
                        .label(choice.source.channel.map_or_else(
                            || "Mono".into(),
                            |channel| format!("Channel {}", channel + 1)
                        ))
                        .on_toggle(move |selected| Message::ImportChannel(index, selected)),
                    text_input("Track name", choice.source.name.as_deref().unwrap_or(""))
                        .on_input(move |name| Message::ImportChannelName(index, name))
                        .width(Fill),
                    text(
                        choice
                            .source
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    )
                    .size(14),
                ]
                .spacing(12),
            );
        }
        if self.import_choices.is_empty() && self.sources_busy() {
            choices = choices.push(text(
                self.source_loading
                    .as_ref()
                    .map_or("Inspecting source tracks…", |loading| loading.stage),
            ));
            for (index, path) in self.loading_paths().iter().enumerate() {
                choices = choices.push(text(format!("Track {}: {}", index + 1, path.display())));
                if let Some(loading) = &self.source_loading
                    && let Some(cache) = loading.waveforms.get(index)
                {
                    choices = choices.push(
                        canvas(
                            TimelineCanvas::waveform(
                                cache,
                                &loading.timeline,
                                Message::Timeline,
                                None,
                                false,
                            )
                            .with_display(self.waveform_display)
                            .with_revision(self.drawing_revision),
                        )
                        .width(Fill)
                        .height(56),
                    );
                }
            }
        }
        for message in &self.messages {
            choices = choices.push(text(message));
        }
        let committing = self
            .source_loading
            .as_ref()
            .is_some_and(|loading| loading.committing);
        let mut actions = row![].spacing(12);
        if !self.import_choices.is_empty() {
            actions = actions.push(
                button("Import channels")
                    .style(primary_button)
                    .on_press(LegacyButton::ConfirmChannels),
            );
        }
        actions = actions.push(
            button(if committing { "Finishing…" } else { "Cancel" })
                .style(secondary_button)
                .on_press_maybe((!committing).then_some(LegacyButton::CancelChannels)),
        );
        container(
            column![
                text("Import tracks").size(24),
                scrollable(choices).height(Fill),
                actions
            ]
            .spacing(12),
        )
        .padding(24)
        .width(Fill)
        .max_width(680)
        .height(height)
        .style(container::rounded_box)
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

    fn import_display_rows(&self) -> Vec<ImportDisplayRow> {
        let mut rows = vec![];
        if let Some(project) = &self.project {
            for (index, source) in project.sources.iter().enumerate() {
                if !self
                    .import_jobs
                    .iter()
                    .any(|job| job.track_ids.contains(&source.track_id))
                {
                    rows.push((Some(index), None));
                }
            }
        }
        for (job_index, job) in self.import_jobs.iter().enumerate() {
            for channel in 0..job.sources.len() {
                if job.accepted {
                    if let Some(index) = self.project.as_ref().and_then(|p| {
                        p.sources
                            .iter()
                            .position(|s| s.track_id == job.track_ids[channel])
                    }) {
                        rows.push((Some(index), None));
                    }
                } else {
                    rows.push((None, Some((job_index, channel))));
                }
            }
        }
        rows
    }

    fn export_source_order(&self) -> Vec<usize> {
        self.import_display_rows()
            .into_iter()
            .filter_map(|(accepted, _)| accepted)
            .collect()
    }

    fn workspace_status(&self) -> String {
        if self.sources_busy() {
            let queued = self
                .import_jobs
                .iter()
                .filter(|job| !job.accepted && job.error.is_none() && job.events.is_none())
                .count();
            let stage = self
                .import_jobs
                .iter()
                .find(|job| job.events.is_some())
                .map_or("Inspecting source metadata", |job| job.status.as_str());
            return format!(
                "{} accepted · {} pending · {queued} queued · {stage}",
                self.project.as_ref().map_or(0, |p| p.sources.len()),
                self.import_jobs
                    .iter()
                    .filter(|j| !j.accepted)
                    .map(|j| j.sources.len())
                    .sum::<usize>()
            );
        }
        let Some(project) = &self.project else {
            return "Import tracks to begin".into();
        };
        if self.preview_processing.is_some() {
            return "Analyzing".into();
        }
        let (missing, room) = self.missing_references();
        if !missing.is_empty() || !room {
            return "Add speaker and room-tone examples".into();
        }
        if let Some(result) = &self.supervised_result {
            if !result.snapshot.matches(project) {
                return "Analysis needs updating".into();
            }
            return String::new();
        }
        "Examples ready · run Analyze".into()
    }

    fn next_action(&self) -> &'static str {
        let Some(project) = &self.project else {
            return "Open a project or import speaker tracks, including multichannel WAVs.";
        };
        if self.preview_processing.is_some() {
            return "Analyzing. Original playback remains available.";
        }
        if !project.pending_source_reviews().is_empty()
            || project.sources.iter().any(|source| !source.available)
        {
            return "Review source files below before Analyze or Export.";
        }
        let (missing, room) = self.missing_references();
        if missing.is_empty() && !room {
            return "Add a room-tone example.";
        }
        if !missing.is_empty() {
            return "Add a clear speech example for each speaker.";
        }
        if project.require_alignment().is_err() {
            return "Confirm synchronized recording starts below before Analyze.";
        }
        if self.reference_stale || self.supervised_result.is_none() {
            return "Run Analyze to apply your saved examples.";
        }
        "Audition Processed or Removed, then export."
    }

    fn analysis_is_next_action(&self) -> bool {
        let (missing, room) = self.missing_references();
        self.preview_processing.is_none()
            && !self.sources_busy()
            && (self.reference_stale || self.supervised_result.is_none())
            && missing.is_empty()
            && room
            && self.project.as_ref().is_some_and(|project| {
                project.require_alignment().is_ok()
                    && project.pending_source_reviews().is_empty()
                    && project.sources.iter().all(|source| source.available)
            })
    }

    fn calibration_controls(&self) -> Element<'_, Message> {
        let Some(project) = &self.project else {
            return column![].into();
        };
        let mut controls = column![
            row![
                inspector_heading("Speaker examples"),
                Space::new().width(Fill),
                text(format!("{} saved", project.reference_labels().len()))
                    .size(13)
                    .color(self.appearance.colors().muted),
            ]
            .align_y(iced::Center),
        ]
        .spacing(8);
        controls = controls.push(
            row![
                self.shortcut_control(
                    Command::UndoLabels,
                    button(text("Undo").size(14))
                        .padding([7, 10])
                        .width(Fill)
                        .style(inspector_button)
                        .on_press(LegacyButton::UndoReferences)
                ),
                self.shortcut_control(
                    Command::RedoLabels,
                    button(text("Redo").size(14))
                        .padding([7, 10])
                        .width(Fill)
                        .style(inspector_button)
                        .on_press(LegacyButton::RedoReferences)
                ),
                button(text("Analyze").size(14))
                    .padding([7, 10])
                    .width(Fill)
                    .style(if self.analysis_is_next_action() {
                        primary_button
                    } else {
                        inspector_button
                    })
                    .on_press(LegacyButton::AnalyzeReferences),
            ]
            .spacing(6)
            .width(Fill),
        );
        let (mut missing, room) = self.missing_references();
        if !room {
            missing.push("room tone".into());
        }
        controls = controls.push(
            text(if missing.is_empty() {
                "Speech and room-tone examples are ready.".into()
            } else {
                format!("Examples needed: {}", missing.join(", "))
            })
            .size(14),
        );
        controls = controls.push(
            button(if self.calibration_timing {
                "Hide examples ▾"
            } else {
                "Show examples ▸"
            })
            .style(quiet_button)
            .on_press(LegacyButton::CalibrationTiming(!self.calibration_timing)),
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
            controls = controls.push(
                text(format!("Example not saved: {error}")).color(self.appearance.colors().error),
            );
        }
        let mut examples = column![].spacing(8);
        if self.calibration_timing {
            examples = examples.push(
                text("Drag an example's ends to resize. Right-click to change speakers or delete.")
                    .size(14),
            );
            if project.reference_labels().is_empty() {
                examples = examples.push(
                    text("No examples yet. Drag across a waveform and choose who is talking.")
                        .size(14),
                );
            }
            for label in project.reference_labels() {
                examples = examples.push(
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
                            .size(14),
                            row![
                                button(text("Play").size(14))
                                    .style(button::text)
                                    .on_press(LegacyButton::PlayExample(label.id)),
                                button(text("Show on timeline").size(14))
                                    .style(button::text)
                                    .on_press(LegacyButton::ShowExample(label.id)),
                                button(text("Delete").size(13))
                                    .style(button::text)
                                    .on_press(LegacyButton::RemoveReference(label.id)),
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
        controls = controls.push(crate::gui_disclosure::reveal(
            container(scrollable(examples).id("speaker-examples")).max_height(220),
            self.calibration_timing,
        ));
        controls.into()
    }

    fn gain_controls(&self) -> Element<'_, Message> {
        let mut controls = column![
            iced::widget::checkbox(self.gain_tool)
                .label("Edit gain points")
                .on_toggle(Message::GainTool),
        ]
        .spacing(8);
        if let Some(label) = self.selected_example.and_then(|id| {
            self.project
                .as_ref()?
                .reference_labels()
                .iter()
                .find(|label| label.id == id)
        }) {
            controls = controls.push(
                text(format!("Example · {}", example_speakers(label)))
                    .size(13)
                    .color(self.appearance.colors().muted),
            );
        }
        let target = self.editing_track();
        if let Some(source) = target.and_then(|track| self.project.as_ref()?.sources.get(track)) {
            let range = if self.gain_tool && self.selected_gain_point.is_some() {
                String::new()
            } else {
                self.timeline
                    .selection()
                    .map_or_else(String::new, |(start, end)| {
                        format!(" · {}–{}", example_time(start), example_time(end))
                    })
            };
            controls = controls.push(
                text(format!(
                    "{} · Speaker {}{range}",
                    if self.gain_tool && self.selected_gain_point.is_some() {
                        "Gain point"
                    } else {
                        "Selected range"
                    },
                    source.speaker_id.0
                ))
                .size(13)
                .color(self.appearance.colors().muted),
            );
        }
        if self.gain_tool {
            controls = controls
                .push(text(self.gain_key_hint()).size(14))
                .push(
                    row![
                        self.shortcut_control(
                            Command::UndoGain,
                            button("Undo gain")
                                .style(secondary_button)
                                .on_press(LegacyButton::UndoGain)
                        ),
                        self.shortcut_control(
                            Command::RedoGain,
                            button("Redo gain")
                                .style(secondary_button)
                                .on_press(LegacyButton::RedoGain)
                        ),
                    ]
                    .spacing(6),
                )
                .push(
                    button("Add point at cursor")
                        .style(secondary_button)
                        .on_press(LegacyButton::InsertGainPoint),
                )
                .push(
                    self.shortcut_control(
                        Command::DeleteGain,
                        button("Remove selected override")
                            .style(secondary_button)
                            .on_press(LegacyButton::DeleteGain),
                    ),
                )
                .push(
                    iced::widget::checkbox(self.replace_gain_constraints)
                        .label("Replace overlapping overrides and protections")
                        .on_toggle(Message::ReplaceGainConstraints),
                );
            if self.can_protect_selection() {
                controls = controls.push(
                    button("Preserve selection on all tracks")
                        .style(secondary_button)
                        .on_press(LegacyButton::ProtectSelection),
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
            text(format!("Project autosaved to {}", project.path().display())).size(12),
            button("Copy project path").style(secondary_button).on_press(LegacyButton::CopyProjectPath),
            text("Original audio is read only. Keep its card or drive connected until processing finishes.").size(12),
            button("Recheck source files")
                .style(secondary_button)
                .on_press(LegacyButton::Validate),
        ]
        .spacing(8);
        if let Err(error) = project.require_alignment() {
            controls = controls
                .push(
                    text("Before Analyze: confirm the tracks share the same recording start.")
                        .size(14),
                )
                .push(
                    iced::widget::checkbox(project.supplied_alignment())
                        .label("Tracks are synchronized")
                        .on_toggle(Message::SuppliedAlignment),
                )
                .push(text(error.to_string()).size(14));
        }
        for (index, source) in project.sources.iter().enumerate() {
            let mut source_controls =
                column![text(format!("Speaker {} source", source.speaker_id.0)).size(14)]
                    .spacing(4);
            if !source.available {
                source_controls = source_controls.push(
                    button("Relink source")
                        .style(primary_button)
                        .on_press(LegacyButton::Relink(index)),
                );
            }
            if project.pending_source_reviews().contains(&source.track_id) {
                source_controls = source_controls.push(
                    button("Accept source changes")
                        .style(primary_button)
                        .on_press(LegacyButton::RebindSource(index)),
                );
            }
            if !source.available || project.pending_source_reviews().contains(&source.track_id) {
                controls = controls.push(source_controls);
            }
        }
        controls.into()
    }

    fn export_controls(&self) -> Element<'_, Message> {
        let mut controls = column![].spacing(8);
        if let Some(processing) = &self.processing {
            controls = controls
                .push(text(format!(
                    "Preparing WAV files: {}%",
                    processing.progress.load(Ordering::Relaxed)
                )))
                .push(
                    button("Cancel WAV preparation")
                        .style(secondary_button)
                        .on_press(LegacyButton::Cancel),
                );
        } else {
            controls = controls.push(
                button(text("Export WAV files").size(16).center())
                    .height(40)
                    .padding([10, 12])
                    .width(Fill)
                    .style(if self.can_prepare_export() {
                        primary_button
                    } else {
                        inspector_button
                    })
                    .on_press_maybe(self.can_prepare_export().then_some(LegacyButton::Export)),
            );
        }
        if self.exporting.is_some() {
            controls = controls.push(text("Checking and saving WAV files…")).push(
                button("Cancel saving")
                    .style(secondary_button)
                    .on_press(LegacyButton::Cancel),
            );
        }
        if let Some(error) = &self.export_error {
            controls = controls.push(
                container(scrollable(
                    text(format!("Export blocked: {error}"))
                        .size(14)
                        .color(self.appearance.colors().error),
                ))
                .max_height(100),
            );
        }
        controls.into()
    }

    fn export_settings_view<'a>(&'a self, dialog: &'a ExportDialog) -> Element<'a, Message> {
        let project = self.project.as_ref().unwrap();
        let mut content = column![
            text("Export WAV files").size(24),
            text(format!(
                "{} tracks / channels · original sample rates",
                project.sources.len()
            ))
        ]
        .spacing(12);
        for (i, index) in self.export_source_order().into_iter().enumerate() {
            let source = &project.sources[index];
            content = content.push(
                text(format!(
                    "{}. {} · {} Hz · {} frames",
                    i + 1,
                    source.display_name(),
                    source.metadata.grid.sample_rate,
                    source.metadata.grid.frame_count
                ))
                .size(14),
            );
        }
        let compatible_grid = project.sources.first().is_some_and(|first| {
            project.sources.iter().all(|s| {
                s.metadata.grid == first.metadata.grid
                    && s.metadata.timing_origin == first.metadata.timing_origin
            })
        });
        content = content
            .push(text("Output layout").size(18))
            .push(iced::widget::radio(
                "Separate mono WAV files",
                crate::ExportLayout::SeparateMono,
                Some(dialog.settings.layout),
                Message::ExportLayout,
            ));
        if compatible_grid {
            content = content.push(iced::widget::radio(
                "One WAV with separate channels in the order above",
                crate::ExportLayout::Multichannel,
                Some(dialog.settings.layout),
                Message::ExportLayout,
            ));
        } else {
            content = content.push(text("Multichannel unavailable: sample rates, frame counts and timing must match. Separate WAVs keep native grids."));
        }
        content = content.push(text("Encoding").size(18));
        for encoding in [
            crate::ExportEncoding::PreserveSource,
            crate::ExportEncoding::Pcm24,
            crate::ExportEncoding::Float32,
        ] {
            let estimate = crate::ExportSettings {
                encoding,
                ..dialog.settings
            }
            .estimated_audio_bytes(project)
            .map(|bytes| {
                format!(
                    "{:.2} MB estimated audio payload",
                    bytes as f64 / 1_000_000.0
                )
            })
            .unwrap_or_else(|e| e.to_string());
            content = content.push(iced::widget::radio(
                format!("{encoding} · {estimate}"),
                encoding,
                Some(dialog.settings.encoding),
                Message::ExportEncoding,
            ));
        }
        content = content.push(text("Non-WAV and float64 sources use float32 with Preserve source encoding. PCM24 payload is 25% smaller than float32. Integer output uses final dither; exact silence stays silent.").size(14));
        let blocker = dialog.settings.validate_layout(project).err();
        if let Some(error) = &blocker {
            content = content.push(text(error.to_string()));
        }
        content = content.push(
            button("Choose destination folder")
                .style(secondary_button)
                .on_press(LegacyButton::ExportDestination),
        );
        content = content.push(
            text(dialog.destination.as_ref().map_or_else(
                || {
                    "Choose a location. A new CleanSlate export folder will be created inside it."
                        .into()
                },
                |p| p.display().to_string(),
            ))
            .size(14),
        );
        for message in &self.messages {
            content = content.push(text(message).size(14));
        }
        content
            .push(
                row![
                    button("Cancel")
                        .style(secondary_button)
                        .on_press(LegacyButton::CancelExportDialog),
                    button("Export files").style(primary_button).on_press_maybe(
                        (blocker.is_none()
                            && dialog.destination.is_some()
                            && self.can_prepare_export())
                        .then_some(LegacyButton::SubmitExport)
                    )
                ]
                .spacing(12),
            )
            .into()
    }

    fn workspace(&self, compact: bool, _height: f32, width: f32) -> Element<'_, Message> {
        let ownership_stale = self.reference_stale;
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
            _ => "Speaker activity · run Analyze to see estimates".into(),
        };
        let header = row![
            column![
                text(
                    self.project
                        .as_ref()
                        .and_then(|project| project.path().file_stem())
                        .map_or_else(
                            || "Untitled project".into(),
                            |name| name.to_string_lossy().into_owned()
                        )
                )
                .size(20)
            ]
            .push(
                text(self.workspace_status())
                    .size(13)
                    .color(self.appearance.colors().muted)
            )
            .spacing(2)
            .width(190),
            Space::new().width(Fill),
            button("Import tracks")
                .style(if self.project.is_none() {
                    primary_button
                } else {
                    quiet_button
                })
                .on_press(LegacyButton::ImportStems),
            self.shortcut_control(
                Command::Menu,
                button("Menu")
                    .style(secondary_button)
                    .on_press(LegacyButton::Menu)
            ),
            button("New project").style(quiet_button).on_press_maybe(
                self.can_start_new_project()
                    .then_some(LegacyButton::NewProject)
            ),
            button("Open project").style(quiet_button).on_press_maybe(
                (!self.sources_busy() && self.processing.is_none())
                    .then_some(LegacyButton::OpenProject)
            ),
            self.shortcut_control(
                Command::SaveProject,
                button("Save project").style(quiet_button).on_press_maybe(
                    (self.project.is_some() && self.source_loading.is_none())
                        .then_some(LegacyButton::SaveProject)
                )
            ),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let mut timeline_content = column![].spacing(6).height(Fill);
        if self.import_jobs.iter().any(|job| !job.accepted) {
            timeline_content = timeline_content.push(row![
                text("Pending audio is excluded from Save, Analyze and Export. Ready tracks remain editable.").size(14),
                button("Cancel all pending").style(secondary_button).on_press(LegacyButton::CancelPendingImports)
            ].spacing(12));
        }
        if let Some(error) = &self.appearance_error {
            timeline_content =
                timeline_content.push(text(error).size(14).color(self.appearance.colors().error));
        }
        if self.project.is_none() && self.import_jobs.is_empty() {
            timeline_content = timeline_content.push(
                container(
                    column![
                        text("Start with your speaker tracks").size(22),
                        text(
                            "Import synchronized speaker tracks or a multichannel WAV, or drop them anywhere in this window."
                        ),
                        button("Import tracks").style(primary_button).on_press(LegacyButton::ImportStems),
                    ]
                    .spacing(12)
                    .align_x(iced::Center),
                )
                .center_x(Fill)
                .center_y(Fill)
                .height(Fill),
            );
        }
        if self.project.is_some() || !self.import_jobs.is_empty() {
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
            for (accepted, pending) in self.import_display_rows() {
                if let Some((job_index, channel)) = pending {
                    let job = &self.import_jobs[job_index];
                    let id = job.id;
                    let mut actions = row![
                        text(job.sources[channel].display_name()).size(14),
                        Space::new().width(Fill),
                        text(&job.status).size(14)
                    ]
                    .spacing(8);
                    if job.error.is_some() {
                        actions = actions.push(
                            button("Retry")
                                .style(primary_button)
                                .on_press(LegacyButton::RetryImport(id)),
                        );
                    }
                    actions = actions.push(
                        button(if job.error.is_some() {
                            "Remove"
                        } else {
                            "Cancel / Remove"
                        })
                        .style(secondary_button)
                        .on_press(LegacyButton::RemoveImport(id)),
                    );
                    let mut content = column![actions].spacing(6);
                    if let Some(error) = &job.error {
                        content = content
                            .push(text(error).size(14).color(self.appearance.colors().error));
                    }
                    content = content.push(
                        canvas(
                            TimelineCanvas::waveform(
                                &job.waveforms[channel],
                                &self.timeline,
                                Message::Timeline,
                                None,
                                false,
                            )
                            .with_display(self.waveform_display)
                            .with_revision(self.drawing_revision),
                        )
                        .width(Fill)
                        .height(Fill),
                    );
                    timeline_content = timeline_content.push(
                        container(content)
                            .padding(4)
                            .height(Length::FillPortion(1))
                            .width(Fill)
                            .style(editor_panel),
                    );
                    continue;
                }
                let project = self.project.as_ref().unwrap();
                let index = accepted.unwrap();
                let source = &project.sources[index];
                let filename = source.display_name();
                let mut short_name = filename.chars().take(24).collect::<String>();
                if filename.chars().count() > 24 {
                    short_name.push('…');
                }
                let title: Element<'_, Message> = if let Some((editing, name)) =
                    &self.track_name_draft
                    && *editing == index
                {
                    let mut editor = column![
                        row![
                            text(format!("Speaker {}", source.speaker_id.0)).size(14),
                            text_input("Track name", name)
                                .id("track-name")
                                .size(14)
                                .padding([4, 6])
                                .on_input(Message::TrackName)
                                .on_submit(Message::SaveTrackName)
                                .width(280),
                            button(text("Save").size(14))
                                .style(primary_button)
                                .on_press(LegacyButton::SaveTrackName),
                            button(text("Cancel").size(14))
                                .style(secondary_button)
                                .on_press(LegacyButton::CancelTrackName),
                        ]
                        .spacing(8)
                        .align_y(iced::Center)
                    ]
                    .spacing(4);
                    if !self.messages.is_empty() {
                        editor = editor.push(
                            text(self.messages.join("\n"))
                                .size(13)
                                .color(self.appearance.colors().error),
                        );
                    }
                    editor.into()
                } else {
                    iced::widget::tooltip(
                        button(
                            text(format!("Speaker {} · {short_name}", source.speaker_id.0))
                                .size(14)
                                .color(self.appearance.colors().text),
                        )
                        .style(button::text)
                        .padding([3, 6])
                        .on_press(LegacyButton::EditTrackName(index)),
                        text(format!("Click to rename · {}", source.path.display())).size(14),
                        iced::widget::tooltip::Position::Bottom,
                    )
                    .into()
                };
                let mut header = row![
                    button(
                        text(if self.solo_tracks & (1u64 << index) != 0 {
                            "Solo · on"
                        } else {
                            "Solo"
                        })
                        .size(14)
                    )
                    .style(if self.solo_tracks & (1u64 << index) != 0 {
                        selected_button
                    } else {
                        secondary_button
                    })
                    .selected(self.solo_tracks & (1u64 << index) != 0)
                    .on_press(LegacyButton::SoloTrack(index)),
                    title,
                    text(if self.editing_track() == Some(index) {
                        "Editing"
                    } else {
                        ""
                    })
                    .size(12)
                    .color(self.appearance.colors().muted),
                    Space::new().width(Fill),
                    button(text("Preserve").size(14))
                        .style(secondary_button)
                        .on_press_maybe(
                            self.timeline
                                .selection()
                                .map(|_| LegacyButton::GainAction(index, true))
                        ),
                    button(text("Attenuate").size(14))
                        .style(secondary_button)
                        .on_press_maybe(
                            self.timeline
                                .selection()
                                .map(|_| LegacyButton::GainAction(index, false))
                        ),
                ]
                .spacing(8)
                .align_y(iced::Center);
                if !source.available {
                    header = header.push(text("Missing source").size(14));
                }
                if project.pending_source_reviews().contains(&source.track_id) {
                    header = header.push(text("Review source").size(14));
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
                            ("Active", self.appearance.colors().success)
                        }
                        Some(crate::TemporalTrackState::Absent) => {
                            ("Inactive", self.appearance.colors().muted)
                        }
                        _ => ("Uncertain", self.appearance.colors().warning),
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
                            text(meter_label.clone()).size(14),
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
                            .with_display(self.waveform_display)
                            .with_editing_target(self.editing_track() == Some(index))
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
                track_content = track_content.push(waveform);
                if self.gain_tool {
                    track_content = track_content.push(
                        canvas(crate::gain_editor::GainCanvas {
                            revision: self.drawing_revision,
                            project,
                            constraints: self
                                .gain_drag
                                .as_ref()
                                .map_or(&self.gain_constraints, |drag| &drag.draft().constraints),
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
                            editing: true,
                            selected_point: self.selected_gain_point.and_then(
                                |(track, edit, point)| (track == index).then_some((edit, point)),
                            ),
                            on_event: Box::new(move |event| Message::GainEvent(index, event)),
                        })
                        .width(Fill)
                        .height(20),
                    );
                }
                timeline_content = timeline_content.push(
                    container(track_content)
                        .padding(4)
                        .height(Length::FillPortion(1))
                        .width(Fill)
                        .style(move |theme| {
                            let mut style = editor_panel(theme);
                            if self.editing_track() == Some(index) {
                                style.border.color = colors(theme).focus;
                                style.border.width = 2.0;
                            }
                            style
                        }),
                );
            }
        }

        let tracks: Element<'_, Message> = if self.project.is_none() {
            scrollable(timeline_content).height(Fill).into()
        } else {
            timeline_content.into()
        };
        let locked = self.playback_scroll == PlaybackScroll::Locked;
        let catch_up = self.playback_scroll == PlaybackScroll::Page;
        let meter_control = self.action_control(
            Command::Meters,
            button(toolbar_label(
                ToolbarIcon::Meters {
                    at_playhead: self.meters_at_playhead,
                },
                "Meters",
            ))
            .padding([6, 8])
            .height(34)
            .style(|theme, status| flat_button(theme, status, false))
            .hover_outline(false),
        );
        let mut waveform_modes = row![].spacing(4).align_y(iced::Center);
        for mode in [
            waveform::WaveformDisplay::Original,
            waveform::WaveformDisplay::Processed,
            waveform::WaveformDisplay::Both,
        ] {
            let selected = self.waveform_display == mode;
            let control = button(container(ToolbarIcon::Waveform(mode).view(None)).center(Fill))
                .width(32)
                .height(30)
                .padding(5)
                .style(move |theme, status| flat_button(theme, status, selected))
                .hover_outline(false)
                .selected(selected)
                .on_press(LegacyButton::WaveformDisplay(mode));
            waveform_modes = waveform_modes.push(
                iced::widget::tooltip(
                    control,
                    container(text(format!("{mode} waveform")).size(13))
                        .padding(8)
                        .style(editor_panel),
                    iced::widget::tooltip::Position::Bottom,
                )
                .delay(Duration::from_millis(600))
                .snap_within_viewport(true),
            );
        }
        let timeline = column![
            row![
                text("Timeline").size(16).font(iced::Font { weight: iced::font::Weight::Semibold, ..iced::Font::with_name("Segoe UI") }),
                Space::new().width(Fill),
                self.action_control(Command::CatchUp,
                    button(container(ToolbarIcon::CatchUp.view(None)).center(Fill))
                        .padding(6).width(34).height(34)
                        .style(move |theme, status| flat_button(theme, status, catch_up))
                        .hover_outline(false).selected(catch_up)),
                self.action_control(Command::Lock,
                    button(toolbar_label(ToolbarIcon::Lock { locked }, "Lock"))
                        .padding([6, 8]).height(34)
                        .style(move |theme, status| flat_button(theme, status, locked))
                        .hover_outline(false).selected(locked)),
                self.action_control(Command::Center,
                    button(toolbar_label(ToolbarIcon::Center, "Center"))
                        .padding([6, 8]).height(34)
                        .style(|theme, status| flat_button(theme, status, false))
                        .hover_outline(false)),
                meter_control,
                Space::new().width(12),
                waveform_modes,
            ].spacing(8).align_y(iced::Center),
            iced::widget::tooltip(
                text("Drag to mark a speaker example · red shading shows reduced volume")
                    .size(14).color(self.appearance.colors().muted),
                container(text("Click to seek. Drag to label speech or room tone.\nWheel to zoom. Ctrl-wheel or middle-drag to pan.\nWhile locked, release a middle-drag to set the new lock position.").size(14))
                    .padding(12).style(editor_panel),
                iced::widget::tooltip::Position::Bottom,
            ),
            tracks,
        ]
        .spacing(10)
        .width(Fill)
        .height(Fill);
        let setup_required = self.project.as_ref().is_some_and(|project| {
            project.require_alignment().is_err()
                || !project.pending_source_reviews().is_empty()
                || project.sources.iter().any(|source| !source.available)
        });
        let preview_state: Element<'_, Message> = if let Some(processing) = &self.preview_processing
        {
            let stage = processing
                .feedback
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            column![
                row![
                    inspector_heading("Analyzing"),
                    Space::new().width(Fill),
                    button(text("Cancel analysis").size(14))
                        .style(inspector_button)
                        .on_press(LegacyButton::CancelAnalysis),
                ]
                .align_y(iced::Center),
                text(format!(
                    "Elapsed {:02}:{:02}",
                    processing.started_at.elapsed().as_secs() / 60,
                    processing.started_at.elapsed().as_secs() % 60
                ))
                .size(14),
                text(stage).size(14),
                text(if self.preview.is_some() {
                    "Previous completed preview remains available"
                } else {
                    "No completed preview yet"
                })
                .size(13),
            ]
            .spacing(6)
            .into()
        } else if let Some(error) = &self.preview_error {
            text(format!("Preview failed: {error}"))
                .size(14)
                .color(self.appearance.colors().error)
                .into()
        } else if setup_required {
            inspector_heading("Sources need review")
                .color(self.appearance.colors().warning)
                .into()
        } else if self.reference_stale {
            text("Analysis needs updating")
                .color(self.appearance.colors().warning)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..iced::Font::with_name("Segoe UI")
                })
                .into()
        } else if self.preview.is_some() {
            row![
                container(Space::new().width(5).height(5)).style(|theme| container::Style {
                    background: Some(colors(theme).success.into()),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                text("Ready").size(14).color(self.appearance.colors().muted),
                Space::new().width(Fill),
                button(
                    text(if self.analysis_details_open {
                        "Details ▾"
                    } else {
                        "Details ▸"
                    })
                    .size(13)
                )
                .style(quiet_button)
                .on_press_maybe(
                    self.analysis_details
                        .as_ref()
                        .map(|_| LegacyButton::AnalysisDetails(!self.analysis_details_open))
                ),
            ]
            .spacing(8)
            .align_y(iced::Center)
            .into()
        } else {
            text("Ready to add examples").into()
        };
        let parameters = &self.preview_parameters;
        let mut setup = column![
            button(if setup_required {
                "Sources · needs review ▾"
            } else if self.setup_details {
                "Sources ▾"
            } else {
                "Sources ▸"
            })
            .style(quiet_button)
            .on_press(LegacyButton::SetupDetails(!self.setup_details))
        ]
        .spacing(12);
        setup = setup.push(crate::gui_disclosure::reveal(
            self.source_controls(),
            self.setup_details || setup_required,
        ));
        let attenuation = row![
            self.sound_dial(SoundParameter::Attenuation, 96.0),
            column![
                inspector_heading("Attenuation"),
                self.sound_value(SoundParameter::Attenuation, ""),
                row![
                    button(text(if parameters.mute { "Mute: on" } else { "Mute" }).size(13))
                        .style(if parameters.mute {
                            selected_button
                        } else {
                            inspector_button
                        })
                        .selected(parameters.mute)
                        .on_press(LegacyButton::Mute),
                    button(text("Reset").size(13))
                        .style(inspector_button)
                        .on_press(LegacyButton::ResetSound(SoundParameter::Attenuation)),
                ]
                .spacing(6),
            ]
            .spacing(6)
            .width(Fill),
        ]
        .spacing(14)
        .align_y(iced::Center);
        let transitions = column![
            inspector_heading("Transitions and gaps"),
            self.timing_control(SoundParameter::Open, "Fade to full volume"),
            self.timing_control(SoundParameter::Close, "Fade to reduced volume"),
            self.timing_control(SoundParameter::Gap, "Ignore gaps shorter than"),
        ]
        .spacing(12);
        let mut inspector_header = column![preview_state].spacing(6);
        if self.preview.is_none()
            || self.reference_stale
            || setup_required
            || self.preview_error.is_some()
            || self.preview_processing.is_some()
        {
            inspector_header = inspector_header.push(
                text(self.next_action())
                    .size(14)
                    .color(self.appearance.colors().muted),
            );
        }
        if self.preview.is_some()
            && !self.reference_stale
            && self.preview_processing.is_none()
            && let Some(details) = &self.analysis_details
        {
            let mut timings = column![].spacing(4);
            for (index, (label, seconds)) in details.iter().enumerate() {
                if index == 1 {
                    timings = timings.push(iced::widget::rule::horizontal(1));
                }
                timings = timings.push(
                    row![
                        text(*label).size(13).width(Fill),
                        text(format!("{seconds:.1} s")).size(13),
                    ]
                    .spacing(12),
                );
            }
            inspector_header = inspector_header.push(crate::gui_disclosure::reveal(
                timings,
                self.analysis_details_open,
            ));
        }
        let mut inspector_status = column![
            container(scrollable(inspector_header))
                .max_height(if self.analysis_details_open {
                    230
                } else if compact {
                    110
                } else {
                    150
                })
                .width(Fill)
        ]
        .spacing(6);
        if !self.messages.is_empty() {
            let updates = self
                .messages
                .iter()
                .rev()
                .fold(column![].spacing(6), |updates, message| {
                    updates.push(text(message).size(13))
                });
            inspector_status = inspector_status.push(
                container(scrollable(updates).id("inspector-feedback"))
                    .max_height(if compact { 52 } else { 72 })
                    .width(Fill),
            );
        }
        let body = column![
            self.calibration_controls(),
            iced::widget::rule::horizontal(1),
            attenuation,
            transitions,
            self.gain_controls(),
            iced::widget::rule::horizontal(1),
            setup,
        ]
        .spacing(18)
        .padding(iced::Padding {
            left: 2.0,
            right: 14.0,
            ..iced::Padding::default()
        });
        let inspector_content: Element<'_, Message> = if compact {
            column![
                row![
                    inspector_status.width(Fill),
                    container(self.export_controls()).width(220),
                ]
                .spacing(12)
                .align_y(iced::Top),
                iced::widget::rule::horizontal(1),
                scrollable(body).height(Fill),
            ]
            .spacing(10)
            .height(240)
            .into()
        } else {
            column![
                inspector_status,
                iced::widget::rule::horizontal(1),
                scrollable(body).height(Fill),
                iced::widget::rule::horizontal(1),
                self.export_controls(),
            ]
            .spacing(12)
            .height(Fill)
            .into()
        };
        let inspector = container(inspector_content)
            .padding(16)
            .width(if compact { Fill } else { Length::Fixed(340.0) })
            .style(editor_panel);
        let inspector: Element<'_, Message> = if self.project.is_none() && self.sources_busy() {
            container(column![text("Loading tracks").size(20),
                text("Waveforms appear as tracks load. Once the source checks finish, add speaker examples and run Analyze."),
                  text("You can scroll the tracks and drop more audio files. Editing becomes available when the import is validated."),
              ].spacing(12)).padding(16).width(if compact { Fill } else { Length::Fixed(340.0) }).into()
        } else {
            inspector.into()
        };
        let can_audition = self.preview.is_some()
            || self.audition_mode == AuditionMode::Original
                && self
                    .project
                    .as_ref()
                    .is_some_and(|project| crate::original_envelopes(project).is_ok());
        let can_stop = self.audition.is_some()
            || self.audition_range.is_some()
            || self.audition_preparation.is_busy();
        let playback_icon = if self
            .reference_audition
            .as_ref()
            .is_some_and(|snapshot| !snapshot.paused)
        {
            ToolbarIcon::Pause
        } else {
            ToolbarIcon::Play
        };
        let mut timestamp = column![text(format!(
            "{:02}:{:06.3}",
            self.timeline.playhead() / TICKS_PER_SECOND / 60,
            (self.timeline.playhead() % (TICKS_PER_SECOND * 60)) as f64 / TICKS_PER_SECOND as f64
        ))];
        if let Some(status) = self.audition_status().or_else(|| {
            self.audition_preparation
                .is_busy()
                .then_some("Preparing playback…")
        }) {
            timestamp = timestamp.push(text(status).size(13));
        }
        let playback = row![
            self.shortcut_control(
                Command::Start,
                button(
                    container(ToolbarIcon::Previous.view(Some(self.appearance.colors().text)))
                        .center(Fill)
                )
                .width(40)
                .height(36)
                .style(secondary_button)
                .on_press(LegacyButton::Previous)
            ),
            self.shortcut_control(
                if self.reference_menu.is_some() {
                    Command::Loop
                } else {
                    Command::Play
                },
                button(
                    container(playback_icon.view(Some(if can_audition {
                        self.appearance.colors().action_text
                    } else {
                        self.appearance.colors().disabled
                    })))
                    .center(Fill)
                )
                .width(40)
                .height(36)
                .style(primary_button)
                .on_press_maybe(can_audition.then_some(LegacyButton::Audition))
            ),
            self.shortcut_control(
                Command::Play,
                button(
                    container(ToolbarIcon::Stop.view(Some(if can_stop {
                        self.appearance.colors().text
                    } else {
                        self.appearance.colors().disabled
                    })))
                    .center(Fill)
                )
                .width(40)
                .height(36)
                .style(secondary_button)
                .on_press_maybe(can_stop.then_some(LegacyButton::Stop))
            ),
        ]
        .spacing(8)
        .align_y(iced::Center);
        let mut listening_modes = row![].spacing(2).align_y(iced::Center);
        for (command, mode, label, message) in [
            (
                Command::Original,
                AuditionMode::Original,
                "Original",
                LegacyButton::Original,
            ),
            (
                Command::Processed,
                AuditionMode::Processed,
                "Processed",
                LegacyButton::Processed,
            ),
            (
                Command::Removed,
                AuditionMode::Removed,
                "Removed",
                LegacyButton::Removed,
            ),
        ] {
            let selected = self.audition_mode == mode;
            let enabled = mode == AuditionMode::Original || self.preview.is_some();
            listening_modes = listening_modes.push(
                self.shortcut_control(
                    command,
                    button(text(label).size(14).font(iced::Font {
                        weight: if selected {
                            iced::font::Weight::Semibold
                        } else {
                            iced::font::Weight::Normal
                        },
                        ..iced::Font::with_name("Segoe UI")
                    }))
                    .width(98)
                    .height(34)
                    .style(move |theme, status| flat_button(theme, status, selected))
                    .hover_outline(false)
                    .selected(selected)
                    .on_press_maybe(enabled.then_some(message)),
                ),
            );
        }
        let listening = row![
            listening_modes,
            text(format!("Monitor {:+.1} dB", self.monitor_db)),
            slider(-24.0..=6.0, self.monitor_db, Message::MonitorGain)
                .step(1.0_f32)
                .width(120),
        ]
        .spacing(8)
        .align_y(iced::Center);
        let transport: Element<'_, Message> = if width < 1400.0 {
            column![
                row![
                    container(timestamp).width(Fill),
                    playback,
                    Space::new().width(Fill)
                ]
                .align_y(iced::Center),
                container(listening).center_x(Fill),
            ]
            .spacing(10)
            .into()
        } else {
            row![
                container(timestamp).width(Fill),
                playback,
                container(listening).align_right(Fill)
            ]
            .spacing(16)
            .align_y(iced::Center)
            .into()
        };
        let editor: Element<'_, Message> = if compact {
            column![timeline, inspector].spacing(12).height(Fill).into()
        } else {
            row![timeline, inspector].spacing(12).height(Fill).into()
        };

        let ownership_monitor = row![
            text(ownership_status).size(16).width(Fill),
            text("Speaker estimates · bars do not show volume").size(13),
        ]
        .spacing(8)
        .align_y(iced::Center);
        let base: Element<'_, Message> =
            container(column![header, editor, ownership_monitor, transport].spacing(12))
                .padding(12)
                .width(Fill)
                .height(Fill)
                .into();
        let base = crate::gui_controls::focus_scope(base, self.workspace_input_active());
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
                        button("Remove manual gain adjustment")
                            .style(primary_button)
                            .on_press(LegacyButton::DeleteRegion(track, time)),
                        text("Restore automatic gain for this section.").size(14),
                        button("Cancel")
                            .style(secondary_button)
                            .on_press(LegacyButton::DismissReferenceMenu),
                    ]
                    .spacing(6),
                )
                .padding(10)
                .style(editor_panel);
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
                text("None selected = room tone").size(14),
                text(format!(
                    "{} Clear · {} Save · Enter also confirms · Esc Cancel",
                    self.shortcuts.hint(Command::ClearSpeakers),
                    self.shortcuts.hint(Command::SaveExample)
                ))
                .size(13),
                text(format!(
                    "{} Pause / resume · Stop ends audition",
                    self.shortcuts.hint(Command::Loop)
                ))
                .size(13)
            ]
            .spacing(6);
            if let Some(project) = &self.project {
                for (index, source) in project.sources.iter().enumerate() {
                    let speaker = source.speaker_id;
                    menu = menu.push(
                        row![
                            container(
                                text(self.shortcuts.hint(
                                    Command::speaker(index).expect("at most eight speakers")
                                ))
                                .size(13)
                            )
                            .padding(4)
                            .style(editor_panel),
                            self.shortcut_control(
                                Command::speaker(index).expect("at most eight speakers"),
                                iced::widget::checkbox(self.reference_speakers.contains(&speaker))
                                    .label(format!("Speaker {}", speaker.0))
                                    .on_toggle(move |active| Message::ReferenceSpeaker(
                                        speaker, active
                                    ))
                            )
                        ]
                        .spacing(8)
                        .align_y(iced::Center),
                    );
                }
                menu = menu.push(
                    row![
                        self.shortcut_control(
                            Command::SaveExample,
                            button(if self.selected_example.is_some() {
                                "Save changes"
                            } else {
                                "Save example"
                            })
                            .style(primary_button)
                            .on_press(LegacyButton::ContextReference(
                                self.reference_speakers.clone(),
                                false
                            ))
                        ),
                        button("Stop")
                            .style(secondary_button)
                            .on_press(LegacyButton::Stop),
                        button("Cancel")
                            .style(secondary_button)
                            .on_press(LegacyButton::DismissReferenceMenu)
                    ]
                    .spacing(8),
                );
                if let Some(id) = self.selected_example {
                    menu = menu.push(
                        text(
                            "Drag the left or right handle on the waveform to adjust this example.",
                        )
                        .size(14),
                    );
                    menu = menu.push(
                        button("Delete example")
                            .style(secondary_button)
                            .on_press(LegacyButton::RemoveReference(id)),
                    );
                }
            }
            if let Some(status) = self.audition_status() {
                menu = menu.push(text(status).size(14));
            } else if self.reference_audition.as_ref().is_some_and(|s| s.paused) {
                menu = menu.push(
                    text(format!(
                        "Audition paused · {} resumes",
                        self.shortcuts.hint(Command::Loop)
                    ))
                    .size(14),
                );
            } else if self.audition.is_none()
                && let Some(error) = self.messages.first()
            {
                menu = menu.push(text(error).size(14));
            }
            if let Some(error) = &self.calibration_error {
                menu = menu.push(text(format!("Example not saved: {error}")).size(14));
                if !self.reference_conflicts().is_empty() {
                    menu = menu.push(
                        button("Replace conflicting examples")
                            .style(primary_button)
                            .on_press(LegacyButton::ContextReference(
                                self.reference_speakers.clone(),
                                true,
                            )),
                    );
                }
            }
            let panel = container(menu)
                .padding(10)
                .width(320.0_f32.min(size.width))
                .style(editor_panel);
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
