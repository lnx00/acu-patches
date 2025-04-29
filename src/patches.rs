use std::sync::MutexGuard;

use disable_camera_smoothing::DisableCameraSmoothing;
use fix_mouse_sensitivity::MouseSensitivityFix;

pub mod disable_camera_smoothing;
pub mod disable_integrity_checks;
pub mod fix_mouse_sensitivity;

pub trait Feature {
    fn inst() -> MutexGuard<'static, Self>;

    fn enable(&mut self) -> Result<(), String>;
    fn disable(&mut self) -> Result<(), String>;
}

pub fn run_all_patches() -> Result<(), String> {
    disable_integrity_checks::disable_integrity_checks()?;

    println!("Waiting for the game to be ready...");
    while !disable_integrity_checks::is_integrity_checks_disabled() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Game is ready! Applying patches...");

    DisableCameraSmoothing::inst().enable()?;
    MouseSensitivityFix::inst().enable()?;

    Ok(())
}

pub fn disable_all_patches() -> Result<(), String> {
    DisableCameraSmoothing::inst().disable()?;
    MouseSensitivityFix::inst().disable()?;

    Ok(())
}
