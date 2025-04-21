use crate::utils;

pub fn disable_camera_smoothing() -> Result<(), String> {
    println!("Disabling camera smoothing...");

    unsafe {
        let game_module = libmem::find_module("ACU.exe").ok_or("game module not found")?;
        let smooth_check =
            libmem::sig_scan("74 ? 41 8B 06 41 89 85", game_module.base, game_module.size)
                .ok_or("signature not found")?;

        let patch_bytes: [u8; 2] = [0x90, 0x90];
        utils::patch_bytes(smooth_check, &patch_bytes)?;
    }

    Ok(())
}
