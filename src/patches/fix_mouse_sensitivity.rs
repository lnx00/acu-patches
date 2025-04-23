use std::ffi::c_void;

const ROOT_CLOCK_ADDRESS: usize = 0x14525D9D0;
const SENSITIVITY_FACTOR_ADDRESS: usize = 0x142E77F18;

const REFERENCE_FRAME_TIME: f32 = 0.016; // 60 FPS
const DEFAULT_SENSITIVITY_FACTOR: f32 = 0.15;

static mut ORIGINAL_FUNC: Option<
    unsafe extern "system" fn(
        a1: usize,
        a2: *mut f32,
        a3: *mut f32,
        a4: *mut f32,
        a5: *mut f32,
        a6: usize,
        a7: *mut u8,
        a8: usize,
        a9: usize,
        out_sens_non_zero: *mut bool,
        a11: usize,
        a12: usize,
        a13: f32,
        a14: usize,
        a15: f32,
    ) -> u8,
> = None;

/// This function handles the mouse sensitivity calculations
extern "system" fn hk_handle_input(
    a1: usize,
    a2: *mut f32,
    a3: *mut f32,
    a4: *mut f32,
    a5: *mut f32,
    a6: usize,
    a7: *mut u8,
    a8: usize,
    a9: usize,
    out_sens_non_zero: *mut bool,
    a11: usize,
    a12: usize,
    a13: f32,
    a14: usize,
    a15: f32,
) -> u8 {
    unsafe {
        //println!("hook called");

        // game root clock is stored at 0x14525D9D0
        let g_root_clock = *(ROOT_CLOCK_ADDRESS as *mut usize) as *mut u8;
        let frame_delta_time = *(g_root_clock.offset(0x18) as *mut f32);

        // read the f32 value stored at 0x142E77F18. This is the mouse sensitivity.
        //let sensitivity_factor = *(0x142E77F18 as *mut f32);
        let p_sensitivity_factor = SENSITIVITY_FACTOR_ADDRESS as *mut f32;
        if !p_sensitivity_factor.is_null() {
            let sensitivity_factor = *p_sensitivity_factor;
            let new_factor = (REFERENCE_FRAME_TIME / frame_delta_time) * DEFAULT_SENSITIVITY_FACTOR;

            // write the new value to the memory address that p_sensitivity_factor points to.
            p_sensitivity_factor.write(new_factor);

            //*(0x142E77F18 as *mut f32) = new_factor;
            println!(
                "frame delta time: {}, old factor: {}, new factor: {}",
                frame_delta_time, sensitivity_factor, new_factor
            );
        }

        return ORIGINAL_FUNC.unwrap()(
            a1,
            a2,
            a3,
            a4,
            a5,
            a6,
            a7,
            a8,
            a9,
            out_sens_non_zero,
            a11,
            a12,
            a13,
            a14,
            a15,
        );
    }
}

pub fn fix_mouse_sensitivity() -> Result<(), String> {
    unsafe {
        // Allow changes to the sensitivity factor
        libmem::prot_memory(SENSITIVITY_FACTOR_ADDRESS, 4, libmem::Prot::XRW)
            .ok_or("failed to change protection")?;

        // Hook the function responsible for mouse sensitivity calculations
        let original_func: usize = 0x141F66320;
        let hook_func: usize = hk_handle_input as *mut c_void as usize;

        let trampoline =
            libmem::hook_code(original_func, hook_func).ok_or("failed to hook function")?;

        ORIGINAL_FUNC = trampoline.callable();
    }

    Ok(())
}
