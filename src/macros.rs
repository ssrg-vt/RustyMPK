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
    (static mut $name:ident: $var_type:ty, $val:expr) => {
        #[link_section = ".isolated_data"]
        static mut $name: $var_type = $val;
    };
    /* uninitialized */
    (static mut $name:ident: $var_type:ty) => {
        #[link_section = ".isolated_data"]
        static mut $name: $var_type = 0;
    };
    /* pub */
    (pub static mut $name:ident: $var_type:ty, $val:expr) => {
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

macro_rules! isolate_function_no_ret {
    ($f:ident($($x:tt)*)) => {{
	    use x86_64::kernel::percore::core_scheduler;
        use scheduler;
        let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;

		asm!("mov %rsp, $0;
              mov $1, %rsp;
			  mov $$0xC, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			: "=r"(scheduler::SAFE_STACK_POINTER)
			: "r"(__isolated_stack)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

        $f($($x)*);

        asm!("xor %eax, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
              lfence;
			  mov $0, %rsp"
			:
			: "r"(scheduler::SAFE_STACK_POINTER)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");
    }};
}

macro_rules! isolate_function {
    ($f:ident($($x:tt)*)) => {{
	    use x86_64::kernel::percore::core_scheduler;
        use scheduler;
        let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;

		asm!("mov %rsp, $0;
              mov $1, %rsp;
			  mov $$0xC, %eax;
			  xor %ecx, %ecx;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			: "=r"(scheduler::SAFE_STACK_POINTER)
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
			: "r"(scheduler::SAFE_STACK_POINTER)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");
        temp_ret
    }};
}