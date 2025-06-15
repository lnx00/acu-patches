use std::{thread, time::{Duration, Instant}};

pub mod integrity;

#[repr(C)]
pub struct Clock {
    pad_0000: [u8; 0x18], // 0x00
    pub delta_time: f32,  // 0x18
}

pub fn disable_integrity_checks() -> Result<(), String> {
    integrity::IntegrityHook::inst().apply()?;
    integrity::terminate_integrity_checks()?;

    Ok(())
}

pub fn cleanup_integrity_checks() -> Result<(), String> {
    integrity::IntegrityHook::inst().cleanup()?;

    Ok(())
}

/// Blocks the caller until the game's memory is ready to be patched.
pub fn wait_for_game() {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(15);

    while !integrity::was_disabled() {
        if start_time.elapsed() >= timeout {
            println!("Timeout while waiting for integrity check termination! Assuming that they're already disabled...");
            return;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    thread::sleep(std::time::Duration::from_secs(3));
}
