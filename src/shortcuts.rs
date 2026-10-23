use iced::keyboard::{Key, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Context {
    Workspace,
    Labels,
    Gain,
    Popup,
    Project,
}
impl Context {
    fn mask(self) -> u8 {
        match self {
            Self::Workspace => 3,
            Self::Labels => 1,
            Self::Gain => 2,
            Self::Popup => 4,
            Self::Project => 7,
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Workspace => "Workspace (no popup, dialog or text entry)",
            Self::Labels => "Workspace · gain editing off",
            Self::Gain => "Workspace · gain editing on",
            Self::Popup => "Who is talking? popup",
            Self::Project => "Workspace or speaker popup (no text entry)",
        }
    }
}

pub(crate) struct Definition {
    pub command: Command,
    pub name: &'static str,
    pub context: Context,
    pub defaults: &'static [&'static str],
}
// One declaration owns identity, display metadata and optional bindings.
macro_rules! commands {
    ($($command:ident => ($name:literal, $context:ident, [$($default:literal),*])),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        pub(crate) enum Command { $($command),* }
        pub(crate) const DEFINITIONS: &[Definition] = &[
            $(Definition { command: Command::$command, name: $name,
                context: Context::$context, defaults: &[$($default),*] }),*
        ];
        impl Command {
            pub(crate) fn definition(self) -> &'static Definition {
                match self { $(Self::$command => &DEFINITIONS[Self::$command as usize]),* }
            }
        }
    };
}
commands! {
    Play => ("Play or stop", Workspace, ["Space"]),
    Start => ("Seek to start", Workspace, ["Home"]),
    Original => ("Monitor Original", Workspace, ["1"]),
    Processed => ("Monitor Processed", Workspace, ["2"]),
    Removed => ("Monitor Removed", Workspace, ["3"]),
    FitRecording => ("Fit recording", Workspace, ["f"]),
    FitSelection => ("Fit selection", Workspace, ["Shift+f"]),
    Center => ("Center playhead", Workspace, ["c"]),
    Lock => ("Toggle playhead lock", Workspace, ["l"]),
    CatchUp => ("Toggle catch up", Workspace, ["Shift+c"]),
    Meters => ("Move speaker meters", Workspace, []),
    Help => ("Open Help", Workspace, ["Alt+h"]),
    Menu => ("Open Menu", Workspace, ["Alt+m"]),
    Reference => ("Open keyboard shortcuts", Workspace, ["Ctrl+/"]),
    Support => ("Purchase CleanSlate", Workspace, []),
    Purchase => ("Purchase on Ko-fi", Workspace, []),
    Sponsor => ("Support on GitHub Sponsors", Workspace, []),
    Roadmap => ("View project roadmap", Workspace, []),
    SaveProject => ("Save project", Project, ["Cmd+s"]),
    UndoLabels => ("Undo labels", Labels, ["Cmd+z"]),
    RedoLabels => ("Redo labels", Labels, ["Cmd+Shift+z", "Cmd+y"]),
    UndoGain => ("Undo gain", Gain, ["Cmd+z"]),
    RedoGain => ("Redo gain", Gain, ["Cmd+Shift+z", "Cmd+y"]),
    DeleteGain => ("Delete selected manual gain edit", Gain, ["Delete"]),
    Earlier => ("Move selected point earlier", Gain, ["ArrowLeft"]),
    Later => ("Move selected point later", Gain, ["ArrowRight"]),
    Louder => ("Increase selected point gain", Gain, ["ArrowUp"]),
    Quieter => ("Decrease selected point gain", Gain, ["ArrowDown"]),
    SaveExample => ("Save example", Popup, ["s"]),
    ClearSpeakers => ("Clear speakers", Popup, ["r"]),
    Loop => ("Pause / resume loop", Popup, ["Space"]),
    Speaker1 => ("Toggle speaker 1", Popup, ["1"]),
    Speaker2 => ("Toggle speaker 2", Popup, ["2"]),
    Speaker3 => ("Toggle speaker 3", Popup, ["3"]),
    Speaker4 => ("Toggle speaker 4", Popup, ["4"]),
    Speaker5 => ("Toggle speaker 5", Popup, ["5"]),
    Speaker6 => ("Toggle speaker 6", Popup, ["6"]),
    Speaker7 => ("Toggle speaker 7", Popup, ["7"]),
    Speaker8 => ("Toggle speaker 8", Popup, ["8"]),
}
use Command::*;
impl Command {
    pub(crate) fn speaker(index: usize) -> Option<Self> {
        [
            Speaker1, Speaker2, Speaker3, Speaker4, Speaker5, Speaker6, Speaker7, Speaker8,
        ]
        .get(index)
        .copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Binding {
    key: String,
    #[serde(default)]
    control: bool,
    #[serde(default)]
    alt: bool,
    #[serde(default)]
    shift: bool,
    #[serde(default)]
    logo: bool,
}
impl Binding {
    pub(crate) fn capture(key: &Key, modifiers: Modifiers) -> Result<Self, String> {
        let key =
            match key.as_ref() {
                Key::Character(value) if value.chars().count() == 1 => value.to_lowercase(),
                Key::Named(named) => format!("{named:?}"),
                _ => return Err(
                    "Press a character, navigation key or function key with optional modifiers."
                        .into(),
                ),
            };
        let binding = Self {
            key,
            control: modifiers.control(),
            alt: modifiers.alt(),
            shift: modifiers.shift(),
            logo: modifiers.logo(),
        };
        binding
            .supported()
            .then_some(binding)
            .ok_or_else(|| "This key is reserved for widgets or the operating system.".into())
    }
    fn supported(&self) -> bool {
        (self.key.chars().count() == 1
            && !self
                .key
                .chars()
                .any(|c| c.is_control() || c.is_whitespace())
            && self.key.to_lowercase() == self.key)
            || [
                "Space",
                "Home",
                "End",
                "Delete",
                "Insert",
                "ArrowLeft",
                "ArrowRight",
                "ArrowUp",
                "ArrowDown",
                "PageUp",
                "PageDown",
                "Backspace",
            ]
            .contains(&self.key.as_str())
            || self
                .key
                .strip_prefix('F')
                .and_then(|v| v.parse::<u8>().ok())
                .is_some_and(|v| (1..=24).contains(&v) && self.key == format!("F{v}"))
    }
    fn default(value: &str) -> Self {
        let mut parts = value.split('+').collect::<Vec<_>>();
        let key = parts.pop().expect("default key").to_string();
        Self {
            key,
            control: parts.contains(&"Ctrl")
                || (parts.contains(&"Cmd") && !cfg!(target_os = "macos")),
            logo: parts.contains(&"Cmd") && cfg!(target_os = "macos"),
            alt: parts.contains(&"Alt"),
            shift: parts.contains(&"Shift"),
        }
    }
    pub(crate) fn label(&self) -> String {
        let mut parts = Vec::new();
        if self.control {
            parts.push("Ctrl".into());
        }
        if self.logo {
            parts.push(
                if cfg!(target_os = "macos") {
                    "Command"
                } else {
                    "Super"
                }
                .into(),
            );
        }
        if self.alt {
            parts.push("Alt".into());
        }
        if self.shift {
            parts.push("Shift".into());
        }
        parts.push(match self.key.as_str() {
            "+" => "Plus".into(),
            "ArrowLeft" => "Left".into(),
            "ArrowRight" => "Right".into(),
            "ArrowUp" => "Up".into(),
            "ArrowDown" => "Down".into(),
            "PageUp" => "Page Up".into(),
            "PageDown" => "Page Down".into(),
            _ if self.key.chars().count() == 1 => self.key.to_uppercase(),
            _ => self.key.clone(),
        });
        parts.join("+")
    }
    fn reserved(&self, command: Command) -> bool {
        let command_modifier = if cfg!(target_os = "macos") {
            self.logo
        } else {
            self.control
        };
        !self.supported()
            || (command_modifier
                && !self.alt
                && !self.shift
                && self.key == "s"
                && command != SaveProject)
            || (command_modifier && ["a", "c", "v", "x", "q", "w"].contains(&self.key.as_str()))
            || (self.alt && ["F4", "Tab", "Space"].contains(&self.key.as_str()))
            || (self.control && self.alt)
            || (self.logo
                && (!cfg!(target_os = "macos") || ["h", "m", "Space"].contains(&self.key.as_str())))
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub(crate) struct Shortcuts(BTreeMap<Command, Option<Binding>>);
impl Shortcuts {
    fn bindings(&self, command: Command) -> Vec<Binding> {
        let mut bindings: Vec<Binding> = match self.0.get(&command) {
            Some(binding) => binding.iter().cloned().collect(),
            None => command
                .definition()
                .defaults
                .iter()
                .map(|s| Binding::default(s))
                .collect(),
        };
        // Retain the platform project-save chord as a compatibility binding.
        if command == SaveProject {
            let compatibility = Binding::default(command.definition().defaults[0]);
            if !bindings.contains(&compatibility) {
                bindings.push(compatibility);
            }
        }
        bindings
    }
    pub(crate) fn hint(&self, command: Command) -> String {
        let labels = self
            .bindings(command)
            .iter()
            .map(Binding::label)
            .collect::<Vec<_>>();
        if labels.is_empty() {
            "Unbound".into()
        } else {
            labels.join(" | ")
        }
    }
    pub(crate) fn resolve(
        &self,
        key: &Key,
        modifiers: Modifiers,
        context: Context,
    ) -> Option<Command> {
        let binding = Binding::capture(key, modifiers).ok()?;
        DEFINITIONS
            .iter()
            .find(|d| {
                d.context.mask() & context.mask() != 0
                    && self.bindings(d.command).contains(&binding)
            })
            .map(|d| d.command)
    }
    pub(crate) fn validate(
        &self,
        command: Command,
        binding: Option<&Binding>,
    ) -> Result<(), String> {
        let Some(binding) = binding else {
            return Ok(());
        };
        if binding.reserved(command) {
            return Err("Reserved for text editing, confirmation, navigation or the operating system; Ctrl/Command+S always saves the project.".into());
        }
        for definition in DEFINITIONS {
            if definition.command != command
                && definition.context.mask() & command.definition().context.mask() != 0
                && self.bindings(definition.command).contains(binding)
            {
                return Err(format!(
                    "{} conflicts with {} in {}.",
                    binding.label(),
                    definition.name,
                    definition.context.name()
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn assign(
        &mut self,
        command: Command,
        binding: Option<Binding>,
    ) -> Result<(), String> {
        self.validate(command, binding.as_ref())?;
        self.0.insert(command, binding);
        Ok(())
    }
    pub(crate) fn restore(&mut self, command: Command) -> Result<(), String> {
        for binding in command.definition().defaults {
            self.validate(command, Some(&Binding::default(binding)))?;
        }
        self.0.remove(&command);
        Ok(())
    }
    pub(crate) fn from_value(value: &serde_json::Value) -> (Self, Vec<String>) {
        let mut shortcuts = Self::default();
        let mut errors = Vec::new();
        let Some(entries) = value.as_object() else {
            return (
                shortcuts,
                vec!["Keyboard shortcuts must be an object. Restore defaults in Settings.".into()],
            );
        };
        for (name, value) in entries {
            let command =
                serde_json::from_value::<Command>(serde_json::Value::String(name.clone()));
            let binding = serde_json::from_value::<Option<Binding>>(value.clone());
            match (command, binding) {
                (Ok(command), Ok(binding)) if binding.as_ref().is_none_or(|b| !b.reserved(command)) => { shortcuts.0.insert(command, binding); }
                _ => errors.push(format!("Ignored invalid or obsolete shortcut {name}; check Keyboard shortcuts in Settings.")),
            }
        }
        // Validate the complete map, including overrides which vacate another command's default.
        loop {
            let invalid = shortcuts.0.iter().find_map(|(&command, binding)| {
                shortcuts
                    .validate(command, binding.as_ref())
                    .err()
                    .map(|error| (command, error))
            });
            let Some((command, error)) = invalid else {
                break;
            };
            shortcuts.0.remove(&command);
            errors.push(format!(
                "Using default for {}. {error}",
                command.definition().name
            ));
        }
        (shortcuts, errors)
    }
}
