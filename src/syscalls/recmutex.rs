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
	unsafe {
		let temp = isolate_function_strong!(Box::into_raw(boxed_mutex));
		isolation_start!();
		*recmutex = temp;
		isolation_end!();
	}
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_init(recmutex: *mut *mut RecursiveMutex) -> i32 {
	//kernel_enter!("sys_recmutex_init");
	let ret = kernel_function!(__sys_recmutex_init(recmutex));
	//kernel_exit!("sys_recmutex_init");
	return ret;
}

#[no_mangle]
fn __sys_recmutex_destroy(recmutex: *mut RecursiveMutex) -> i32 {
	if recmutex.is_null() {
		return -EINVAL;
	}

	// Consume the pointer to the raw memory into a Box again
	// and drop the Box to free the associated memory.
	unsafe {
		isolate_function_strong!(Box::from_raw(recmutex));
	}
	0
}

#[no_mangle]
pub extern "C" fn sys_recmutex_destroy(recmutex: *mut RecursiveMutex) -> i32 {
	//kernel_enter!("sys_recmutex_destroy");
	let ret = kernel_function!(__sys_recmutex_destroy(recmutex));
	//kernel_exit!("sys_recmutex_destroy");
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
	//kernel_enter!("sys_recmutex_lock");
	let ret =  kernel_function!(__sys_recmutex_lock(recmutex));
	//kernel_exit!("sys_recmutex_lock");
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
	//kernel_enter!("sys_recmutex_unlock");
	let ret = kernel_function!(__sys_recmutex_unlock(recmutex));
	//kernel_exit!("sys_recmutex_unlock");
	return ret;
}