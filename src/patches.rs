pub mod disable_camera_smoothing;
pub mod disable_integrity_checks;

fn wait_for_game() {
    loop {
        let game_setting = unsafe { libmem::read_memory::<usize>(0x145217348) };
        if game_setting != 0 {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Extra wait due to another integrity check while launching
    std::thread::sleep(std::time::Duration::from_secs(10));
}

pub fn run_all_patches() -> Result<(), String> {
    disable_integrity_checks::disable_integrity_checks()?;

    println!("Waiting for the game to be ready...");
    wait_for_game();
    println!("Game is ready! Applying patches...");

    disable_camera_smoothing::disable_camera_smoothing()?;

    Ok(())
}
