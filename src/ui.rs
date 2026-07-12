use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use bunny_plugin::{
    PluginContext,
    bunny_ui::{
        ui::BunnyUi,
        widgets::{drag_value::DragValue, shortcut_button::ShortcutButton},
    },
};
use tracing::error;

use crate::{address::Addresses, config::{Config, DeathRetireKind}};

const TRIGGER_TIMEOUT: Duration = Duration::from_secs(1);

pub struct State {
    pub context: PluginContext,
    pub addresses: Addresses,
    pub config: Config,
    config_path: PathBuf,
    pub triggers: Triggers,
}

impl State {
    pub fn new(context: PluginContext, addresses: Addresses) -> Self {
        let config_path = context
            .config_dir()
            .join(format!("{}.toml", env!("CARGO_PKG_NAME")));
        let config = Config::load(&config_path).unwrap_or_default();
        Self {
            context,
            addresses,
            config_path,
            config,
            triggers: Triggers::default(),
        }
    }
}

impl<'a> State {
    pub fn menu(&'a mut self, ui: &mut BunnyUi<'a>) {
        ui.horizontal(|ui| {
            ui.label("Retire keybind:");
            ui.add(ShortcutButton::new(
                &mut self.config.retire_keybind,
                "Retire keybind",
            ));
        });

        ui.label("Auto retire triggers:");
        ui.indent(|ui| {
            let temp = self.config.retire_on_death.enabled;
            ui.checkbox(&mut self.config.retire_on_death.enabled, "Death");
            if temp {
                ui.indent(|ui| {
                    ui.radio_value(
                        &mut self.config.retire_on_death.kind,
                        DeathRetireKind::HealthDepleted,
                        DeathRetireKind::HealthDepleted.to_string(),
                    );
                    ui.radio_value(
                        &mut self.config.retire_on_death.kind,
                        DeathRetireKind::Carted,
                        DeathRetireKind::Carted.to_string(),
                    );
                    if let DeathRetireKind::Carted = self.config.retire_on_death.kind {
                        ui.indent(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Carts needed:");
                                ui.add(
                                    DragValue::new(&mut self.config.retire_on_death.carts_needed)
                                        .range(1.0..=99.0)
                                        .speed(0.05),
                                )
                            });
                        });
                    }
                });
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Wait time skip keybind:");
            ui.add(ShortcutButton::new(
                &mut self.config.wait_time_keybind,
                "Wait time keybind",
            ));
        });
        ui.checkbox(
            &mut self.config.skip_quest_complete_wait,
            "Skip quest complete wait time",
        );
    }

    pub fn ui(&mut self, ui: &mut BunnyUi) {
        self.triggers.handle_timeouts();

        ui.input_mut(|i| {
            if i.consume_shortcut(&self.config.retire_keybind) {
                self.triggers.retire();
            }

            if i.consume_shortcut(&self.config.wait_time_keybind) {
                self.triggers.skip_wait();
            }
        });
    }

    pub fn save_config(&self) {
        if let Err(e) = self.config.save(&self.config_path) {
            error!("Config save error: {e}");
        }
    }
}

#[derive(Default)]
pub struct Triggers {
    pub retire: bool,
    retire_pressed: Option<Instant>,
    pub skip_wait: bool,
    skip_wait_pressed: Option<Instant>,
}

impl Triggers {
    fn retire(&mut self) {
        self.retire = true;
        self.retire_pressed = Some(Instant::now());
    }

    fn skip_wait(&mut self) {
        self.skip_wait = true;
        self.skip_wait_pressed = Some(Instant::now());
    }

    fn handle_timeouts(&mut self) {
        if let Some(retire_pressed) = self.retire_pressed
            && retire_pressed.elapsed() > TRIGGER_TIMEOUT
        {
            self.retire = false;
            self.retire_pressed = None;
        }
        if let Some(skip_wait_pressed) = self.skip_wait_pressed
            && skip_wait_pressed.elapsed() > TRIGGER_TIMEOUT
        {
            self.skip_wait = false;
            self.skip_wait_pressed = None;
        }
    }
}
