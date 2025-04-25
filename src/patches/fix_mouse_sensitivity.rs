use std::arch::x86_64::__m128;
use std::ffi::c_void;

use crate::game::Clock;

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
            We adjust the mouse sensitivity by multiplying the axis movement with a factor, that
            is inversely proportional to the frame time. This will keep the sensitivity constant,
            regardless of the FPS.

            We use the mouse sensitivity at 60 FPS (0.016 ms frame time) as a reference.
        */

        let g_root_clock = &**(ROOT_CLOCK_ADDRESS as *mut *mut Clock);
        let frame_delta_time = g_root_clock.delta_time;

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
        let original_func: usize = GET_AXIS_MOVEMENT_ADDRESS;
        let hook_func: usize = hk_get_axis_movement as *mut c_void as usize;

        let trampoline =
            libmem::hook_code(original_func, hook_func).ok_or("failed to hook function")?;

        ORIGINAL_FUNC = trampoline.callable();
    }

    println!("> Mouse sensitivity fix applied");
    Ok(())
}
