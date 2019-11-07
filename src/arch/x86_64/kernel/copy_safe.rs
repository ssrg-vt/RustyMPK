#![allow(dead_code)]
use core::ptr::{write_bytes, copy_nonoverlapping};
use arch::x86_64::kernel::BOOT_INFO;
use arch;
use x86::msr::*;
use mm;

isolate_global_var!(pub static mut UNSAFE_STORAGE: usize = 0);
static mut LIST: [usize;10] = [0;10];
static SIZE: usize = 40960;
pub fn init() {
    //info!("copy_safe init");
    let unsafe_storage = mm::unsafe_allocate(SIZE, true);
   //arch::x86_64::mm::paging::print_page_table_entry::<arch::x86_64::mm::paging::BasePageSize>(unsafe_storage);
    unsafe {
        //LIST[0] = BOOT_INFO as usize;
        UNSAFE_STORAGE = unsafe_storage;
        //write_bytes(UNSAFE_STORAGE as *mut u8, 0x00, SIZE);
        wrmsr(IA32_KERNEL_GSBASE, UNSAFE_STORAGE as u64);
        //info!("UNSAFE_STORAGE: {:#X}",UNSAFE_STORAGE);
        //arch::x86_64::mm::paging::print_page_table_entry::<arch::x86_64::mm::paging::BasePageSize>(UNSAFE_STORAGE);
    }
}

pub fn list_add(addr: usize) {
    static mut idx: usize = 0;
    unsafe {
        LIST[idx] = addr;
        idx+=1;
    };
}

#[inline]
fn is_valid(addr: usize) -> bool {
    if (addr == 0) {
        return false;
    }
    else if unsafe{LIST.iter().any(|v| v == &addr)} {
        //info!("addr {:#X} is valid", addr);
        return true;
    } 
    else {
        //info!("addr {:#X} is invalid", addr);
        return false;
    }

}

pub fn copy_from_safe<T>(src: *const T, count: usize) {
    if src.is_null() {
        error!("copy_from_safe error, null pointer");
        return;
    }

    if count > SIZE {
        error!("copy_from_safe error, too large size");
        return;
    }

    if is_valid(src as usize) {
        unsafe {
            copy_nonoverlapping(src, UNSAFE_STORAGE as *mut T, count);
        }
        return;
    }
    error!("copy_from_safe error");
}

pub fn copy_to_safe<T>(dst: *mut T, count: usize) {
    if dst.is_null() {
        error!("copy_to_safe error, null pointer");
        return;
    }
    
    if count > SIZE {
        error!("copy_to_safe error, too large size");
        return;
    }

    if is_valid(dst as usize) {
        unsafe {
            copy_nonoverlapping(UNSAFE_STORAGE as *const T, dst, count);
        }
        return;
    }
    error!("copy_to_safe error");
}

pub fn clear_unsafe_storage()
{
    unsafe { write_bytes(UNSAFE_STORAGE as *mut u8, 0x00, SIZE)};
}