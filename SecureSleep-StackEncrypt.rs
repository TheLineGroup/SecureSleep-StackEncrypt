use std::ptr::null_mut;
use std::ffi::OsString;
use winapi::um::processthreadsapi::{GetCurrentProcessId, OpenProcess};
use winapi::um::memoryapi::{VirtualAllocEx, VirtualFreeEx};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, HANDLE};

#[repr(C)]
struct UNICODE_STRING {
    Length: u16,
    MaximumLength: u16,
    Buffer: *mut u16,
}

extern "system" {
    fn GetModuleHandleExW(dwFlags: u32, lpModuleName: *const u16, phModule: *mut HANDLE) -> i32;
}

fn get_module_handle_ex(lp_module_name: *const u16) -> Result<HANDLE, String> {
    if lp_module_name.is_null() {
        return Err("Invalid module name.".to_string());
    }

    unsafe {
        let mut h_module = null_mut();
        let result = GetModuleHandleExW(0, lp_module_name, &mut h_module as *mut HANDLE);
        if result != 0 {
            Ok(h_module)
        } else {
            Err(format!("Failed to get module handle. Error code: {}", result))
        }
    }
}

fn main() {
    let lp_module_name = OsString::from("kernel32.dll").encode_wide().collect::<Vec<u16>>();
    match get_module_handle_ex(lp_module_name.as_ptr()) {
        Ok(h_module) => {
            println!("Module handle: {:?}", h_module);
            // Free the module handle when it's no longer needed
            let result = unsafe { VirtualFreeEx(h_module, null_mut(), 0, MEM_RELEASE) };
            if result == 0 {
                eprintln!("Failed to free module handle.");
            }
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}