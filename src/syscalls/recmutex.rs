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
fn __sys_recmutex_init(recmutex: *mut *mut RecursiveMutex) -> i32 {
	if recmutex.is_null() {
		return -EINVAL;
	}

	// Create a new boxed recursive mutex and return a pointer to the raw memory.
	let boxed_mutex = Box::new(RecursiveMutex::new());
	let temp = Box::into_raw(boxed_mutex);
	unsafe {
		isolation_start!();
		*recmutex = temp;
		isolation_end!();
	}
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_init(recmutex: *mut *mut RecursiveMutex) -> i32 {
	let ret = kernel_function!(__sys_recmutex_init(recmutex));
	return ret;
}

#[no_mangle]
fn __sys_recmutex_destroy(recmutex: *mut RecursiveMutex) -> i32 {
	if recmutex.is_null() {
		return -EINVAL;
	}

	// Consume the pointer to the raw memory into a Box again
	// and drop the Box to free the associated memory.
	/*unsafe {
		isolate_function_strong!(Box::from_raw(recmutex));
	}*/
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_destroy(recmutex: *mut RecursiveMutex) -> i32 {
	let ret = kernel_function!(__sys_recmutex_destroy(recmutex));
	return ret;
}

#[no_mangle]
fn __sys_recmutex_lock(recmutex: *mut RecursiveMutex) -> i32 {
	if recmutex.is_null() {
		return -EINVAL;
	}

	let mutex = unsafe {
							isolation_start!();
							let temp = &*recmutex;
							isolation_end!();
							temp
						};
	mutex.acquire();
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_lock(recmutex: *mut RecursiveMutex) -> i32 {
	let ret =  kernel_function!(__sys_recmutex_lock(recmutex));
	return ret;
}

#[no_mangle]
fn __sys_recmutex_unlock(recmutex: *mut RecursiveMutex) -> i32 {
	if recmutex.is_null() {
		return -EINVAL;
	}

	let mutex = unsafe {
							isolation_start!();
							let temp = &*recmutex;
							isolation_end!();
							temp
						};
	mutex.release();
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_unlock(recmutex: *mut RecursiveMutex) -> i32 {
	let ret = kernel_function!(__sys_recmutex_unlock(recmutex));
	return ret;
}
