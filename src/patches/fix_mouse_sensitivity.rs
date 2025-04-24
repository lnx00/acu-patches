use std::arch::x86_64::__m128;
use std::ffi::c_void;

const ROOT_CLOCK_ADDRESS: usize = 0x14525D9D0;
const GET_AXIS_MOVEMENT_ADDRESS: usize = 0x141F6A320;

const REFERENCE_FRAME_TIME: f32 = 0.016; // 60 FPS

#[allow(improper_ctypes_definitions)]
static mut ORIGINAL_FUNC: Option<
    unsafe extern "system" fn(
        a1: i64,
        a2: i64,
        a3: *mut f32,
        a4: *mut i64,
        a5: *mut i64,
        a6: *mut f32,
        invert_factor: f32,
        a8: f32,
        a9: f32,
    ) -> __m128,
> = None;

/// This function handles the mouse sensitivity calculations
#[allow(improper_ctypes_definitions)]
extern "fastcall" fn hk_get_axis_movement(
    a1: i64,
    a2: i64,
    a3: *mut f32,
    a4: *mut i64,
    a5: *mut i64,
    a6: *mut f32,
    invert_factor: f32,
    a8: f32,
    a9: f32,
) -> __m128 {
    unsafe {
        /*
            We adjust the mouse sensitivity by calculating a factor based on the frame delta time.
            We take the mouse sensitivity at 60 FPS (0.016 ms) as a reference and scale axis movement
            calculation accordingly. This will result in the same mouse sensitivity at any FPS.
        */

        // game root clock is stored at 0x14525D9D0
        let g_root_clock = *(ROOT_CLOCK_ADDRESS as *mut usize) as *mut u8;
        let frame_delta_time = *(g_root_clock.offset(0x18) as *mut f32);

        let new_factor = REFERENCE_FRAME_TIME / frame_delta_time;

        /*println!(
            "frame delta time: {}, sensitivity factor: {}",
            frame_delta_time, new_factor
        );*/

        return ORIGINAL_FUNC.unwrap()(a1, a2, a3, a4, a5, a6, invert_factor * new_factor, a8, a9);
    }
}

/// Fixes the mouse sensitivity being tied to the FPS.
pub fn fix_mouse_sensitivity() -> Result<(), String> {
    unsafe {
        // Hook the function responsible for mouse sensitivity calculations
        let original_func: usize = GET_AXIS_MOVEMENT_ADDRESS;
        let hook_func: usize = hk_get_axis_movement as *mut c_void as usize;

        let trampoline =
            libmem::hook_code(original_func, hook_func).ok_or("failed to hook function")?;

        ORIGINAL_FUNC = trampoline.callable();
    }

    println!("> Mouse sensitivity fixed");
    Ok(())
}
