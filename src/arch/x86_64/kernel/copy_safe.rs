#![allow(dead_code)]
use core::ptr::{write_bytes, copy_nonoverlapping};
use core::mem::size_of;
use x86::msr::*;
use mm;
use arch::x86_64::kernel::processor;

safe_global_var!(static mut LIST: [usize;100] = [0;100]);
safe_global_var!(static SIZE: usize = 0x1000);

pub fn unsafe_storage_init() {
        let unsafe_storage = mm::unsafe_allocate(SIZE, true);
        unsafe {
                info!("Init unsafe_storage: {:#X}", unsafe_storage);
                wrmsr(IA32_KERNEL_GSBASE, unsafe_storage as u64);
                list_add(processor::readgs());
        }
}

#[inline]
pub fn is_unsafe_storage_init() -> bool {
        unsafe { (rdmsr(IA32_KERNEL_GSBASE) != 0) }
}

#[inline]
pub fn get_unsafe_storage() -> usize {
        unsafe {
                let address: usize;

                if rdmsr(IA32_KERNEL_GSBASE) == 0 {
                    return 0;
                }

	        asm!("swapgs; rdgsbase $0; swapgs" : "=r"(address) ::: "volatile");
                address
        }
}

pub fn list_add(addr: usize) {
        safe_global_var!(static mut IDX: usize = 0);
        unsafe {
                if LIST.iter().any(|v| v == &addr) {
                        return;
                }
                if IDX >= 100 {
                        error!("LIST is full!!");
                        error!(" ");
                        return;
                }
                LIST[IDX] = addr;
                IDX+=1;
        };
}

#[inline]
fn is_valid(addr: usize) -> bool {
        if addr == 0 {
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
                error!(" ");
                return;
        }

        if count > SIZE {
                error!("copy_from_safe error, too large size");
                error!(" ");
                return;
        }

        if is_valid(src as usize) {
                unsafe {
                        copy_nonoverlapping(src, get_unsafe_storage() as *mut T, count);
                }
                return;
        }
        error!("copy_from_safe error");
        error!(" ");
}

pub fn copy_to_safe<T>(dst: *mut T, count: usize) {
        if dst.is_null() {
                error!("copy_to_safe error, null pointer");
                error!(" ");
                return;
        }

        if count > SIZE {
                error!("copy_to_safe error, too large size");
                error!(" ");
                return;
        }

        if is_valid(dst as usize) {
                unsafe {
                        copy_nonoverlapping(get_unsafe_storage() as *const T, dst, count);
                }
                return;
        }
        error!("copy_to_safe error");
        error!(" ");
}

pub fn clear_unsafe_storage()
{
        unsafe { write_bytes(get_unsafe_storage() as *mut u8, 0x00, SIZE)};
}

pub fn clear_unsafe_storage2<T>(_: *const T)
{
        unsafe { write_bytes(get_unsafe_storage() as *mut u8, 0x00, size_of::<T>())};
}
