use bunny_plugin::{PluginContext, PluginInfo, bunny_ui::ui::BunnyUi, hook_cell::HookCell};

use crate::{
    address::Addresses,
    hooks::{on_quest_end, on_quest_update},
    ui::State,
};

const PLUGIN_NAME: &str = "Quick Retire";
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

pub static STATE: HookCell<State> = HookCell::new();

// Called once when the plugin is loaded
#[unsafe(no_mangle)]
pub extern "C" fn init(context: PluginContext) -> PluginInfo {
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(context.log_level())
        .init();

    let addresses = Addresses::new(context.mhfo_info());
    let state = State::new(context, addresses);
    let state_res = STATE.set(state);
    let mut info = PluginInfo::new(PLUGIN_NAME, PLUGIN_VERSION)
        .with_quest_hook(on_quest_update)
        .with_quest_ending_hook(on_quest_end);

    if state_res.is_err() {
        info = info.with_init_fail("State/Hooks init failed: HookCell was already initialized");
    }
    info
}

// Called every frame when the plugin's dropdown in the manager window is open
#[unsafe(no_mangle)]
pub extern "C" fn menu(ui: &mut BunnyUi) {
    let state = unsafe { STATE.get_unchecked_mut() };
    state.menu(ui);
}

// Called every frame
#[unsafe(no_mangle)]
pub extern "C" fn ui(ui: &mut BunnyUi) {
    let state = unsafe { STATE.get_unchecked_mut() };
    state.ui(ui);
}

// Called once per user defined autosave interval, and when the plugin is manually disabled by the user or the game is closed
#[unsafe(no_mangle)]
pub extern "C" fn save() {
    if let Some(state) = STATE.get() {
        state.save_config();
    }
}

// Called when the plugin is manually disabled by the user
pub fn unload() {
    unsafe {
        STATE.drop();
    }
}
