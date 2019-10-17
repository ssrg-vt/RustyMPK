#![allow(dead_code)]

use arch::x86_64::mm::paging;
use arch::x86_64::mm::paging::{PageSize, BasePageSize};
use arch::x86_64::kernel::processor;

const EINVAL: i32 = 22;
const ENOSYS: i32 = 38;

pub enum MpkPerm {
    MpkRw,
    MpkRo,
    MpkNone
}

#[inline]
fn rdpkru() -> u32 {

    let val: u32;
    unsafe {
        asm!("xor %ecx, %ecx;
              rdpkru;
              movl %eax, $0"
             : "=r"(val)
             :
             : "eax", "edx", "ecx"
             : "volatile");
    }
    val
}

#[inline]
fn wrpkru(val: u32) {

    unsafe {
        asm!("mov $0, %eax;
              xor %ecx, %ecx;
              xor %edx, %edx;
              wrpkru;
              lfence"
             :
             : "r"(val)
             : "eax", "ecx", "edx"
             : "volatile");
    }
}

pub fn mpk_swap_pkru(new_pkru: u32) -> u32 {

    if processor::supports_ospke() == true {
        return 0;
    }

    let old_pkru: u32;
    old_pkru = rdpkru();
    wrpkru(new_pkru);
    return old_pkru;
}

fn pkru_set_ro(key: u8, val: &mut u32) -> i32 {

    if key > 15
    {
        return -EINVAL;
    }

    *val &= !(1 << (key * 2));
    *val |= 1 << ((key*2) + 1);

    return 0;
}

fn pkru_set_rw(key: u8, val: &mut u32) -> i32 {

    if key > 15
    {
        return -EINVAL;
    }

    *val &= !(1 << (key*2));
    *val &= !(1 << ((key*2) + 1));

    return 0;
}

fn pkru_set_no_access(key: u8, val: &mut u32) -> i32 {

    if key > 15
    {
        return -EINVAL;
    }

    *val |= 1 << (key * 2);
    *val |= 1 << ((key * 2) + 1);

    return 0;
}

pub fn mpk_mem_set_key<S: PageSize>(mut addr: usize, mut size: usize, key: u8) -> i32 {

    if processor::supports_ospke() == false {
        return -ENOSYS;
    }

    if key > 15
    {
        return -EINVAL;
    }

    /* FIXME */
    /* If needed floor addr to the nearest page */
    addr = (addr) & !(S::SIZE-1);
    /* If needed ceil [addr + size[ to the nearest page */
    if ((S::SIZE-1)&size) > 0 {
        size = (size + S::SIZE) & !(S::SIZE-1);
    }

    let mut count :usize = size/S::SIZE;
    if size%S::SIZE > 0
    {
        count = count + 1;
    }
    return paging::set_pkey::<S>(addr, count, key);
}

pub fn mpk_set_perm(key: u8, perm: MpkPerm) -> i32 {

    if processor::supports_ospke() == false {
        return -ENOSYS;
    }

    let mut pkru: u32;
    pkru = rdpkru();

    match perm {
        MpkPerm::MpkRw => {
            pkru_set_rw(key, &mut pkru);
        }

        MpkPerm::MpkRo => {
            pkru_set_ro(key, &mut pkru);
        }

        MpkPerm::MpkNone => {
            pkru_set_no_access(key, &mut pkru);
        }
    }

    wrpkru(pkru);
    return 0;
}

pub fn mpk_clear_pkru() {

    if processor::supports_ospke() == false {
        return;
    }

    wrpkru(0x0);
}

/* Return the PKRU value */
pub fn mpk_get_pkru() -> u32 {

    if processor::supports_ospke() == false {
        return 0;
    }

    return rdpkru();
}

/* Set the pkru value to 'val' */
pub fn mpk_set_pkru(val: u32) {

    if processor::supports_ospke() == true {
        wrpkru(val);
    }
}