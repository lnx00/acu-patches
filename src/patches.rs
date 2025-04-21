pub mod disable_integrity_checks;
pub mod disable_camera_smoothing;

pub fn run_all_patches() -> Result<(), String> {
    disable_integrity_checks::disable_integrity_checks()?;
    disable_camera_smoothing::disable_camera_smoothing()?;

    Ok(())
}