// Copyright (c) 2018 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use alloc::boxed::Box;
use errno::*;
use synch::recmutex::RecursiveMutex;
use mm;

#[no_mangle]
pub extern "C" fn sys_recmutex_init(recmutex: *mut *mut RecursiveMutex) -> i32 {
	kernel_enter!("sys_recmutex_init");
	if recmutex.is_null() {
		kernel_exit!("sys_recmutex_init");
		return -EINVAL;
	}

	// Create a new boxed recursive mutex and return a pointer to the raw memory.
	let boxed_mutex = Box::new(RecursiveMutex::new());
	unsafe {
		*recmutex = Box::into_raw(boxed_mutex);
	}
	kernel_exit!("sys_recmutex_init");
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_destroy(recmutex: *mut RecursiveMutex) -> i32 {
	kernel_enter!("sys_recmutex_destroy");
	if recmutex.is_null() {
		kernel_exit!("sys_recmutex_destroy");
		return -EINVAL;
	}

	// Consume the pointer to the raw memory into a Box again
	// and drop the Box to free the associated memory.
	unsafe {
		Box::from_raw(recmutex);
	}
	kernel_exit!("sys_recmutex_destroy");
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_lock(recmutex: *mut RecursiveMutex) -> i32 {
	kernel_enter!("sys_recmutex_lock");
	if recmutex.is_null() {
		kernel_exit!("sys_recmutex_lock");
		return -EINVAL;
	}

	let mutex = unsafe { &*recmutex };
	mutex.acquire();
	kernel_exit!("sys_recmutex_lock");
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_unlock(recmutex: *mut RecursiveMutex) -> i32 {
	kernel_enter!("sys_recmutex_unlock");
	if recmutex.is_null() {
		kernel_exit!("sys_recmutex_unlock");
		return -EINVAL;
	}

	let mutex = unsafe { &*recmutex };
	mutex.release();
	kernel_exit!("sys_recmutex_unlock");
	0
}
