// Copyright (c) 2018 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Kernel Message Buffer for Multi-Kernel mode.
//! Can be read from the Linux side as no serial port is available.

use core::intrinsics::volatile_store;
use core::sync::atomic::{AtomicUsize, Ordering};
use mm;

const KMSG_SIZE: usize = 0x1000;

#[repr(align(64))]
#[repr(C)]
struct KmsgSection {
	buffer: [u8; KMSG_SIZE + 1],
}

unsafe_global_var!( static mut KMSG: KmsgSection = KmsgSection {
	buffer: [0; KMSG_SIZE + 1],
});

safe_global_var!(static BUFFER_INDEX: AtomicUsize = AtomicUsize::new(0));

unsafe fn write_byte<T>(buffer: *mut T, byte: T) {
	volatile_store(buffer, byte);
}

pub fn kmsg_write_byte(byte: u8) {
	let index = BUFFER_INDEX.fetch_add(1, Ordering::SeqCst);
	unsafe {
		isolate_function_strong!(write_byte(&mut KMSG.buffer[index % KMSG_SIZE], byte));
	}
}
