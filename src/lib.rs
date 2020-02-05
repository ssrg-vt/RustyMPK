// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//                    Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/*
 * First version is derived and adapted for HermitCore from
 * Philipp Oppermann's excellent series of blog posts (http://blog.phil-opp.com/)
 * and Eric Kidd's toy OS (https://github.com/emk/toyos-rs).
 */

#![warn(clippy::all)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::identity_op)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::toplevel_ref_arg)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::println_empty_string)]
#![allow(clippy::single_match)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::forget_copy)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(specialization)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(type_ascription)]
#![feature(alloc_error_handler)]
#![allow(unused_macros)]
#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

// EXTERNAL CRATES
#[cfg_attr(test, macro_use)] 
extern crate alloc;
#[macro_use]
extern crate bitflags;
#[cfg(target_arch = "x86_64")]
extern crate multiboot;
#[cfg(target_arch = "x86_64")]
extern crate x86;
#[macro_use]
extern crate log;

#[cfg(target_arch = "x86_64")]
extern crate raw_cpuid;

//#[macro_use]
//extern crate lazy_static;

#[macro_use]
mod macros;

#[macro_use]
mod logging;

mod arch;
mod collections;
mod config;
mod console;
mod drivers;
mod environment;
mod errno;
mod kernel_message_buffer;
mod mm;
#[cfg(not(test))]
mod runtime_glue;
mod scheduler;
mod synch;
mod syscalls;

pub use arch::*;
pub use config::*;
pub use syscalls::*;

use alloc::alloc::Layout;
use arch::percore::*;
use core::alloc::GlobalAlloc;
use mm::allocator::LockedHeap;

#[cfg(not(test))]
#[global_allocator]
static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Interface to allocate memory from system heap
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn sys_malloc(size: usize, align: usize) -> *mut u8 {
	let layout: Layout = Layout::from_size_align(size, align).unwrap();
	let ptr;

	unsafe {
		//isolation_start!();
		//ptr = ALLOCATOR.alloc(layout);
		//isolation_end!();
		ptr = ALLOCATOR.alloc(layout);
	}

	trace!(
		"sys_malloc: allocate memory at 0x{:x} (size 0x{:x}, align 0x{:x})",
		ptr as usize,
		size,
		align
	);
	ptr
}

/// Interface to increase the size of a memory region
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn sys_realloc(ptr: *mut u8, size: usize, align: usize, new_size: usize) -> *mut u8 {
	let layout: Layout = Layout::from_size_align(size, align).unwrap();
	let new_ptr;

	unsafe {
		new_ptr = ALLOCATOR.realloc(ptr, layout, new_size);
	}

	trace!(
		"sys_realloc: resize memory at 0x{:x}, new address 0x{:x}",
		ptr as usize,
		new_ptr as usize
	);
	new_ptr
}

/// Interface to deallocate a memory region from the system heap
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn sys_free(ptr: *mut u8, size: usize, align: usize) {
	let layout: Layout = Layout::from_size_align(size, align).unwrap();

	trace!(
		"sys_free: deallocate memory at 0x{:x} (size 0x{:x})",
		ptr as usize,
		size
	);

	unsafe {
		ALLOCATOR.dealloc(ptr, layout);
	}
}

/// Helper function to check if uhyve provide an IP device
fn has_ipdevice() -> bool {
	let ip = arch::x86_64::kernel::get_ip();

	if ip[0] == 255 && ip[1] == 255 && ip[2] == 255 && ip[3] == 255 {
		false
	} else {
		true
	}
}

/// Entry point of a kernel thread, which initialize the libos
#[cfg(not(test))]
extern "C" fn initd(_arg: usize) {
	extern "C" {
		fn runtime_entry(argc: i32, argv: *const *const u8, env: *const *const u8) -> !;
		//#[cfg(feature = "newlib")]
		//fn init_lwip();
		#[cfg(feature = "newlib")]
		fn init_uhyve_netif() -> i32;
	}
	info!("initd starts");
	// initialize LwIP library for newlib-based applications
	/*#[cfg(feature = "newlib")]
	unsafe {
		if has_ipdevice() {
			isolate_function_strong!(init_lwip());
		}
	}*/

	if environment::is_uhyve() {
		// Initialize the uhyve-net interface using the IP and gateway addresses specified in hcip, hcmask, hcgateway.
		info!("HermitCore is running on uhyve!");
		if has_ipdevice() {
			/*#[cfg(feature = "newlib")]
			unsafe {
				isolate_function_strong!(init_uhyve_netif());
			}*/

			#[cfg(not(feature = "newlib"))]
			let _ = drivers::net::init();
		}
	} else if !environment::is_single_kernel() {
		// Initialize the mmnif interface using static IPs in the range 192.168.28.x.
		info!("HermitCore is running side-by-side to Linux!");
	}
	syscalls::init();

	// give the IP thread time to initialize the network interface
	core_scheduler().reschedule();

	#[cfg(not(feature = "newlib"))]
	{
		// Switch global allocator to user heap
		mm::init_user_allocator();
	}
	// Get the application arguments and environment variables.
	let (argc, argv, environ) = syscalls::get_application_parameters();
	user_start!(false);

        arch::processor::fpu_init();
        info!("Call runtime_entry");
	unsafe {
		runtime_entry(argc, argv, environ);
	}
}

fn performance_evaluation() {
	use core::ptr::write_bytes;
	let buffer = mm::unsafe_allocate(4096, true);
	for size in [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096].iter() {
		let scheduler = core_scheduler();
		let mut start = arch::processor::get_timer_ticks();
		for _ in 0..1000000 {
			unsafe {isolate_function_strong!(write_bytes(buffer as *mut u8, 0xCDu8, *size))};
			//safe_set_core_scheduler(scheduler);
		}
		start = arch::processor::get_timer_ticks() - start;

		info!("{}", start);
	}
}

fn security_evaluation_unsafe_isolation() {
	let scheduler = core_scheduler();
	info!("before set scheduler");
	bad_set_core_scheduler(scheduler);
	info!("after set scheduler");
}

/// Entry Point of HermitCore for the Boot Processor
#[cfg(not(test))]
fn boot_processor_main() -> ! {
	// Initialize the kernel and hardware.
	arch::message_output_init();
	logging::init();

	info!("Welcome to HermitCore-rs {} with intra-unikernel isolation", env!("CARGO_PKG_VERSION"));
	info!("Kernel starts at 0x{:x}", environment::get_base_address());
        info!(
		"TLS starts at 0x{:x} (size {} Bytes)",
		environment::get_tls_start(),
		environment::get_tls_memsz()
	);

	arch::boot_processor_init();
	scheduler::init();
	scheduler::add_current_core();

	if environment::is_single_kernel() && !environment::is_uhyve() {
		arch::boot_application_processors();
	}

        // Start the initd task.
	let core_scheduler = core_scheduler();
	core_scheduler.spawn(initd, 0, scheduler::task::NORMAL_PRIO);

	// Run the scheduler loop.
	loop {
		core_scheduler.reschedule_and_wait();
	}
}

/// Entry Point of HermitCore for an Application Processor
#[cfg(not(test))]
fn application_processor_main() -> ! {
	arch::application_processor_init();
	scheduler::add_current_core();
	let core_scheduler = core_scheduler();

	info!("Entering idle loop for application processor");

	// Run the scheduler loop.
	loop {
		core_scheduler.reschedule_and_wait();
	}
}
