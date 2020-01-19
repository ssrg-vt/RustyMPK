// Copyright (c) 2019 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use alloc::boxed::Box;
use arch::percore::*;
use core::mem;
use scheduler;
use scheduler::task::PriorityTaskQueue;
use mm;

struct CondQueue {
	queue: PriorityTaskQueue,
	id: usize,
}

impl CondQueue {
	pub fn new(id: usize) -> Self {
		CondQueue {
			queue: PriorityTaskQueue::new(),
			id: id,
		}
	}
}

impl Drop for CondQueue {
	fn drop(&mut self) {
		debug!("Drop queue for condition variable with id 0x{:x}", self.id);
	}
}

#[no_mangle]
fn __sys_destroy_queue(ptr: usize) -> i32 {
	let id = ptr as *mut usize;
	if id.is_null() {
		debug!("sys_wait: ivalid address to condition variable");
		return -1;
	}

	let temp_id;
	unsafe {
		isolation_start!();
		temp_id = *id;
		isolation_end!();
	}
	if temp_id != 0 {
		let cond = unsafe { isolate_function_strong!(Box::from_raw(temp_id as *mut CondQueue)) };
		mem::drop(cond);

		// reset id
		unsafe {
			isolation_start!();
			*id = 0;
			isolation_end!();
		}
	}
	0
}

#[no_mangle]
pub unsafe fn sys_destroy_queue(ptr: usize) -> i32 {
	let ret = kernel_function!(__sys_destroy_queue(ptr));
	return ret;
}

#[no_mangle]
fn __sys_notify(ptr: usize, count: i32) -> i32 {
	let id = ptr as *const usize;
	let temp_id;
	unsafe {
		isolation_start!();
		temp_id = *id;
		isolation_end!();
	}
	if id.is_null() || temp_id == 0 {
		// invalid argument
		debug!("sys_notify: invalid address to condition variable");
		return -1;
	}

	let cond;
	unsafe {
		isolation_start!();
		cond = &mut *((*id) as *mut CondQueue);
		isolation_end!();
	}

	if count < 0 {
		// Wake up all task that has been waiting for this condition variable
		while let Some(task) = cond.queue.pop() {
			let core_scheduler = scheduler::get_scheduler(task.borrow().core_id);
			core_scheduler.blocked_tasks.lock().custom_wakeup(task);
		}
	} else {
		for _ in 0..count {
			// Wake up any task that has been waiting for this condition variable
			if let Some(task) = cond.queue.pop() {
				let core_scheduler = scheduler::get_scheduler(task.borrow().core_id);
				core_scheduler.blocked_tasks.lock().custom_wakeup(task);
			} else {
				debug!("Unable to wakeup task");
			}
		}
	}
	0
}

#[no_mangle]
pub unsafe fn sys_notify(ptr: usize, count: i32) -> i32 {
	let ret = kernel_function!(__sys_notify(ptr, count));
	return ret;
}

#[no_mangle]
fn __sys_add_queue(ptr: usize, timeout_ns: i64) -> i32 {
	let id = ptr as *mut usize;
	if id.is_null() {
		debug!("sys_wait: ivalid address to condition variable");
		kernel_exit!("sys_add_queue");
		return -1;
	}

	let temp_id;
	unsafe {
		isolation_start!();
		temp_id = *id;
		isolation_end!();
	}
	if temp_id == 0 {
		debug!("Create condition variable queue");
		let queue = Box::new(CondQueue::new(ptr));
		let temp = Box::into_raw(queue) as usize;
		unsafe {
			isolation_start!();
			*id = temp;
			isolation_end!();
		}
	}

	let wakeup_time = if timeout_ns <= 0 {
		None
	} else {
		Some(timeout_ns as u64 / 1000)
	};

	// Block the current task and add it to the wakeup queue.
	let core_scheduler = core_scheduler();
	core_scheduler
		.blocked_tasks
		.lock()
		.add(core_scheduler.current_task.clone(), wakeup_time);

	unsafe {
		isolation_start!();
		let cond = &mut *((*id) as *mut CondQueue);
		isolation_end!();
		cond.queue.push(core_scheduler.current_task.clone());
	}
	0
}

#[no_mangle]
pub unsafe fn sys_add_queue(ptr: usize, timeout_ns: i64) -> i32 {
	let ret = kernel_function!(__sys_add_queue(ptr, timeout_ns));
	return ret;
}

#[no_mangle]
fn __sys_wait(_ptr: usize) -> i32 {
	// Switch to the next task.
	core_scheduler().scheduler();
	0
}

#[no_mangle]
pub fn sys_wait(_ptr: usize) -> i32 {
	let ret = kernel_function!(__sys_wait(_ptr));
	return ret;
}
