use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_strings::PCSTR;

#[allow(dead_code)]
pub enum MsgBoxType {
    Info,
    Warning,
    Error,
}

pub fn msg_box(msg: &str, title: &str, box_type: MsgBoxType) {
    let icon = match box_type {
        MsgBoxType::Info => MB_ICONINFORMATION,
        MsgBoxType::Warning => MB_ICONWARNING,
        MsgBoxType::Error => MB_ICONERROR,
    };

    unsafe {
        MessageBoxA(
            None,
            PCSTR(format!("{}\0", msg).as_ptr()),
            PCSTR(format!("{}\0", title).as_ptr()),
            MB_OK | icon,
        );
    }
}

pub fn attach_console() {
    let _ = unsafe { AllocConsole() };
}

pub fn detach_console() {
    let _ = unsafe { FreeConsole() };
}

pub fn is_button_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}
