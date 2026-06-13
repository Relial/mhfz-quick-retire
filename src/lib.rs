use mimalloc::MiMalloc;

mod address;
mod config;
mod hooks;
mod plugin;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[unsafe(no_mangle)]
extern "system" fn DllMain(_hinst: u32, _fdw_reason: u32, _lpv_reserved: *mut ()) -> bool {
    true
}
