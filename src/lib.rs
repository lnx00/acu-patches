use config::Config;
use std::thread;
use windows::Win32::{
    Foundation::{HINSTANCE, HMODULE},
    System::{
        LibraryLoader::FreeLibraryAndExitThread,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
};

use crate::plugin::{make_version, ACUPluginInfo, ACUPluginLoaderInterface};

mod config;
mod game;
mod patches;
mod platform;
mod utils;
mod plugin;

struct SendWrapper<T>(T);
unsafe impl<T> Send for SendWrapper<T> {}

const PKG_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

const VK_F11: i32 = 0x7A;

fn run() -> Result<(), String> {
    println!("Disabling integrity checks...");
    game::disable_integrity_checks()?;

    println!("Integrity checks disabled! Waiting for the game...");
    game::wait_for_game();

    println!("Game ready! Applying patches...");
    patches::run_all_patches()?;

    println!("All patches applied successfully! Press F11 to unload.");
    while !platform::is_button_down(VK_F11) {
        thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("Unloading patches...");
    patches::disable_all_patches()?;
    game::cleanup_integrity_checks()?;

    Ok(())
}

fn main_thread(dll_module: SendWrapper<HINSTANCE>) {
    let config = Config::read("./plugins/acu_patches.toml").unwrap_or_default();

    // Attach console window
    if config.show_console {
        let title = format!("ACU Patches v{} by lnx00", PKG_VERSION.unwrap_or("?.?.?"));
        platform::attach_console(&title);
    }

    // Run main logic
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        platform::msg_box(&e, "Error", platform::MsgBoxType::Error);
    }

    // Detach console
    if config.show_console {
        platform::detach_console();
    }

    unsafe { FreeLibraryAndExitThread(HMODULE(dll_module.0.0), 0) };
}

extern "C" fn init_patches(_plugin_loader: &ACUPluginLoaderInterface) -> bool {
    patches::run_all_patches().is_ok()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ACUPluginStart(
    _plugin_loader: &ACUPluginLoaderInterface,
    your_plugin_info_out: &mut ACUPluginInfo,
) -> bool {
    your_plugin_info_out.m_plugin_api_version = make_version(0, 9, 1, 0);
    your_plugin_info_out.m_plugin_version = make_version(0, 4, 0, 0);

    your_plugin_info_out.m_init_stage_when_code_patches_are_safe_to_apply = Some(init_patches);

    true
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(_dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            /*let safe_dll_module = SendWrapper(dll_module);
            thread::spawn(move || {
                main_thread(safe_dll_module);
            });*/
        }

        DLL_PROCESS_DETACH => (),

        _ => (),
    }

    true
}
