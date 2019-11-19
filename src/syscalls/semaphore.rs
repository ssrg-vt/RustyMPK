// Copyright (c) 2018 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use alloc::boxed::Box;
use arch;
use errno::*;
use synch::semaphore::Semaphore;
use mm;

#[no_mangle]
pub extern "C" fn sys_sem_init(sem: *mut *mut Semaphore, value: u32) -> i32 {
	kernel_enter!("sys_sem_init");
	//println!("sys_sem_init, sem: {:#X}", sem as usize);
	if sem.is_null() {
		kernel_exit!("sys_sem_init");
		return -EINVAL;
	}

	// Create a new boxed semaphore and return a pointer to the raw memory.
	let boxed_semaphore = Box::new(Semaphore::new(value as isize));
	unsafe {
		*sem = Box::into_raw(boxed_semaphore);
	}
	kernel_exit!("sys_sem_init");
	0
}

#[no_mangle]
pub extern "C" fn sys_sem_destroy(sem: *mut Semaphore) -> i32 {
	kernel_enter!("sys_sem_destroy");
	if sem.is_null() {
		kernel_exit!("sys_sem_destroy");
		return -EINVAL;
	}

	// Consume the pointer to the raw memory into a Box again
	// and drop the Box to free the associated memory.
	unsafe {
		Box::from_raw(sem);
	}
	kernel_exit!("sys_sem_destroy");
	0
}

#[no_mangle]
pub extern "C" fn sys_sem_post(sem: *const Semaphore) -> i32 {
	kernel_enter!("sys_sem_post");
	if sem.is_null() {
		kernel_exit!("sys_sem_post");
		return -EINVAL;
	}

	// Get a reference to the given semaphore and release it.
	let semaphore = unsafe { &*sem };
	semaphore.release();
	kernel_exit!("sys_sem_post");
	0
}

#[no_mangle]
pub extern "C" fn sys_sem_trywait(sem: *const Semaphore) -> i32 {
	kernel_enter!("sys_sem_trywait");
	if sem.is_null() {
		kernel_exit!("sys_sem_trywait");
		return -EINVAL;
	}

	// Get a reference to the given semaphore and acquire it in a non-blocking fashion.
	let semaphore = unsafe { &*sem };
	if semaphore.try_acquire() {
		kernel_exit!("sys_sem_trywait");
		0
	} else {
		kernel_exit!("sys_sem_trywait");
		-ECANCELED
	}
}

#[no_mangle]
pub extern "C" fn sys_sem_timedwait(sem: *const Semaphore, ms: u32) -> i32 {
	kernel_enter!("sys_sem_timedwait");
	//println!("sys_sem_timedwait, sem: {:#X}", sem as usize);
	if sem.is_null() {
		kernel_exit!("sys_sem_timedwait1");
		return -EINVAL;
	}

	// Calculate the absolute wakeup time in processor timer ticks out of the relative timeout in milliseconds.
	let wakeup_time = if ms > 0 {
		Some(arch::processor::get_timer_ticks() + u64::from(ms) * 1000)
	} else {
		None
	};

	// Get a reference to the given semaphore and wait until we have acquired it or the wakeup time has elapsed.
	let semaphore = unsafe { &*sem };
	if semaphore.acquire(wakeup_time) {
		kernel_exit!("sys_sem_timedwait2");
		0
	} else {
		kernel_exit!("sys_sem_timedwait3");
		-ETIME
	}
}

#[no_mangle]
pub extern "C" fn sys_sem_cancelablewait(sem: *const Semaphore, ms: u32) -> i32 {
	sys_sem_timedwait(sem, ms)
}
