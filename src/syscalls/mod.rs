// Copyright (c) 2018 Colin Finck, RWTH Aachen University
//                    Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

mod condvar;
mod interfaces;
#[cfg(feature = "newlib")]
mod lwip;
mod processor;
mod random;
mod recmutex;
mod semaphore;
mod spinlock;
mod system;
mod tasks;
mod timer;

pub use self::condvar::*;
pub use self::processor::*;
pub use self::random::*;
pub use self::recmutex::*;
pub use self::semaphore::*;
pub use self::spinlock::*;
pub use self::system::*;
pub use self::tasks::*;
pub use self::timer::*;
use environment;
#[cfg(feature = "newlib")]
use synch::spinlock::SpinlockIrqSave;
use syscalls::interfaces::SyscallInterface;

#[cfg(feature = "newlib")]
const LWIP_FD_BIT: i32 = (1 << 30);

#[cfg(feature = "newlib")]
safe_global_var!(pub static LWIP_LOCK: SpinlockIrqSave<()> = SpinlockIrqSave::new(()));

safe_global_var!(static mut SYS: &'static dyn SyscallInterface = &interfaces::Generic);

pub fn init() {
	
	// We know that HermitCore has successfully initialized a network interface.
	// Now check if we can load a more specific SyscallInterface to make use of networking.
	if environment::is_proxy() {
		panic!("Currently, we don't support the proxy mode!");
	} else if environment::is_uhyve() {
		unsafe {SYS = &interfaces::Uhyve};
	}

		// Perform interface-specific initialization steps.
	unsafe {SYS.init()};

	random_init();
	#[cfg(feature = "newlib")]
	sbrk_init();
}

pub fn get_application_parameters() -> (i32, *const *const u8, *const *const u8) {
	unsafe { SYS.get_application_parameters() }
}

#[no_mangle]
pub extern "C" fn sys_shutdown(arg: i32) -> ! {
	unsafe { kernel_function!(SYS.shutdown(arg)) }
}

#[no_mangle]
pub extern "C" fn sys_unlink(name: *const u8) -> i32 {
	unsafe { kernel_function!(SYS.unlink(name)) }
}

#[no_mangle]
pub extern "C" fn sys_open(name: *const u8, flags: i32, mode: i32) -> i32 {
	unsafe { kernel_function!(SYS.open(name, flags, mode)) }
}

#[no_mangle]
pub extern "C" fn sys_close(fd: i32) -> i32 {
	unsafe { kernel_function!(SYS.close(fd)) }
}

#[no_mangle]
pub extern "C" fn sys_read(fd: i32, buf: *mut u8, len: usize) -> isize {
	unsafe { kernel_function!(SYS.read(fd, buf, len)) }
}

#[no_mangle]
pub extern "C" fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize {
	unsafe { kernel_function!(SYS.write(fd, buf, len)) }
}

#[no_mangle]
pub extern "C" fn sys_lseek(fd: i32, offset: isize, whence: i32) -> isize {
	unsafe { kernel_function!(SYS.lseek(fd, offset, whence)) }
}

#[no_mangle]
pub extern "C" fn sys_stan(file: *const u8, st: usize) -> i32 {
	unsafe { kernel_function!(SYS.stat(file, st)) }
}
