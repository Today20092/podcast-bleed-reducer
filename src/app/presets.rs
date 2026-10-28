//! Dialog presentation and registered actions; sound/storage rules live in `sound` and `Project`.
use super::*;
use crate::sound::Presets;
use iced::widget::column;

pub(super) struct PresetDialog {
    collection: Presets,
    selected: String,
    name: String,
    error: Option<String>,
    path: Option<PathBuf>,
    writable: bool,
}
impl PresetDialog {
    pub(super) fn select(&mut self, selected: &str) {
        if self.collection.get(selected).is_some() {
            self.selected = selected.into();
            self.name = selected.into();
        }
    }
    pub(super) fn edit_name(&mut self, name: &str) {
        self.name = name.into();
        if self.writable {
            self.error = None;
        }
    }
}

impl CleanSlateApp {
    pub(super) fn preset_action(&mut self, command: Command) -> Task<Message> {
        use Command::*;
        if command == SoundPresets {
            let path = self
                .preferences_path
                .as_ref()
                .map(|p| p.with_file_name("sound-presets.json"));
            let loaded = path
                .as_ref()
                .ok_or_else(|| "No valid user settings directory.".to_string())
                .and_then(|p| Presets::load(p).map_err(|e| format!("Could not load {}: {e} Keep a backup and repair this file before saving custom presets.", p.display())));
            let writable = loaded.is_ok();
            let (collection, error) = match loaded {
                Ok(p) => (p, None),
                Err(e) => (Presets::default(), Some(e)),
            };
            self.preset_dialog = Some(PresetDialog {
                collection,
                selected: "Default".into(),
                name: String::new(),
                error,
                path,
                writable,
            });
            return Task::none();
        }
        if command == ClosePresets {
            self.preset_dialog = None;
            return Task::none();
        }
        let Some(dialog) = self.preset_dialog.as_mut() else {
            return Task::none();
        };
        if command == ApplyPreset {
            let result = dialog
                .collection
                .get(&dialog.selected)
                .ok_or_else(|| "Choose a preset.".into())
                .and_then(|sound| {
                    self.project
                        .as_mut()
                        .ok_or_else(|| "Open a project first.".to_string())?
                        .apply_sound(sound)
                        .map_err(|e| e.to_string())
                });
            match result {
                Ok(true) => {
                    self.preview_parameters = self.project.as_ref().unwrap().parameters();
                    self.render_generation.invalidate();
                    self.queue_preview(Duration::from_millis(40));
                }
                Ok(false) => {}
                Err(error) => self.preset_dialog.as_mut().unwrap().error = Some(error),
            }
            return Task::none();
        }
        let replaces = matches!(command, UpdatePreset | DeletePreset)
            || (matches!(command, SavePreset | RenamePreset)
                && dialog.collection.contains_name(&dialog.name));
        if replaces
            && !confirm_preset(
                "Replace or remove the stored custom preset? Current project values remain unchanged.",
            )
        {
            return Task::none();
        }
        let mut next = dialog.collection.clone();
        let sound = crate::SoundSettings::from(self.preview_parameters);
        let result = match command {
            SavePreset => next.store(&dialog.name, sound, replaces),
            UpdatePreset => next.store(&dialog.selected, sound, true),
            RenamePreset => next.rename(&dialog.selected, &dialog.name, replaces),
            DeletePreset => next.delete(&dialog.selected).map(|()| "Default".into()),
            _ => return Task::none(),
        }
        .and_then(|selected| {
            let path = dialog
                .path
                .as_ref()
                .ok_or_else(|| std::io::Error::other("No valid settings directory."))?;
            next.save(path)?;
            Ok(selected)
        });
        match result {
            Ok(selected) => {
                dialog.collection = next;
                dialog.selected = selected;
                dialog.error = None;
            }
            Err(error) => dialog.error = Some(error.to_string()),
        }
        Task::none()
    }

    pub(super) fn preset_available(&self, command: Command) -> bool {
        let Some(dialog) = &self.preset_dialog else {
            return false;
        };
        match command {
            Command::ClosePresets => true,
            Command::ApplyPreset => {
                self.project.is_some() && dialog.collection.get(&dialog.selected).is_some()
            }
            Command::SavePreset => {
                self.project.is_some() && dialog.writable && !dialog.name.trim().is_empty()
            }
            Command::UpdatePreset | Command::RenamePreset | Command::DeletePreset => {
                dialog.writable && dialog.collection.is_custom(&dialog.selected)
            }
            _ => false,
        }
    }

    pub(super) fn preset_view(&self) -> Element<'_, Message> {
        let dialog = self.preset_dialog.as_ref().unwrap();
        let current = crate::SoundSettings::from(self.preview_parameters);
        let modified = dialog
            .collection
            .get(&dialog.selected)
            .is_some_and(|s| s != current);
        let mut body = column![
            text("Sound presets").size(24),
            text("Starting points for attenuation and transitions. Recording-specific edits stay in this project."),
            iced::widget::pick_list(dialog.collection.names(), Some(dialog.selected.clone()), Message::PresetSelected),
            text(if modified { "Current sound differs from this preset · Modified" } else { "Current sound matches this preset" }),
            text(if self.preview_parameters.mute { "Mute is on. Presets keep it on; attenuation differences are audible after you turn Mute off." } else { "Presets preserve Mute and all recording-specific work." }),
            self.action_control(Command::ApplyPreset, button("Apply preset").style(primary_button)),
            text_input("Custom preset name", &dialog.name).on_input(Message::PresetName),
        ].spacing(12);
        for (label, command) in [
            ("Save current sound as…", Command::SavePreset),
            ("Update selected custom preset", Command::UpdatePreset),
            ("Rename selected custom preset", Command::RenamePreset),
            ("Delete selected custom preset", Command::DeletePreset),
        ] {
            body = body.push(
                self.action_control(command, button(label).style(secondary_button).width(Fill)),
            );
        }
        if let Some(error) = &dialog.error {
            body = body.push(text(error).color(self.appearance.colors().error));
        }
        body = body.push(self.action_control(
            Command::ClosePresets,
            button("Close").style(secondary_button),
        ));
        container(scrollable(body).height(Fill))
            .padding(20)
            .max_width(560)
            .height(Fill)
            .style(editor_panel)
            .into()
    }
}

fn confirm_preset(description: &str) -> bool {
    rfd::MessageDialog::new()
        .set_title("Confirm preset change")
        .set_description(description)
        .set_buttons(rfd::MessageButtons::OkCancel)
        .show()
        == rfd::MessageDialogResult::Ok
}
