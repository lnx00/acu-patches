use crate::utils;

pub fn disable_camera_smoothing() -> Result<(), String> {
    /*
        The game already has logic for disabling camera smoothing, but it is usually not possible
        to enable it. We can however patch the condition that checks if mouse smoothing is
        disable to always run, which uses the mouse movement directly instead of lerping it.
    */

    let game_module = libmem::find_module("ACU.exe").ok_or("game module not found")?;
    let smooth_check = unsafe {
        libmem::sig_scan("74 ? 41 8B 06 41 89 85", game_module.base, game_module.size)
            .ok_or("signature not found")?
    };

    let patch_bytes: [u8; 2] = [0x90, 0x90];
    utils::patch_bytes(smooth_check, &patch_bytes)?;

    println!("> Camera smoothing patch applied");
    Ok(())
}
