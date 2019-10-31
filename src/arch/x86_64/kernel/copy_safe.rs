#![allow(dead_code)]
use core::ptr::copy_nonoverlapping;
use arch::x86_64::kernel::BOOT_INFO;
use ::mm;

isolate_global_var!(pub static mut UNSAFE_STORAGE: usize = 0);
static mut list: [usize;10] = [0;10];

pub fn init() {
    let unsafe_storage = mm::unsafe_allocate(4096, true);
    unsafe {
        list[0] = BOOT_INFO as usize;

        isolation_start!();
        UNSAFE_STORAGE = unsafe_storage;
        isolation_end!();
    }
}

fn is_valid(addr: usize) -> bool {
    if (addr == 0) {
        return false;
    }
    else if unsafe{list.iter().any(|v| v == &addr)} {
        info!("addr {:#X} is valid", addr);
        return true;
    } 
    else {
        info!("addr {:#X} is invalid", addr);
        return false;
    }

}

pub fn copy_from_safe<T>(src: *const T, dst: *mut T, count: usize) {
    if src.is_null() {
        error!("copy_from_safe error");
        return;
    }

    if dst.is_null() {
        error!("copy_from_safe error");
        return;
    }

    if count > 4096 {
        error!("copy_from_safe error");
        return;
    }

    if is_valid(dst as usize) {
        unsafe {copy_nonoverlapping(src, UNSAFE_STORAGE as *mut T, count);}
        info!("copy_from_safe done");
    }
    error!("copy_from_safe error");
}

pub fn copy_to_safe<T>(src: *const T, dst: *mut T, count: usize) {
    if src.is_null() {
        error!("copy_to_safe error");
        return;
    }

    if dst.is_null() {
        error!("copy_to_safe error");
        return;
    }
    
    if count > 4096 {
        error!("copy_to_safe error");
        return;
    }

    if is_valid(src as usize) {
        unsafe {
            copy_nonoverlapping(UNSAFE_STORAGE as *const T, dst, count);
            core::ptr::write_bytes(UNSAFE_STORAGE as *mut T, 0x00, count);
        }
        info!("copy_to_safe done");
    }
    error!("copy_to_safe error");
}
