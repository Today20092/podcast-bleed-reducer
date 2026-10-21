use iced::keyboard::{Key, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) enum Command {
    Play,
    Start,
    Original,
    Processed,
    Removed,
    FitRecording,
    FitSelection,
    Center,
    Lock,
    Help,
    Menu,
    Reference,
    SaveProject,
    UndoLabels,
    RedoLabels,
    UndoGain,
    RedoGain,
    DeleteGain,
    Earlier,
    Later,
    Louder,
    Quieter,
    SaveExample,
    ClearSpeakers,
    Loop,
    Speaker1,
    Speaker2,
    Speaker3,
    Speaker4,
    Speaker5,
    Speaker6,
    Speaker7,
    Speaker8,
}

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
use Command::*;
pub(crate) const DEFINITIONS: &[Definition] = &[
    Definition {
        command: Play,
        name: "Play or stop",
        context: Context::Workspace,
        defaults: &["Space"],
    },
    Definition {
        command: Start,
        name: "Seek to start",
        context: Context::Workspace,
        defaults: &["Home"],
    },
    Definition {
        command: Original,
        name: "Monitor Original",
        context: Context::Workspace,
        defaults: &["1"],
    },
    Definition {
        command: Processed,
        name: "Monitor Processed",
        context: Context::Workspace,
        defaults: &["2"],
    },
    Definition {
        command: Removed,
        name: "Monitor Removed",
        context: Context::Workspace,
        defaults: &["3"],
    },
    Definition {
        command: FitRecording,
        name: "Fit recording",
        context: Context::Workspace,
        defaults: &["f"],
    },
    Definition {
        command: FitSelection,
        name: "Fit selection",
        context: Context::Workspace,
        defaults: &["Shift+f"],
    },
    Definition {
        command: Center,
        name: "Center playhead",
        context: Context::Workspace,
        defaults: &["c"],
    },
    Definition {
        command: Lock,
        name: "Toggle playhead lock",
        context: Context::Workspace,
        defaults: &["l"],
    },
    Definition {
        command: Help,
        name: "Open Help",
        context: Context::Workspace,
        defaults: &["Alt+h"],
    },
    Definition {
        command: Menu,
        name: "Open Menu",
        context: Context::Workspace,
        defaults: &["Alt+m"],
    },
    Definition {
        command: Reference,
        name: "Open keyboard shortcuts",
        context: Context::Workspace,
        defaults: &["Ctrl+/"],
    },
    Definition {
        command: SaveProject,
        name: "Save project",
        context: Context::Project,
        defaults: &["Cmd+s"],
    },
    Definition {
        command: UndoLabels,
        name: "Undo labels",
        context: Context::Labels,
        defaults: &["Cmd+z"],
    },
    Definition {
        command: RedoLabels,
        name: "Redo labels",
        context: Context::Labels,
        defaults: &["Cmd+Shift+z", "Cmd+y"],
    },
    Definition {
        command: UndoGain,
        name: "Undo gain",
        context: Context::Gain,
        defaults: &["Cmd+z"],
    },
    Definition {
        command: RedoGain,
        name: "Redo gain",
        context: Context::Gain,
        defaults: &["Cmd+Shift+z", "Cmd+y"],
    },
    Definition {
        command: DeleteGain,
        name: "Delete selected manual gain edit",
        context: Context::Gain,
        defaults: &["Delete"],
    },
    Definition {
        command: Earlier,
        name: "Move selected point earlier",
        context: Context::Gain,
        defaults: &["ArrowLeft"],
    },
    Definition {
        command: Later,
        name: "Move selected point later",
        context: Context::Gain,
        defaults: &["ArrowRight"],
    },
    Definition {
        command: Louder,
        name: "Increase selected point gain",
        context: Context::Gain,
        defaults: &["ArrowUp"],
    },
    Definition {
        command: Quieter,
        name: "Decrease selected point gain",
        context: Context::Gain,
        defaults: &["ArrowDown"],
    },
    Definition {
        command: SaveExample,
        name: "Save example",
        context: Context::Popup,
        defaults: &["s"],
    },
    Definition {
        command: ClearSpeakers,
        name: "Clear speakers",
        context: Context::Popup,
        defaults: &["r"],
    },
    Definition {
        command: Loop,
        name: "Pause / resume loop",
        context: Context::Popup,
        defaults: &["Space"],
    },
    Definition {
        command: Speaker1,
        name: "Toggle speaker 1",
        context: Context::Popup,
        defaults: &["1"],
    },
    Definition {
        command: Speaker2,
        name: "Toggle speaker 2",
        context: Context::Popup,
        defaults: &["2"],
    },
    Definition {
        command: Speaker3,
        name: "Toggle speaker 3",
        context: Context::Popup,
        defaults: &["3"],
    },
    Definition {
        command: Speaker4,
        name: "Toggle speaker 4",
        context: Context::Popup,
        defaults: &["4"],
    },
    Definition {
        command: Speaker5,
        name: "Toggle speaker 5",
        context: Context::Popup,
        defaults: &["5"],
    },
    Definition {
        command: Speaker6,
        name: "Toggle speaker 6",
        context: Context::Popup,
        defaults: &["6"],
    },
    Definition {
        command: Speaker7,
        name: "Toggle speaker 7",
        context: Context::Popup,
        defaults: &["7"],
    },
    Definition {
        command: Speaker8,
        name: "Toggle speaker 8",
        context: Context::Popup,
        defaults: &["8"],
    },
];
impl Command {
    pub(crate) fn definition(self) -> &'static Definition {
        DEFINITIONS
            .iter()
            .find(|d| d.command == self)
            .expect("command definition")
    }
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
        parts.push(if self.key.chars().count() == 1 {
            self.key.to_uppercase()
        } else {
            self.key.clone()
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
        // Project save and popup Enter are fixed compatibility confirmations.
        if command == SaveProject && !bindings.contains(&Binding::default("Cmd+s")) {
            bindings.push(Binding::default("Cmd+s"));
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
