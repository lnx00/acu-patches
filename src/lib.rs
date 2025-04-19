use libmem::Prot;
use std::thread;
use windows::Win32::{
    Foundation::{HINSTANCE, HMODULE},
    System::{
        LibraryLoader::FreeLibraryAndExitThread,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
    UI::WindowsAndMessaging::{MB_OK, MessageBoxA},
};
use windows_strings::*;

struct SendWrapper<T>(T);
unsafe impl<T> Send for SendWrapper<T> {}

fn patch_disable_smoothing() -> Result<(), String> {
    unsafe {
        let game_module = libmem::find_module("ACU.exe").ok_or("game module not found")?;
        let smooth_check =
            libmem::sig_scan("74 ? 41 8B 06 41 89 85", game_module.base, game_module.size)
                .ok_or("signature not found")?;

        let old_protect =
            libmem::prot_memory(smooth_check, 0, Prot::XRW).ok_or("failed to change protection")?;

        let patch_bytes: [u8; 2] = [0x90, 0x90];
        libmem::write_memory(smooth_check, &patch_bytes);

        libmem::prot_memory(smooth_check, 0, old_protect).ok_or("failed to restore protection")?;
    }

    Ok(())
}

fn apply_patches() {
    unsafe {
        if let Err(e) = patch_disable_smoothing() {
            let err_msg = PCSTR(format!("Patch failed: {}\0", e).as_ptr());
            MessageBoxA(None, err_msg, s!("Error"), MB_OK);
        } else {
            MessageBoxA(
                None,
                s!("Patch applied successfully!"),
                s!("Success"),
                MB_OK,
            );
        }
    }
}

fn main_thread(hinst_dll: SendWrapper<HINSTANCE>) {
    unsafe {
        apply_patches();
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
