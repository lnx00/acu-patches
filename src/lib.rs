use std::thread;
use windows::Win32::{
    Foundation::{HINSTANCE, HMODULE},
    System::{
        LibraryLoader::FreeLibraryAndExitThread,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
};

mod patches;
mod platform;
mod utils;

struct SendWrapper<T>(T);
unsafe impl<T> Send for SendWrapper<T> {}

pub const VK_F11: i32 = 0x7A;

fn apply_patches() {
    if let Err(e) = patches::run_all_patches() {
        platform::msg_box(
            &format!("Failed to apply patches: {}", e),
            "Error",
            platform::MsgBoxType::Error,
        );
        
        eprintln!("Failed to apply patches: {}", e);
    } else {
        println!("Patches applied successfully!");
    }
}

fn main_thread(hinst_dll: SendWrapper<HINSTANCE>) {
    unsafe {
        platform::attach_console();
        println!("Console attached! Press F11 to exit.");

        apply_patches();

        while !platform::is_button_down(VK_F11) {
            thread::sleep(std::time::Duration::from_millis(100));
        }

        platform::detach_console();
        FreeLibraryAndExitThread(HMODULE(hinst_dll.0.0), 0);
    }
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
