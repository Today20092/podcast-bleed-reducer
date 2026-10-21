use iced::{Color, Theme};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Appearance {
    #[default]
    Dark,
    Light,
}
pub(crate) fn preference_path() -> Result<PathBuf, String> {
    path_for(
        std::env::consts::OS,
        std::env::var_os("LOCALAPPDATA"),
        std::env::var_os("HOME"),
        std::env::var_os("XDG_CONFIG_HOME"),
    )
}
fn path_for(
    os: &str,
    local: Option<OsString>,
    home: Option<OsString>,
    xdg: Option<OsString>,
) -> Result<PathBuf, String> {
    let absolute = |value: Option<OsString>| value.map(PathBuf::from).filter(|p| p.is_absolute());
    let root = match os {
        "windows" => absolute(local).map(|p| p.join("CleanSlate")),
        // shortcut: unsandboxed package location; use Foundation if packaging becomes sandboxed.
        "macos" => absolute(home)
            .map(|p| p.join("Library/Application Support/io.github.Today20092.CleanSlate")),
        _ => absolute(xdg)
            .or_else(|| absolute(home).map(|p| p.join(".config")))
            .map(|p| p.join("cleanslate")),
    };
    root.map(|p| p.join("preferences.json")).ok_or_else(|| {
        "Appearance preferences are unavailable: no valid user configuration directory.".into()
    })
}
pub(crate) fn load(path: &Path) -> (Appearance, Option<String>) {
    match std::fs::read(path) {
        Ok(bytes) => match serde_json::from_slice::<serde_json::Value>(&bytes)
            .and_then(|value| serde_json::from_value::<Appearance>(value["appearance"].clone()))
        {
            Ok(appearance) => (appearance, None),
            Err(error) => (
                Appearance::Dark,
                Some(format!(
                    "Could not read appearance preferences; using Dark. {error}"
                )),
            ),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Appearance::Dark, None),
        Err(error) => (
            Appearance::Dark,
            Some(format!(
                "Could not read appearance preferences; using Dark. {error}"
            )),
        ),
    }
}
pub(crate) fn save(path: &Path, appearance: Appearance) -> Result<(), String> {
    update_preferences(
        path,
        [(
            "appearance",
            serde_json::to_value(appearance).map_err(|e| e.to_string())?,
        )],
    )
}

fn read_preferences(path: &Path) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
            format!(
                "Could not read preferences; keep a copy and repair {} before saving. {e}",
                path.display()
            )
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Default::default()),
        Err(error) => Err(format!("Could not read {}. {error}", path.display())),
    }
}

fn update_preferences<const N: usize>(
    path: &Path,
    updates: [(&str, serde_json::Value); N],
) -> Result<(), String> {
    let mut preferences = read_preferences(path)?;
    for (name, value) in updates {
        preferences.insert(name.into(), value);
    }
    let parent = path
        .parent()
        .ok_or("Preferences need a parent directory.")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&preferences).map_err(|e| e.to_string())?;
    crate::project::atomic_save(path, &bytes).map_err(|e| e.to_string())
}

pub(crate) fn load_shortcuts(path: &Path) -> (crate::shortcuts::Shortcuts, bool, Option<String>) {
    let preferences = match read_preferences(path) {
        Ok(preferences) => preferences,
        Err(error) => return (Default::default(), false, Some(error)),
    };
    let (shortcuts, mut errors) = preferences
        .get("shortcuts")
        .map(crate::shortcuts::Shortcuts::from_value)
        .unwrap_or_default();
    let unlock = match preferences.get("unlock_playhead_when_zooming") {
        Some(serde_json::Value::Bool(value)) => *value,
        None => false,
        _ => {
            errors.push("Invalid Unlock playhead when zooming preference; using off. Set it again in Settings.".into());
            false
        }
    };
    (
        shortcuts,
        unlock,
        (!errors.is_empty()).then(|| errors.join("\n")),
    )
}

pub(crate) fn save_shortcuts(
    path: &Path,
    shortcuts: &crate::shortcuts::Shortcuts,
    unlock: bool,
) -> Result<(), String> {
    update_preferences(
        path,
        [
            (
                "shortcuts",
                serde_json::to_value(shortcuts).map_err(|e| e.to_string())?,
            ),
            ("unlock_playhead_when_zooming", unlock.into()),
        ],
    )
}

#[derive(Clone, Copy)]
pub(crate) struct Colors {
    pub surface: Color,
    pub panel: Color,
    pub text: Color,
    pub muted: Color,
    pub disabled: Color,
    pub control: Color,
    pub hover: Color,
    pub pressed: Color,
    pub border: Color,
    pub panel_border: Color,
    pub action: Color,
    pub action_hover: Color,
    pub action_pressed: Color,
    pub action_text: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub focus: Color,
    pub scrim: Color,
    pub selection: Color,
    pub zoom: Color,
    pub attenuation: Color,
    pub waveform: Color,
    pub original_waveform: Color,
    pub gain: Color,
    pub automatic_gain: Color,
    pub manual_gain: Color,
    pub protection: Color,
    pub label: Color,
    pub active_label: Color,
    pub selected_label: Color,
    pub label_text: Color,
    pub inactive: Color,
    pub playhead: Color,
}
impl Appearance {
    pub(crate) fn from_theme(theme: &Theme) -> Self {
        if theme.palette().background.r > 0.5 {
            Self::Light
        } else {
            Self::Dark
        }
    }
    pub(crate) fn colors(self) -> Colors {
        match self {
            Self::Dark => Colors {
                surface: iced::color!(0x202226),
                panel: iced::color!(0x292D33),
                text: iced::color!(0xE4E7EB),
                muted: iced::color!(0xADB5C0),
                disabled: iced::color!(0xA8B0BC),
                control: iced::color!(0x343940),
                hover: iced::color!(0x414750),
                pressed: iced::color!(0x272B31),
                border: iced::color!(0x8290A2),
                panel_border: iced::color!(0x3D434C),
                action: iced::color!(0x779BC7),
                action_hover: iced::color!(0x8FB0D8),
                action_pressed: iced::color!(0x6B8DB8),
                action_text: iced::color!(0x152233),
                success: iced::color!(0x76B99A),
                warning: iced::color!(0xD8B579),
                error: iced::color!(0xDB8B8B),
                focus: iced::color!(0x9AC5FF),
                scrim: Color::from_rgba(0.0, 0.0, 0.0, 0.7),
                selection: Color::from_rgba8(70, 120, 220, 0.18),
                zoom: Color::from_rgba8(95, 180, 255, 0.25),
                attenuation: Color::from_rgba8(220, 65, 65, 0.24),
                waveform: iced::color!(0x6FBEB7),
                original_waveform: Color::from_rgba8(111, 190, 183, 0.65),
                gain: iced::color!(0xD282F5),
                automatic_gain: iced::color!(0x8FABC7),
                manual_gain: iced::color!(0xEB96F5),
                protection: iced::color!(0xDCAF41),
                label: iced::color!(0x6E87BE),
                active_label: iced::color!(0x5A87FF),
                selected_label: iced::color!(0xA0B9FF),
                label_text: iced::color!(0x101824),
                inactive: iced::color!(0xA0A9BB),
                playhead: iced::color!(0xFFFFFF),
            },
            Self::Light => Colors {
                surface: iced::color!(0xF3F5F7),
                panel: iced::color!(0xFFFFFF),
                text: iced::color!(0x202936),
                muted: iced::color!(0x4D5969),
                disabled: iced::color!(0x566376),
                control: iced::color!(0xE5EAF0),
                hover: iced::color!(0xD5DEEA),
                pressed: iced::color!(0xC4D1E1),
                border: iced::color!(0x697C94),
                panel_border: iced::color!(0xA6B1C0),
                action: iced::color!(0x285A91),
                action_hover: iced::color!(0x244E7C),
                action_pressed: iced::color!(0x1C416B),
                action_text: iced::color!(0xFFFFFF),
                success: iced::color!(0x246647),
                warning: iced::color!(0x78520A),
                error: iced::color!(0xA22D3A),
                focus: iced::color!(0x164B88),
                scrim: Color::from_rgba(0.10, 0.14, 0.20, 0.35),
                selection: Color::from_rgba8(40, 90, 170, 0.15),
                zoom: Color::from_rgba8(25, 90, 155, 0.22),
                attenuation: Color::from_rgba8(185, 40, 50, 0.16),
                waveform: iced::color!(0x176D66),
                original_waveform: Color::from_rgba8(23, 109, 102, 0.75),
                gain: iced::color!(0x76359A),
                automatic_gain: iced::color!(0x536D83),
                manual_gain: iced::color!(0x843C95),
                protection: iced::color!(0x856009),
                label: iced::color!(0xB9CBE5),
                active_label: iced::color!(0x285A91),
                selected_label: iced::color!(0x92B2DD),
                label_text: iced::color!(0x101824),
                inactive: iced::color!(0x667589),
                playhead: iced::color!(0x17283D),
            },
        }
    }
    pub(crate) fn theme(self) -> Theme {
        let c = self.colors();
        Theme::custom(
            match self {
                Self::Dark => "CleanSlate Dark",
                Self::Light => "CleanSlate Light",
            },
            iced::theme::Palette {
                background: c.surface,
                text: c.text,
                primary: c.action,
                success: c.success,
                warning: c.warning,
                danger: c.error,
            },
        )
    }
}
pub(crate) fn colors(theme: &Theme) -> Colors {
    Appearance::from_theme(theme).colors()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contrast(foreground: Color, background: Color) -> f32 {
        let luminance = |color: Color| {
            let linear = |c: f32| {
                if c <= 0.04045 {
                    c / 12.92
                } else {
                    ((c + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
        };
        let (a, b) = (luminance(foreground), luminance(background));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    fn composite(foreground: Color, background: Color) -> Color {
        Color::from_rgb(
            foreground.r * foreground.a + background.r * (1.0 - foreground.a),
            foreground.g * foreground.a + background.g * (1.0 - foreground.a),
            foreground.b * foreground.a + background.b * (1.0 - foreground.a),
        )
    }
    #[test]
    fn both_palettes_have_readable_text_controls_and_composited_overlays() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            let c = appearance.colors();
            for background in [c.surface, c.panel, c.control, c.hover, c.pressed] {
                for foreground in [c.text, c.muted] {
                    assert!(
                        contrast(foreground, background) >= 4.5,
                        "{appearance:?} text {foreground:?} on {background:?}"
                    );
                }
            }
            for background in [c.surface, c.panel] {
                for foreground in [c.error, c.warning, c.success] {
                    assert!(
                        contrast(foreground, background) >= 4.5,
                        "{appearance:?} status"
                    );
                }
                for foreground in [c.focus, c.border] {
                    assert!(
                        contrast(foreground, background) >= 3.0,
                        "{appearance:?} focus/control"
                    );
                }
                for overlay in [c.selection, c.attenuation, c.zoom] {
                    let background = composite(overlay, background);
                    assert!(
                        contrast(c.text, background) >= 4.5,
                        "{appearance:?} overlay text"
                    );
                    assert!(
                        contrast(c.playhead, background) >= 3.0,
                        "{appearance:?} overlay playhead"
                    );
                    assert!(
                        contrast(c.waveform, background) >= 3.0,
                        "{appearance:?} overlay waveform"
                    );
                    assert!(
                        contrast(c.gain, background) >= 3.0,
                        "{appearance:?} overlay gain"
                    );
                }
                let combined = composite(c.selection, composite(c.attenuation, background));
                assert!(
                    contrast(c.text, combined) >= 4.5,
                    "{appearance:?} combined overlays"
                );
                for foreground in [
                    c.waveform,
                    c.gain,
                    c.automatic_gain,
                    c.manual_gain,
                    c.protection,
                ] {
                    assert!(
                        contrast(foreground, background) >= 3.0,
                        "{appearance:?} drawing role {foreground:?}"
                    );
                }
                assert!(
                    contrast(composite(c.original_waveform, background), background) >= 3.0,
                    "{appearance:?} original waveform alpha"
                );
            }
            for background in [c.action, c.action_hover, c.action_pressed] {
                assert!(
                    contrast(c.action_text, background) >= 4.5,
                    "{appearance:?} action"
                );
            }
            for background in [c.label, c.selected_label] {
                assert!(
                    contrast(c.label_text, background) >= 4.5,
                    "{appearance:?} labels"
                );
            }
            assert!(
                contrast(c.disabled, c.control) >= 4.5,
                "{appearance:?} disabled"
            );
        }
    }
    #[test]
    fn preference_roots_are_absolute_platform_specific_and_never_project_relative() {
        let root = std::env::temp_dir();
        let value = Some(root.clone().into_os_string());
        let relative = Some(OsString::from("relative"));
        assert_eq!(
            path_for("windows", value.clone(), None, None).unwrap(),
            root.join("CleanSlate/preferences.json")
        );
        assert!(path_for("windows", relative.clone(), value.clone(), None).is_err());
        assert!(path_for("windows", Some(OsString::new()), value.clone(), None).is_err());
        assert_eq!(
            path_for("macos", None, value.clone(), None).unwrap(),
            root.join(
                "Library/Application Support/io.github.Today20092.CleanSlate/preferences.json"
            )
        );
        assert!(path_for("macos", None, relative.clone(), None).is_err());
        assert_eq!(
            path_for("linux", None, None, value.clone()).unwrap(),
            root.join("cleanslate/preferences.json")
        );
        assert_eq!(
            path_for("linux", None, value.clone(), relative).unwrap(),
            root.join(".config/cleanslate/preferences.json")
        );
        assert!(path_for("linux", None, None, None).is_err());
    }

    #[test]
    fn preferences_default_reload_invalid_and_failed_save() {
        let dir =
            std::env::temp_dir().join(format!("cleanslate-appearance-{}", std::process::id()));
        let path = dir.join("preferences.json");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(load(&path), (Appearance::Dark, None));
        save(&path, Appearance::Light).unwrap();
        assert_eq!(load(&path), (Appearance::Light, None));
        save(&path, Appearance::Dark).unwrap();
        assert_eq!(load(&path), (Appearance::Dark, None));
        for invalid in ["{", "{}", "{\"appearance\":\"unknown\"}"] {
            std::fs::write(&path, invalid).unwrap();
            let (appearance, error) = load(&path);
            assert_eq!(appearance, Appearance::Dark);
            assert!(error.is_some());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), invalid);
        }
        assert!(save(&path.join("invalid.json"), Appearance::Light).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
