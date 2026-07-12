use std::path::Path;

use anyhow::Result;
use bunny_plugin::bunny_ui::{
    input::{KeyboardShortcut, Modifiers},
    key::Key,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeathRetireKind {
    HealthDepleted,
    #[default]
    Carted,
}

impl std::fmt::Display for DeathRetireKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DeathRetireKind::HealthDepleted => "On health depleted",
            DeathRetireKind::Carted => "On cart",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DeathRetire {
    pub enabled: bool,
    pub kind: DeathRetireKind,
    pub carts_needed: u32,
}

impl Default for DeathRetire {
    fn default() -> Self {
        Self {
            enabled: false,
            kind: Default::default(),
            carts_needed: 1,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub retire_keybind: KeyboardShortcut,
    pub retire_on_death: DeathRetire,
    pub wait_time_keybind: KeyboardShortcut,
    pub skip_quest_complete_wait: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            retire_keybind: KeyboardShortcut::new(Modifiers::CTRL, Key::R),
            retire_on_death: Default::default(),
            wait_time_keybind: KeyboardShortcut::new(Modifiers::CTRL, Key::X),
            skip_quest_complete_wait: false,
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let config = toml::from_slice(&bytes)?;
        Ok(config)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let contents = toml::to_string(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}
