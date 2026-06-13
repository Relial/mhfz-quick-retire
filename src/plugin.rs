use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use abi_stable::std_types::RStr;
use bunny_ui::ui::BunnyUi;
use bunny_ui::widgets::drag_value::DragValue;
use bunny_ui::widgets::shortcut_button::ShortcutButton;
use ilhook::x86::ClosureHookPoint;
use tracing::error;

use crate::address::{Addresses, MainDllInfo};
use crate::config::{Config, DeathRetireKind};
use crate::hooks;

const TRIGGER_ACTIVE_TIME: Duration = Duration::from_secs(1);

pub static mut STATE: Option<State> = None;

pub static RETIRE: Trigger = Trigger::new();
pub static SKIP_WAIT: Trigger = Trigger::new();

pub struct Trigger(AtomicBool);

impl Trigger {
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn set(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.0.store(false, Ordering::Relaxed);
    }

    pub fn trigger(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
struct Triggers {
    retire: Option<Instant>,
    skip_wait: Option<Instant>,
}

impl Triggers {
    fn retire(&mut self) {
        self.retire = Some(Instant::now());
        RETIRE.set();
    }

    fn skip_wait(&mut self) {
        self.skip_wait = Some(Instant::now());
        SKIP_WAIT.set();
    }

    fn reset(&mut self) {
        if let Some(set) = self.retire
            && set.elapsed() > TRIGGER_ACTIVE_TIME
        {
            self.retire = None;
            RETIRE.reset();
        }

        if let Some(set) = self.skip_wait
            && set.elapsed() > TRIGGER_ACTIVE_TIME
        {
            self.skip_wait = None;
            SKIP_WAIT.reset();
        }
    }
}

pub struct State<'a> {
    config_path: PathBuf,
    addresses: Addresses,
    hooks: Vec<ClosureHookPoint<'a>>,
    pub config: Config,
    triggers: Triggers,
}

impl<'a> State<'_> {
    fn menu(&'a mut self, ui: &mut BunnyUi<'a>) {
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

    fn ui(&mut self, ui: &mut BunnyUi) {
        self.triggers.reset();

        ui.input_mut(|i| {
            if i.consume_shortcut(&self.config.retire_keybind) {
                self.triggers.retire();
            }

            if i.consume_shortcut(&self.config.wait_time_keybind) {
                self.triggers.skip_wait();
            }
        });
    }

    fn save_config(&self) {
        if let Err(e) = self.config.save(&self.config_path) {
            error!("Config save error: {e}");
        }
    }
}

// Called once when the plugin is loaded
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn init(config_dir_path: RStr<'_>, main_dll_info: MainDllInfo) -> bool {
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .init();

    let addresses = Addresses::new(main_dll_info);
    let hooks = hooks::init(&addresses).unwrap_or_default();
    let mut config_path = PathBuf::from(config_dir_path.as_str());
    config_path.push(format!("{}.toml", env!("CARGO_PKG_NAME")));

    let config = Config::load(&config_path).unwrap_or_default();

    let state = State {
        addresses,
        hooks,
        config,
        config_path,
        triggers: Triggers::default(),
    };
    unsafe { STATE = Some(state) };

    true
}

// Called every frame when the plugin's dropdown in the manager window is open
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn menu(ui: &mut BunnyUi) {
    if let Some(state) = unsafe { &mut STATE } {
        state.menu(ui);
    }
}

// Called every frame
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn ui(ui: &mut BunnyUi) {
    if let Some(state) = unsafe { &mut STATE } {
        state.ui(ui);
    }
}

// Called once when the plugin is manually disabled and when the game shuts down
#[unsafe(no_mangle)]
pub extern "C" fn unload() {
    unsafe {
        STATE = None;
    }
}

// Called once per user defined autosave interval, and before the unload function
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn save() {
    if let Some(state) = unsafe { &STATE } {
        state.save_config();
    }
}
