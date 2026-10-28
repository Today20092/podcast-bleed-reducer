//! Reusable sound values. Recording-specific state and mute are deliberately excluded.
use crate::AutomixParameters;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoundSettings {
    pub attenuation_db: f32,
    pub open_ms: f32,
    pub close_ms: f32,
    pub minimum_gap_ms: f32,
}
impl From<AutomixParameters> for SoundSettings {
    fn from(p: AutomixParameters) -> Self {
        Self {
            attenuation_db: p.attenuation_db,
            open_ms: p.open_ms,
            close_ms: p.close_ms,
            minimum_gap_ms: p.minimum_gap_ms,
        }
    }
}
impl SoundSettings {
    pub fn validate(self) -> io::Result<()> {
        if !self.attenuation_db.is_finite()
            || !(-120.0..=0.0).contains(&self.attenuation_db)
            || [self.open_ms, self.close_ms, self.minimum_gap_ms]
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=60000.0).contains(v))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Sound requires finite attenuation from -120 to 0 dB and times from 0 to 60000 ms.",
            ));
        }
        Ok(())
    }
    pub(crate) fn apply_to(self, mut p: AutomixParameters) -> AutomixParameters {
        p.attenuation_db = self.attenuation_db;
        p.open_ms = self.open_ms;
        p.close_ms = self.close_ms;
        p.minimum_gap_ms = self.minimum_gap_ms;
        p
    }
}

pub(crate) const FACTORIES: [(&str, SoundSettings); 3] = [
    (
        "Default",
        SoundSettings {
            attenuation_db: -18.0,
            open_ms: 50.0,
            close_ms: 500.0,
            minimum_gap_ms: 768.0,
        },
    ),
    (
        "Gentle",
        SoundSettings {
            attenuation_db: -12.0,
            open_ms: 50.0,
            close_ms: 500.0,
            minimum_gap_ms: 768.0,
        },
    ),
    (
        "Strong",
        SoundSettings {
            attenuation_db: -24.0,
            open_ms: 50.0,
            close_ms: 500.0,
            minimum_gap_ms: 768.0,
        },
    ),
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedPreset {
    name: String,
    sound: SoundSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Presets {
    version: u32,
    custom: Vec<NamedPreset>,
}
impl Default for Presets {
    fn default() -> Self {
        Self {
            version: 1,
            custom: vec![],
        }
    }
}
fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}
fn name(value: &str) -> io::Result<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 120 || value.chars().any(char::is_control) {
        return Err(invalid(
            "Use a preset name of 1–120 bytes without control characters.",
        ));
    }
    if FACTORIES.iter().any(|(n, _)| n.eq_ignore_ascii_case(value)) {
        return Err(invalid("Factory preset names are reserved."));
    }
    Ok(value.into())
}
impl Presets {
    pub(crate) fn load(path: &Path) -> io::Result<Self> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e),
        };
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(invalid)?;
        if value.get("version").and_then(|v| v.as_u64()) != Some(1) {
            return Err(invalid(
                "Unsupported preset version; keep this file and use a compatible application.",
            ));
        }
        let presets: Self = serde_json::from_value(value).map_err(invalid)?;
        let mut names = std::collections::HashSet::new();
        for p in &presets.custom {
            if name(&p.name)? != p.name || !names.insert(p.name.to_lowercase()) {
                return Err(invalid("Invalid or duplicate custom preset name."));
            }
            p.sound.validate()?;
        }
        Ok(presets)
    }
    pub(crate) fn save(&self, path: &Path) -> io::Result<()> {
        // Read first: a corrupt or newer collection must never be replaced by fallback defaults.
        Self::load(path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        crate::project::atomic_save(path, &serde_json::to_vec_pretty(self).map_err(invalid)?)
    }
    pub(crate) fn names(&self) -> Vec<String> {
        FACTORIES
            .iter()
            .map(|(n, _)| (*n).into())
            .chain(self.custom.iter().map(|p| p.name.clone()))
            .collect()
    }
    pub(crate) fn get(&self, value: &str) -> Option<SoundSettings> {
        FACTORIES
            .iter()
            .find(|(n, _)| *n == value)
            .map(|(_, s)| *s)
            .or_else(|| {
                self.custom
                    .iter()
                    .find(|p| p.name == value)
                    .map(|p| p.sound)
            })
    }
    pub(crate) fn is_custom(&self, value: &str) -> bool {
        self.custom.iter().any(|p| p.name == value)
    }
    pub(crate) fn contains_name(&self, value: &str) -> bool {
        self.names()
            .iter()
            .any(|n| n.to_lowercase() == value.trim().to_lowercase())
    }
    pub(crate) fn store(
        &mut self,
        value: &str,
        sound: SoundSettings,
        replace: bool,
    ) -> io::Result<String> {
        let value = name(value)?;
        sound.validate()?;
        if let Some(preset) = self
            .custom
            .iter_mut()
            .find(|p| p.name.to_lowercase() == value.to_lowercase())
        {
            if !replace {
                return Err(invalid("A preset with that name already exists."));
            }
            *preset = NamedPreset {
                name: value.clone(),
                sound,
            };
        } else {
            self.custom.push(NamedPreset {
                name: value.clone(),
                sound,
            });
        }
        Ok(value)
    }
    pub(crate) fn delete(&mut self, value: &str) -> io::Result<()> {
        let index = self
            .custom
            .iter()
            .position(|p| p.name == value)
            .ok_or_else(|| invalid("Choose a custom preset."))?;
        self.custom.remove(index);
        Ok(())
    }
    pub(crate) fn rename(&mut self, old: &str, new: &str, replace: bool) -> io::Result<String> {
        if !self.is_custom(old) {
            return Err(invalid("Choose a custom preset."));
        }
        let sound = self.get(old).unwrap();
        let new = name(new)?;
        if old.eq_ignore_ascii_case(&new) {
            self.custom.iter_mut().find(|p| p.name == old).unwrap().name = new.clone();
        } else {
            self.store(&new, sound, replace)?;
            self.delete(old)?;
        }
        Ok(new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> std::path::PathBuf {
        crate::audio_io::multichannel_tests::root()
    }

    #[test]
    fn custom_presets_roundtrip_validate_names_and_leave_projects_independent() {
        let root = root();
        let path = root.join("presets.json");
        let mut presets = Presets::load(&path).unwrap();
        let gentle = presets.get("Gentle").unwrap();
        assert_eq!(gentle.attenuation_db, -12.0);
        assert_eq!(
            presets.get("Default").unwrap(),
            SoundSettings::from(AutomixParameters::default())
        );
        presets.store("  My room  ", gentle, false).unwrap();
        assert!(presets.store("MY ROOM", gentle, false).is_err());
        assert!(presets.store("default", gentle, true).is_err());
        let strong = presets.get("Strong").unwrap();
        presets.store("My room", strong, true).unwrap();
        assert_eq!(presets.get("My room"), Some(strong));
        presets.store("My room", gentle, true).unwrap();
        let mut invalid = gentle;
        invalid.open_ms = -1.0;
        assert!(presets.store("Broken", invalid, false).is_err());
        presets.save(&path).unwrap();
        let mut reopened = Presets::load(&path).unwrap();
        assert_eq!(reopened.get("My room"), Some(gentle));
        reopened.store("Other", strong, false).unwrap();
        assert!(reopened.rename("My room", "Other", false).is_err());
        assert_eq!(reopened.get("Other"), Some(strong));
        reopened.rename("My room", "Other", true).unwrap();
        assert_eq!(reopened.get("Other"), Some(gentle));
        reopened.rename("Other", "Quiet room", false).unwrap();
        assert!(reopened.get("My room").is_none());
        reopened.delete("Quiet room").unwrap();
        assert_eq!(gentle.attenuation_db, -12.0);
        assert!(reopened.delete("Default").is_err());
        reopened.save(&path).unwrap();
        assert_eq!(Presets::load(&path).unwrap().names().len(), 3);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsupported_corrupt_and_failed_writes_preserve_existing_bytes() {
        let root = root();
        let path = root.join("presets.json");
        for bytes in [br#"{"version":2,"custom":[]}"#.as_slice(), b"broken JSON"] {
            fs::write(&path, bytes).unwrap();
            assert!(Presets::load(&path).is_err());
            assert!(Presets::default().save(&path).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        fs::write(&path, br#"{"version":1,"custom":[]}"#).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).unwrap();
        let mut presets = Presets::load(&path).unwrap();
        presets.store("Custom", FACTORIES[1].1, false).unwrap();
        #[cfg(windows)]
        assert!(presets.save(&path).is_err());
        #[cfg(windows)]
        assert_eq!(fs::read(&path).unwrap(), br#"{"version":1,"custom":[]}"#);
        // Restore the original writable permissions before removing the fixture.
        let permissions = fs::metadata(&root).unwrap().permissions();
        fs::set_permissions(&path, permissions).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
