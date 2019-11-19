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
pub extern "C" fn sys_spinlock_init(lock: *mut *mut SpinlockContainer) -> i32 {
	kernel_enter!("sys_spinlock_init");
	if lock.is_null() {
		kernel_exit!("sys_spinlock_init");
		return -EINVAL;
	}

	let boxed_container = Box::new(SpinlockContainer {
		lock: Spinlock::new(()),
		guard: None,
	});
	unsafe {
		*lock = Box::into_raw(boxed_container);
	}
	kernel_exit!("sys_spinlock_init");
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_destroy(lock: *mut SpinlockContainer) -> i32 {
	kernel_enter!("sys_spinlock_destroy");
	if lock.is_null() {
		kernel_exit!("sys_spinlock_destroy");
		return -EINVAL;
	}

	// Consume the lock into a box, which is then dropped.
	unsafe {
		Box::from_raw(lock);
	}
	kernel_exit!("sys_spinlock_destroy");
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_lock(lock: *mut SpinlockContainer) -> i32 {
	kernel_enter!("sys_spinlock_lock");
	if lock.is_null() {
		kernel_exit!("sys_spinlock_lock");
		return -EINVAL;
	}

	let container = unsafe { &mut *lock };
	assert!(
		container.guard.is_none(),
		"Called sys_spinlock_lock when a lock is already held!"
	);
	container.guard = Some(container.lock.lock());
	kernel_exit!("sys_spinlock_lock");
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_unlock(lock: *mut SpinlockContainer) -> i32 {
	kernel_enter!("sys_spinlock_unlock");
	if lock.is_null() {
		kernel_exit!("sys_spinlock_unlock");
		return -EINVAL;
	}

	let container = unsafe { &mut *lock };
	assert!(
		container.guard.is_some(),
		"Called sys_spinlock_unlock when no lock is currently held!"
	);
	container.guard = None;
	kernel_exit!("sys_spinlock_unlock");
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_irqsave_init(lock: *mut *mut SpinlockIrqSaveContainer) -> i32 {
	kernel_enter!("sys_spinlock_irqsave_init");
	if lock.is_null() {
		kernel_exit!("sys_spinlock_irqsave_init");
		return -EINVAL;
	}

	let boxed_container = Box::new(SpinlockIrqSaveContainer {
		lock: SpinlockIrqSave::new(()),
		guard: None,
	});
	unsafe {
		*lock = Box::into_raw(boxed_container);
	}
	kernel_exit!("sys_spinlock_irqsave_init");
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_irqsave_destroy(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	kernel_enter!("sys_spinlock_irqsave_destroy");
	if lock.is_null() {
		kernel_exit!("sys_spinlock_irqsave_destroy");
		return -EINVAL;
	}

	// Consume the lock into a box, which is then dropped.
	unsafe {
		Box::from_raw(lock);
	}
	kernel_exit!("sys_spinlock_irqsave_destroy");
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_irqsave_lock(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	kernel_enter!("sys_spinlock_irqsave_lock");
	if lock.is_null() {
		kernel_exit!("sys_spinlock_irqsave_lock");
		return -EINVAL;
	}

	let container = unsafe { &mut *lock };
	assert!(
		container.guard.is_none(),
		"Called sys_spinlock_irqsave_lock when a lock is already held!"
	);
	container.guard = Some(container.lock.lock());
	kernel_exit!("sys_spinlock_irqsave_lock");
	0
}

#[no_mangle]
pub extern "C" fn sys_spinlock_irqsave_unlock(lock: *mut SpinlockIrqSaveContainer) -> i32 {
	kernel_enter!("sys_spinlock_irqsave_unlock");
	if lock.is_null() {
		kernel_exit!("sys_spinlock_irqsave_unlock");
		return -EINVAL;
	}

	let container = unsafe { &mut *lock };
	assert!(
		container.guard.is_some(),
		"Called sys_spinlock_irqsave_unlock when no lock is currently held!"
	);
	container.guard = None;
	kernel_exit!("sys_spinlock_irqsave_unlock");
	0
}
