use std::ptr::null_mut;
use winapi::um::processthreadsapi::{GetCurrentProcessId, OpenProcess};
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree, ReadProcessMemory, WriteProcessMemory};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_WRITE};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use rand::{Rng, rngs::ThreadRng};
use aes::Aes128;
use aes::cipher::{NewCipher, BlockEncrypt, BlockDecrypt, KeyInit};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use aes::cipher::generic_array::GenericArray;
use std::thread;
use std::time::Duration;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

fn main() {
    custom_sleep(Duration::from_secs(5)); // Custom sleep for 5 seconds
}

fn custom_sleep(duration: Duration) {
    let process_id = unsafe { GetCurrentProcessId() };
    let process_handle = unsafe { 
        OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_WRITE, 0, process_id) 
    };

    if process_handle.is_null() {
        eprintln!("Failed to open process.");
        return;
    }

    // Allocate memory in the current process - demonstration purposes
    let base_address = unsafe {
        VirtualAlloc(null_mut(), 4096, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE)
    };

    if base_address.is_null() {
        eprintln!("Failed to allocate memory.");
        unsafe { CloseHandle(process_handle); }
        return;
    }

    // Encrypt stack data
    let key = GenericArray::from_slice(b"an example very very secret key.");
    let iv = GenericArray::from_slice(b"unique initialization vector");
    let cipher = Aes128Cbc::new(key, iv);

    let mut stack_data: Vec<u8> = vec![0; 4096];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut stack_data);
    let ciphertext = cipher.encrypt_vec(&stack_data);

    // Write encrypted stack data to memory
    let mut bytes_written: usize = 0;
    unsafe {
        WriteProcessMemory(process_handle, base_address, ciphertext.as_ptr() as _, 4096, &mut bytes_written);
    }

    // Sleep for the specified duration
    thread::sleep(duration);

    // Read encrypted stack data
    let mut encrypted_stack_data: Vec<u8> = vec![0; 4096];
    let mut bytes_read: usize = 0;
    unsafe {
        ReadProcessMemory(process_handle, base_address, encrypted_stack_data.as_mut_ptr() as _, 4096, &mut bytes_read);
    }

    // Decrypt stack data after sleep
    let decrypted_stack_data = cipher.decrypt_vec(&encrypted_stack_data).expect("Decryption error");

    // Cleanup
    unsafe {
        VirtualFree(base_address, 0, MEM_RELEASE);
        CloseHandle(process_handle);
    }
}
