// Copyright (c) 2018 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use alloc::boxed::Box;
use errno::*;
use synch::spinlock::*;
use mm;

pub struct SpinlockContainer<'a> {
	lock: Spinlock<()>,
	guard: Option<SpinlockGuard<'a, ()>>,
}

pub struct SpinlockIrqSaveContainer<'a> {
	lock: SpinlockIrqSave<()>,
	guard: Option<SpinlockIrqSaveGuard<'a, ()>>,
}

#[no_mangle]
fn __sys_spinlock_init(lock: *mut *mut SpinlockContainer) -> i32 {
	if lock.is_null() {
		return -EINVAL;
	}

	let boxed_container = Box::new(SpinlockContainer {
		lock: Spinlock::new(()),
		guard: None,
	});
	let ret = Box::into_raw(boxed_container);
	unsafe {
        isolation_start!();
		*lock = ret;
        isolation_end!();
	}
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_init(lock: *mut *mut SpinlockContainer) -> i32 {
	let ret = kernel_function!(__sys_spinlock_init(lock));
	return ret;
}

#[no_mangle]
fn __sys_spinlock_destroy(lock: *mut SpinlockContainer) -> i32 {
	if lock.is_null() {
		return -EINVAL;
	}

	// Consume the lock into a box, which is then dropped.
	unsafe {
		isolate_function_strong!(Box::from_raw(lock));
	}
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_destroy(lock: *mut SpinlockContainer) -> i32 {
	let ret = kernel_function!(__sys_spinlock_destroy(lock));
	return ret;
}

#[no_mangle]
fn __sys_spinlock_lock(lock: *mut SpinlockContainer) -> i32 {
	if lock.is_null() {
		return -EINVAL;
	}

	let container = unsafe {
		isolation_start!();
		let ret = &mut *lock;
		isolation_end!();
		ret
	};
	assert!(
		container.guard.is_none(),
		"Called sys_spinlock_lock when a lock is already held!"
	);
	container.guard = Some(container.lock.lock());
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_lock(lock: *mut SpinlockContainer) -> i32 {
	let ret = kernel_function!(__sys_spinlock_lock(lock));
	return ret;
}

#[no_mangle]
fn __sys_spinlock_unlock(lock: *mut SpinlockContainer) -> i32 {
	if lock.is_null() {
		return -EINVAL;
	}

	let container = unsafe {
		isolation_start!();
		let ret = &mut *lock;
		isolation_end!();
		ret
	};
	assert!(
		container.guard.is_some(),
		"Called sys_spinlock_unlock when no lock is currently held!"
	);
	container.guard = None;
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_unlock(lock: *mut SpinlockContainer) -> i32 {
	let ret = kernel_function!(__sys_spinlock_unlock(lock));
	return ret;
}

#[no_mangle]
fn __sys_spinlock_irqsave_init(lock: *mut *mut SpinlockIrqSaveContainer) -> i32 {
	if lock.is_null() {
		return -EINVAL;
	}

	let boxed_container = Box::new(SpinlockIrqSaveContainer {
		lock: SpinlockIrqSave::new(()),
		guard: None,
	});
	let ret = Box::into_raw(boxed_container);
	unsafe {
        isolation_start!();
		*lock = ret;
        isolation_end!();
	};
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_irqsave_init(lock: *mut *mut SpinlockIrqSaveContainer) -> i32 {
	let ret = kernel_function!(__sys_spinlock_irqsave_init(lock));
	return ret;
}

#[no_mangle]
fn __sys_spinlock_irqsave_destroy(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	if lock.is_null() {
		return -EINVAL;
	}

	// Consume the lock into a box, which is then dropped.
	unsafe {
		isolate_function_strong!(Box::from_raw(lock));
	}
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_irqsave_destroy(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	let ret = kernel_function!(__sys_spinlock_irqsave_destroy(lock));
	return ret;
}

#[no_mangle]
fn __sys_spinlock_irqsave_lock(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	if lock.is_null() {
		return -EINVAL;
	}

	let container = unsafe {
		isolation_start!();
		let ret = &mut *lock;
		isolation_end!();
		ret
	};
	assert!(
		container.guard.is_none(),
		"Called sys_spinlock_irqsave_lock when a lock is already held!"
	);
	container.guard = Some(container.lock.lock());
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_irqsave_lock(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	let ret = kernel_function!(__sys_spinlock_irqsave_lock(lock));
	return ret;
}

#[no_mangle]
fn __sys_spinlock_irqsave_unlock(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	if lock.is_null() {
		return -EINVAL;
	}

	let container = unsafe {
		isolation_start!();
		let ret = &mut *lock;
		isolation_end!();
		ret
	};
	assert!(
		container.guard.is_some(),
		"Called sys_spinlock_irqsave_unlock when no lock is currently held!"
	);
	container.guard = None;
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_irqsave_unlock(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	let ret = kernel_function!(__sys_spinlock_irqsave_unlock(lock));
	return ret;
}
