pub mod disable_camera_smoothing;
pub mod disable_integrity_checks;

pub fn run_all_patches() -> Result<(), String> {
    disable_integrity_checks::disable_integrity_checks()?;

    // Wait until the game is ready
    println!("Waiting for the game to be ready...");
    std::thread::sleep(std::time::Duration::from_secs(20));

    disable_camera_smoothing::disable_camera_smoothing()?;

    Ok(())
}
