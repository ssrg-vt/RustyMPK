// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//                    Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

macro_rules! align_down {
	($value:expr, $alignment:expr) => {
		$value & !($alignment - 1)
	};
}

macro_rules! align_up {
	($value:expr, $alignment:expr) => {
		align_down!($value + ($alignment - 1), $alignment)
	};
}

/// Print formatted text to our console.
///
/// From http://blog.phil-opp.com/rust-os/printing-to-screen.html, but tweaked
/// for HermitCore.
macro_rules! print {
	($($arg:tt)+) => ({
		use core::fmt::Write;
		$crate::console::CONSOLE.lock().write_fmt(format_args!($($arg)+)).unwrap();
	});
}

/// Print formatted text to our console, followed by a newline.
macro_rules! println {
	($($arg:tt)+) => (print!("{}\n", format_args!($($arg)+)));
}

macro_rules! isolate_var {
    /* .data */
    (static mut $name:ident: $var_type:ty = $val:expr) => {
        #[link_section = ".isolated_data"]
        static mut $name: $var_type = $val;
    };
    /* uninitialized */
    (static mut $name:ident: $var_type:ty) => {
        #[link_section = ".isolated_data"]
        static mut $name: $var_type = 0;
    };
    /* pub */
    (pub static mut $name:ident: $var_type:ty = $val:expr) => {
        #[link_section = ".isolated_data"]
        pub static mut $name: $var_type = $val;
    };
    /* pub uninitialized */
    (pub static mut $name:ident: $var_type:ty) => {
        #[link_section = ".isolated_data"]
        pub static mut $name: $var_type = 0;
    };
}

macro_rules! isolate_pointer {
    /* write on a raw pointer */
    (*$name:ident = $val:expr) => {{
        asm!("mov $$0xC, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			:
			:
			: "eax", "ecx", "edx"
			: "volatile");

        *$name = $val;

        asm!("xor %eax, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
              lfence"
			:
			:
			: "eax", "ecx", "edx"
			: "volatile");

    }};

    /* read on a raw pointer */ 
    (*$name:ident) => {{
        asm!("mov $$0xC, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			:
			:
			: "eax", "ecx", "edx"
			: "volatile");

        let temp_val = *$name;
        
        asm!("xor %eax, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
              lfence"
			:
			:
			: "eax", "ecx", "edx"
			: "volatile");
        temp_val
    }};
}

#[cfg(feature = "shm")]
macro_rules! isolate_function_weak {
	($f:ident($($x:tt)*)) => {{
		info!("shm enabled");
		use x86_64::kernel::percore::core_scheduler;
		use x86_64::mm::paging::{BasePageSize, set_pkey_on_page_table_entry};
		use mm::{SAFE_MEM_REGION, SHARED_MEM_REGION};

		let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;
		let mut __current_rbp: usize = 0;
		let mut __current_rsp: usize = 0;
		let mut __count:usize = 0;

		/* We get the address of current stack frame and calculate size of the stack frame. */
		asm!("mov %rbp, $0;
			  mov %rsp, $1"
			: "=r"(__current_rbp), "=r"(__current_rsp)
			:
			: "rbp", "rsp"
			: "volatile");

		/* Calculate the number of pages of the current stack frame */
		__count = (align_up!(__current_rbp, 4096) - align_down!(__current_rsp, 4096))/4096;
		/* Set the current stack frame as SHARED_MEM_REGION in order that the isolated unsafe function can access it. */
		set_pkey_on_page_table_entry::<BasePageSize>(align_down!(__current_rsp, 4096), __count, SHARED_MEM_REGION);

		/* "mov $$0xC, $eax" is to set SAFE_MEM_REGION (pkey of 1) permission to NONE */
		asm!("mov $0, %rsp;
			  mov $$0xC, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			: 
			: "r"(__isolated_stack)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $f($($x)*);

		asm!("xor %eax, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence;
			  mov $0, %rsp"
			:
			: "r"(__current_rsp)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");
		set_pkey_on_page_table_entry::<BasePageSize>(align_down!(__current_rsp, 4096), __count, SAFE_MEM_REGION);
		temp_ret
	}};
}

#[cfg(not(feature = "shm"))]
macro_rules! isolate_function_weak {
	($f:ident($($x:tt)*)) => {{
		info!("copying enabled");
		use x86_64::kernel::percore::core_scheduler;
		use x86_64::mm::paging::{BasePageSize, set_pkey_on_page_table_entry};
		use mm::{SAFE_MEM_REGION, SHARED_MEM_REGION};
		use core::intrinsics::copy_nonoverlapping;

		let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;
		let mut __current_rbp: usize = 0;
		let mut __current_rsp: usize = 0;
		let mut __size:usize = 0;

		/* We get the address of current stack frame and calculate size of the stack frame. */
		asm!("mov %rbp, $0;
			  mov %rsp, $1"
			: "=r"(__current_rbp), "=r"(__current_rsp)
			:
			: "rbp", "rsp"
			: "volatile");

		/* Calculate the number of pages of the current stack frame */
		__size = align_up!(__current_rbp, 4096) - align_down!(__current_rsp, 4096);
		/* Set pkey of SHARED_MEM_REGION on the current stack frame */
		set_pkey_on_page_table_entry::<BasePageSize>(align_down!(__current_rsp, 4096), __size/4096, SHARED_MEM_REGION);

		/* Permission to the safe region becomes NONE
		 * i.e., isolation starts
		 */
		asm!("mov $0, %rsp;
			  mov $$0xC, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			  :
			  : "r"(__isolated_stack - __size)
			  : "rsp", "eax", "ecx", "edx"
			  : "volatile");

		/* Copy the original stack frame to the isolated stack */
		copy_nonoverlapping(align_down!(__current_rsp, 4096) as *const u8, (__isolated_stack - __size) as *mut u8, __size);

		/* Change rsp and rbp to point to the isolated stack */
		asm!("mov $0, %rsp;
			  mov $1, %rbp"
			: 
			: "r"(__isolated_stack - (align_up!(__current_rbp, 4096) - __current_rsp)),
			  "r"(__isolated_stack - (align_up!(__current_rbp, 4096) - __current_rbp))
			: "rsp", "rbp"
			: "volatile");

		/* Set the original stack frame no access */
		asm!("mov $$0xCC, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			:
			:
			: "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $f($($x)*);

		/* Set the original stack frame writable */
		asm!("mov $$0xC, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			:
			:
			: "eax", "ecx", "edx"
			: "volatile");

		/* Copy the change back to the original stack frame */
		copy_nonoverlapping((__isolated_stack - __size) as *const u8, align_down!(__current_rsp, 4096) as *mut u8, __size);

		/* Permission to the safe region becomes RW 
		 * i.e., isolation ends
		 */
		asm!("xor %eax, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence;
			  mov $0, %rsp;
			  mov $1, %rbp"
			:
			: "r"(__current_rsp), "r"(__current_rbp)
			: "rsp", "rbp", "eax", "ecx", "edx"
			: "volatile");

		set_pkey_on_page_table_entry::<BasePageSize>(align_down!(__current_rsp, 4096), __size/4096, SAFE_MEM_REGION);
		temp_ret
	}};
}

macro_rules! isolate_function_strong {
	($f:ident($($x:tt)*)) => {{
		use x86_64::kernel::percore::core_scheduler;
		use scheduler::CURRENT_STACK_POINTER;
		let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;

		asm!("mov %rsp, $0;
			mov $1, %rsp;
			mov $$0xC, %eax;
			xor %ecx, %ecx;
			xor %edx, %edx;
			wrpkru;
			lfence"
			: "=r"(CURRENT_STACK_POINTER)
			: "r"(__isolated_stack)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $f($($x)*);

		asm!("xor %eax, %eax;
			xor %ecx, %ecx;
			xor %edx, %edx;
			wrpkru;
			lfence;
			mov $0, %rsp"
			:
			: "r"(CURRENT_STACK_POINTER)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");
		temp_ret
	}};
}

/*
macro_rules! gs_relative {
	(&$name:ident as *const $var_type:ty) => {{
		let offset: usize = &$name as *const $var_type as usize - processor::readgs();
		offset
	}};

	(&mut $name:ident as *mut $var_type:ty) => {{
		let offset: usize = &mut $name as *mut $var_type as usize - processor::readgs();
		offset
	}};
}

macro_rules! isolation_test {
	($f:ident($($x:tt)*)) => {
		parse_parameters!($($x)*);
		$f($($x)*);
		info!("test:{}", test1);
		//isolate_function_weak!($f($($x)*));
	};
}

macro_rules! parse_parameters {
	() => {};

	/* Find a pointer among the parameters */
	($x:tt: *const $var_type:ty, $($tail:tt)*) => {{
		*($x) += 1;
		parse_parameters!($($tail)*);
	}};

	($x:tt: *mut $var_type:ty, $($tail:tt)*) => {{
		*($x) += 1;
		parse_parameters!($($tail)*);
	}};

	/* Walk-through parameters */
	($x:tt, $($tail:tt)*) => {{
		parse_parameters!($($tail)*);
	}};

	/* Find that the last parameter is a pointer */
	($x:tt: *const $var_type:ty) => {{
		*($x) += 1;
	}};

	($x:tt: *mut $var_type:ty) => {{
		*($x) += 1;
		static mut test1:usize = 1;
	}};

	/* the last parameter */
	($x:tt) => {};
	($x:tt: $var_type:ty) => {};
}
*/