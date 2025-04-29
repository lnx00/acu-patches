use std::thread;
use windows::Win32::{
    Foundation::{HINSTANCE, HMODULE},
    System::{
        LibraryLoader::FreeLibraryAndExitThread,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
};

mod game;
mod patches;
mod platform;
mod utils;

struct SendWrapper<T>(T);
unsafe impl<T> Send for SendWrapper<T> {}

pub const VK_F11: i32 = 0x7A;

fn run() -> Result<(), String> {
    platform::attach_console();
    patches::run_all_patches()?;

    while !platform::is_button_down(VK_F11) {
        thread::sleep(std::time::Duration::from_millis(100));
    }

    patches::disable_all_patches()?;
    platform::detach_console();

    Ok(())
}

fn main_thread(dll_module: SendWrapper<HINSTANCE>) {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        platform::msg_box(&e, "Error", platform::MsgBoxType::Error);
    }

    unsafe { FreeLibraryAndExitThread(HMODULE(dll_module.0.0), 0) };
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            let safe_dll_module = SendWrapper(dll_module);
            thread::spawn(move || {
                main_thread(safe_dll_module);
            });
        }

        DLL_PROCESS_DETACH => (),

        _ => (),
    }

    true
}
