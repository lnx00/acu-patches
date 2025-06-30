use std::sync::MutexGuard;

use disable_camera_smoothing::DisableCameraSmoothing;
use mouse_sensitivity_fix::MouseSensitivityFix;

pub mod disable_camera_smoothing;
pub mod mouse_sensitivity_fix;

pub trait Patch {
    fn inst() -> MutexGuard<'static, Self>;

    fn apply(&mut self) -> Result<(), String>;
    fn revert(&mut self) -> Result<(), String>;
}

pub fn run_all_patches() -> Result<(), String> {
    DisableCameraSmoothing::inst().apply()?;
    log::info!("- DisableCameraSmoothing applied");

    MouseSensitivityFix::inst().apply()?;
    log::info!("- MouseSensitivityFix applied");

    Ok(())
}

pub fn disable_all_patches() -> Result<(), String> {
    DisableCameraSmoothing::inst().revert()?;
    log::info!("- DisableCameraSmoothing reverted");

    MouseSensitivityFix::inst().revert()?;
    log::info!("- DisableCameraSmoothing reverted");

    Ok(())
}
