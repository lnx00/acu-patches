use std::ffi::c_void;

use windows::{
    Wdk::System::Threading::{NtQueryInformationThread, ThreadQuerySetWin32StartAddress},
    Win32::{
        Foundation::HANDLE,
        System::{
            LibraryLoader::{GetModuleHandleA, GetProcAddress},
            Threading::{
                LPTHREAD_START_ROUTINE, OpenThread, THREAD_ALL_ACCESS, THREAD_CREATE_SUSPENDED,
                THREAD_CREATION_FLAGS, TerminateThread,
            },
        },
    },
};
use windows_strings::s;

const INTEGRITY_THREAD_START_ADDRESS: usize = 0x14275DE50;

static mut ORIGINAL_CREATE_THREAD: Option<
    unsafe extern "system" fn(
        *mut c_void,
        usize,
        *mut c_void,
        *mut c_void,
        THREAD_CREATION_FLAGS,
        *mut u32,
    ) -> HANDLE,
> = None;

fn check_thread(thread_id: u32) -> Result<bool, String> {
    unsafe {
        let mut thread_start_address = 0x0;

        // Get a handle to the thread
        let thread_handle = OpenThread(THREAD_ALL_ACCESS, false, thread_id)
            .map_err(|_| "failed to open thread handle")?;

        // Query the thread start address
        let nt_status = NtQueryInformationThread(
            thread_handle,
            ThreadQuerySetWin32StartAddress,
            &mut thread_start_address as *mut usize as *mut _,
            0x8,
            std::ptr::null_mut(),
        );

        if nt_status.is_err() {
            return Err(format!(
                "failed to query thread information: {:?}",
                nt_status
            ));
        }

        if thread_start_address == INTEGRITY_THREAD_START_ADDRESS {
            TerminateThread(thread_handle, 0x0).map_err(|_| "failed to terminate thread")?;

            return Ok(true);
        }
    }

    Ok(false)
}

fn terminate_integrity_checks() -> Result<(), String> {
    let process_id = libmem::get_process().unwrap().pid;
    let thread_list = libmem::enum_threads().ok_or("failed to enumerate threads")?;

    // Check all thread of the current process
    for thread in thread_list {
        if thread.owner_pid == process_id {
            let check_result = check_thread(thread.tid);
            match check_result {
                Ok(true) => {
                    println!("Terminated integrity check thread {}", thread.tid);
                }

                Err(e) => {
                    return Err(format!("Error checking thread {}: {}", thread.tid, e));
                }

                _ => {}
            }
        }
    }

    Ok(())
}

unsafe extern "system" fn empty_thread(_: *mut c_void) -> u32 {
    return 0;
}

extern "system" fn hk_create_thread(
    lp_thread_attributes: *mut c_void,
    dw_stack_size: usize,
    mut lp_start_address: *mut c_void,
    lp_parameter: *mut c_void,
    dw_creation_flags: THREAD_CREATION_FLAGS,
    lp_thread_id: *mut u32,
) -> HANDLE {
    unsafe {
        if lp_start_address as usize == INTEGRITY_THREAD_START_ADDRESS {
            println!("CreateThread: preventing integrity check thread creation");
            lp_start_address = empty_thread as *mut c_void;
        }

        return ORIGINAL_CREATE_THREAD.unwrap()(
            lp_thread_attributes,
            dw_stack_size,
            lp_start_address,
            lp_parameter,
            dw_creation_flags,
            lp_thread_id,
        );
    }
}

fn hook_integrity_checks() -> Result<(), String> {
    unsafe {
        let kernel32_handle = GetModuleHandleA(s!("kernel32.dll"))
            .map_err(|_| "failed to get kernel32.dll handle")?;

        let fp_create_thread = GetProcAddress(kernel32_handle, s!("CreateThread"))
            .ok_or("failed to get CreateThread address")?;

        let create_thread_address = fp_create_thread as *mut c_void as usize;
        let hook_address = hk_create_thread as *mut c_void as usize;

        let trampoline = libmem::hook_code(create_thread_address, hook_address)
            .ok_or("failed to hook CreateThread")?;

        ORIGINAL_CREATE_THREAD = trampoline.callable();
    }

    Ok(())
}

pub fn disable_integrity_checks() -> Result<(), String> {
    println!("Disabling integrity checks...");

    hook_integrity_checks()?;
    terminate_integrity_checks()?;

    Ok(())
}
