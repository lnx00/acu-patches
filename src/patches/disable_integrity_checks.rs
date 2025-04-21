use windows::{
    Wdk::System::Threading::{NtQueryInformationThread, ThreadQuerySetWin32StartAddress},
    Win32::System::{
        LibraryLoader::GetModuleHandleA,
        Threading::{OpenThread, THREAD_ALL_ACCESS, TerminateThread},
    },
};
use windows_strings::s;

const INTEGRITY_THREAD_START_ADDRESS: usize = 0x14275DE50;

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

pub fn disable_integrity_checks() -> Result<(), String> {
    println!("Disabling integrity checks...");

    terminate_integrity_checks()?;

    Ok(())
}
