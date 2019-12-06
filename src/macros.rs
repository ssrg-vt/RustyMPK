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

macro_rules! safe_global_var {
	/* read only */
	(static $name:ident: $var_type:ty = $val:expr) => {
        #[link_section = ".safe_data"]
        static $name: $var_type = $val;
    };
    /* uninitialized */
    (static $name:ident: $var_type:ty) => {
        #[link_section = ".safe_data"]
        static $name: $var_type = 0;
    };
    /* pub */
    (pub static $name:ident: $var_type:ty = $val:expr) => {
        #[link_section = ".safe_data"]
        pub static $name: $var_type = $val;
    };
    /* pub uninitialized */
    (pub static $name:ident: $var_type:ty) => {
        #[link_section = ".safe_data"]
        pub static $name: $var_type = 0;
	};

	/* writable */
    (static mut $name:ident: $var_type:ty = $val:expr) => {
        #[link_section = ".safe_data"]
        static mut $name: $var_type = $val;
    };
    /* uninitialized */
    (static mut $name:ident: $var_type:ty) => {
        #[link_section = ".safe_data"]
        static mut $name: $var_type = 0;
    };
    /* pub */
    (pub static mut $name:ident: $var_type:ty = $val:expr) => {
        #[link_section = ".safe_data"]
        pub static mut $name: $var_type = $val;
    };
    /* pub uninitialized */
    (pub static mut $name:ident: $var_type:ty) => {
        #[link_section = ".safe_data"]
        pub static mut $name: $var_type = 0;
    };
}

macro_rules! unsafe_global_var {
	/* read only */
	(static $name:ident: $var_type:ty = $val:expr) => {
		#[link_section = ".unsafe_data"]
		static $name: $var_type = $val;
	};
	/* uninitialized */
	(static $name:ident: $var_type:ty) => {
		#[link_section = ".unsafe_data"]
		static $name: $var_type = 0;
	};
	/* pub */
	(pub static $name:ident: $var_type:ty = $val:expr) => {
		#[link_section = ".unsafe_data"]
		pub static $name: $var_type = $val;
	};
	/* pub uninitialized */
	(pub static $name:ident: $var_type:ty) => {
		#[link_section = ".unsafe_data"]
		pub static $name: $var_type = 0;
	};

	/* writable */
    (static mut $name:ident: $var_type:ty = $val:expr) => {
        #[link_section = ".unsafe_data"]
        static mut $name: $var_type = $val;
    };
    /* uninitialized */
    (static mut $name:ident: $var_type:ty) => {
        #[link_section = ".unsafe_data"]
        static mut $name: $var_type = 0;
    };
    /* pub */
    (pub static mut $name:ident: $var_type:ty = $val:expr) => {
        #[link_section = ".unsafe_data"]
        pub static mut $name: $var_type = $val;
    };
    /* pub uninitialized */
    (pub static mut $name:ident: $var_type:ty) => {
        #[link_section = ".unsafe_data"]
        pub static mut $name: $var_type = 0;
    };
}

macro_rules! user_start {
	($e:expr) => {
		let user_stack_pointer = core_scheduler().current_task.borrow().user_stack_pointer;
		let kernel_stack_pointer;
		#[allow(unused)]
		unsafe {
			// Store the kernel stack pointer and switch to the user stack

			asm!("mov %rsp, $0"
				: "=r"(kernel_stack_pointer)
				:
				: "rsp"
				: "volatile");
			core_scheduler().current_task.borrow_mut().kernel_stack_pointer = kernel_stack_pointer;

			asm!("mov $0, %rsp"
				: 
				: "r"(user_stack_pointer)
				: "rsp"
				: "volatile");

			if $e {
				asm!("mov $$0xfc, %eax;
					  xor %ecx, %ecx;
					  xor %edx, %edx;
					  wrpkru;
					  lfence"
					:
					:
					: "eax", "ecx", "edx"
					: "volatile");
			}
		}
	};
}

macro_rules! user_end {
	() => {
		// And finally start the application.
		#[allow(unused)]
		unsafe {
			asm!("xor %eax, %eax;
				  xor %ecx, %ecx;
				  xor %edx, %edx;
				  wrpkru;
				  lfence"
				:
				:
				: "eax", "ecx", "edx"
				: "volatile");

			let kernel_stack_pointer = core_scheduler().current_task.borrow().kernel_stack_pointer;

			//let user_stack_pointer;
			// Store the kernel stack pointer and switch to the user stack
			asm!("mov $0, %rsp"
				: 
				: "r"(kernel_stack_pointer)
				: "rsp"
				: "volatile");
		}
	}
}

macro_rules! print_kernel_stack_pointer {
	() => {
		#[allow(unused)]
		unsafe {
			let mut kernel_stack_pointer: usize = 0;
			asm!("mov %rsp, $0"
				: "=r"(kernel_stack_pointer)
				:
				: "rsp"
				: "volatile");
			info!("print kernel stack pointer: {:#X}", kernel_stack_pointer);
		}
	}
}

macro_rules! kernel_enter {
	($e:expr) => {
		//unsafe{::SYSCALL_COUNTER += 1; }
		use x86_64::kernel::percore::core_scheduler;
		let kernel_stack_pointer: usize; 
		let user_stack_pointer: usize;

		#[allow(unused)]
		unsafe {
			asm!("xor %eax, %eax;
				  xor %ecx, %ecx;
				  xor %edx, %edx;
				  wrpkru;
				  lfence"
				:
				:
				: "eax", "ecx", "edx"
				: "volatile");

			asm!("mov %rsp, $0"
				: "=r"(user_stack_pointer)
				: 
				: "rsp"
				: "volatile");

			kernel_stack_pointer = core_scheduler().current_task.borrow().kernel_stack_pointer;
			// Switch to kernel stack
			asm!("mov $0, %rsp"
				: 
				: "r"(kernel_stack_pointer)
				: "rsp"
				: "volatile");
			
			core_scheduler().current_task.borrow_mut().user_stack_pointer = user_stack_pointer;
			//println!("=========enter : {}\\", $e);
		}
	};
}

macro_rules! kernel_exit {
	($e:expr) => {
		let user_stack_pointer = core_scheduler().current_task.borrow().user_stack_pointer;
		let kernel_stack_pointer: usize;

		#[allow(unused)]
		unsafe {
			asm!("mov %rsp, $0"
				: "=r"(kernel_stack_pointer)
				:
				: "rsp"
				: "volatile");
			core_scheduler().current_task.borrow_mut().kernel_stack_pointer = kernel_stack_pointer;

			// Switch to user stack
			asm!("mov $0, %rsp"
			  : 
			  : "r"(user_stack_pointer)
			  : "rsp"
			  : "volatile");

			//println!("=========exit : {}/", $e);

			asm!("mov $$0xfc, %eax;
				  xor %ecx, %ecx;
				  xor %edx, %edx;
				  wrpkru;
				  lfence"
				:
				:
				: "eax", "ecx", "edx"
				: "volatile");
		}
	};
}

macro_rules! kernel_function {
	($f:ident($($x:tt)*)) => {{
		//unsafe{::SYSCALL_COUNTER += 1; }
		use x86_64::kernel::percore::core_scheduler;
		let mut kernel_stack_pointer: usize;
		let mut user_stack_pointer: usize;
		#[allow(unused)]
		unsafe {
			// switch permission
			asm!("xor %eax, %eax;
				  xor %ecx, %ecx;
				  xor %edx, %edx;
				  wrpkru;
				  lfence"
				: 
				: 
				: "eax", "ecx", "edx"
				: "volatile");
			
			//let tid = core_scheduler().current_task.borrow().id.into();
	
			// Save user stack pointer and 
			// switch stack to the kernel stack
			asm!("mov %rsp, $0"
				: "=r"(user_stack_pointer)
				:
				: "rsp"
				: "volatile");

			kernel_stack_pointer = core_scheduler().current_task.borrow().kernel_stack_pointer;
			asm!("mov $0, %rsp"
				: 
				: "r"(kernel_stack_pointer)
				: "rsp"
				: "volatile");

			//println!("[{}]enter: {}\\", tid, stringify!($f));
			let temp_ret = $f($($x)*);
			//println!("[{}]exit : {}/", tid, stringify!($f));

			// Save kernel stack pinter and
			// swiatch back to the user stack
			/*
			asm!("mov %rsp, $0"
				: "=r"(kernel_stack_pointer)
				:
				: "rsp"
				: "volatile");
			core_scheduler().current_task.borrow_mut().kernel_stack_pointer = kernel_stack_pointer;
			*/
			asm!("mov $0, %rsp"
				: 
				: "r"(user_stack_pointer)
				: "rsp"
				: "volatile");

			asm!("mov $$0xfc, %eax;
				  xor %ecx, %ecx;
				  xor %edx, %edx;
				  wrpkru;
				  lfence"
				: 
				:
				: "eax", "ecx", "edx"
				: "volatile");

			temp_ret
		}
	}};

	($p:tt.$f:ident($($x:tt)*)) => {{
		//unsafe{::SYSCALL_COUNTER += 1; }
		use x86_64::kernel::percore::core_scheduler;
		#[allow(unused)]
		let mut kernel_stack_pointer: usize;
		#[allow(unused)]
		let mut user_stack_pointer: usize;
		#[allow(unused)]
		unsafe {
			// switch permission
			asm!("xor %eax, %eax;
				  xor %ecx, %ecx;
				  xor %edx, %edx;
				  wrpkru;
				  lfence"
				: 
				: 
				: "eax", "ecx", "edx"
				: "volatile");
			
			let tid = core_scheduler().current_task.borrow().id.into();
	
			// Save user stack pointer and 
			// switch stack to the kernel stack
			asm!("mov %rsp, $0"
				: "=r"(user_stack_pointer)
				:
				: "rsp"
				: "volatile");

			kernel_stack_pointer = core_scheduler().current_task.borrow().kernel_stack_pointer;
			asm!("mov $0, %rsp"
				: 
				: "r"(kernel_stack_pointer)
				: "rsp"
				: "volatile");

			//println!("[{}]enter: {}\\", tid, stringify!($f));
			let temp_ret = $p.$f($($x)*);
			//println!("[{}]exit : {}/", tid, stringify!($f));

			// Save kernel stack pinter and
			// swiatch back to the user stack
			asm!("mov %rsp, $0"
				: "=r"(kernel_stack_pointer)
				:
				: "rsp"
				: "volatile");
			core_scheduler().current_task.borrow_mut().kernel_stack_pointer = kernel_stack_pointer;

			asm!("mov $0, %rsp"
				: 
				: "r"(user_stack_pointer)
				: "rsp"
				: "volatile");

			asm!("mov $$0xfc, %eax;
				  xor %ecx, %ecx;
				  xor %edx, %edx;
				  wrpkru;
				  lfence"
				: 
				:
				: "eax", "ecx", "edx"
				: "volatile");

			temp_ret
		}
	}};
}

macro_rules! isolation_start {
	() => {
		//unsafe{ ::UNSAFE_COUNTER += 1; }
		asm!("xor %ecx, %ecx;
		    rdpkru;
			or $0, %eax;
			xor %edx, %edx;
			wrpkru;
			lfence"
			:
			: "r"(mm::UNSAFE_PERMISSION_IN)
			: "eax", "ecx", "edx"
			: "volatile");
	};
}

macro_rules! isolation_end {
	() => {
		asm!("xor %ecx, %ecx;
			rdpkru;
			and $0, %eax;
			xor %edx, %edx;
			wrpkru;
			lfence"
			:
			: "r"(mm::UNSAFE_PERMISSION_OUT)
			: "eax", "ecx", "edx"
			: "volatile"); 
	};
}

macro_rules! isolation_wrapper {
	($f:ident($($x:tt)*)) => {{
		//unsafe{ ::UNSAFE_COUNTER += 1; }
		asm!("xor %ecx, %ecx;
			rdpkru;
			or $0, %eax;
			xor %edx, %edx;
			wrpkru;
			lfence"
			:
			: "r"(mm::UNSAFE_PERMISSION_IN)
			: "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $f($($x)*);

		asm!("xor %ecx, %ecx;
			rdpkru;
			and $0, %eax;
			xor %edx, %edx;
			wrpkru;
			lfence"
			:
			: "r"(mm::UNSAFE_PERMISSION_OUT)
			: "eax", "ecx", "edx"
			: "volatile"); 

		temp_ret
	}};
}

macro_rules! print_this_page {
    ($addr: expr) => {
		use x86_64::mm::paging::{BasePageSize, LargePageSize, print_page_table_entry};
		if ($addr as usize) <= mm::kernel_end_address() {
			print_page_table_entry::<LargePageSize>($addr as usize);
		}
		else {
			print_page_table_entry::<BasePageSize>($addr as usize);
		}
	};
}

macro_rules! share {
    ($addr: expr) => {
		use x86_64::mm::paging::{BasePageSize, LargePageSize, set_pkey_on_page_table_entry};
		if ($addr as usize) <= mm::kernel_end_address() {
			set_pkey_on_page_table_entry::<LargePageSize>($addr as usize, 1, mm::SHARED_MEM_REGION);
		}
		else {
			set_pkey_on_page_table_entry::<BasePageSize>($addr as usize, 1, mm::SHARED_MEM_REGION);
		}
	};
}

macro_rules! unshare {
    ($addr: expr) => {
		if ($addr as usize) <= mm::kernel_end_address() {
			set_pkey_on_page_table_entry::<LargePageSize>($addr as usize, 1, mm::SAFE_MEM_REGION);
		}
		else {
			set_pkey_on_page_table_entry::<BasePageSize>($addr as usize, 1, mm::SAFE_MEM_REGION);
		}
	};
}

macro_rules! share_local_var {
	($name:ident: $var_type:ty) => {
		use x86_64::mm::paging::{BasePageSize, LargePageSize, set_pkey_on_page_table_entry};
		if (&$name as *const $var_type as usize) <= mm::kernel_end_address() {
			set_pkey_on_page_table_entry::<LargePageSize>(&$name as *const $var_type as usize, 1, mm::SHARED_MEM_REGION);
		}
		else {
			set_pkey_on_page_table_entry::<BasePageSize>(&$name as *const $var_type as usize, 1, mm::SHARED_MEM_REGION);
		}
	};

	($p:ident.$name:ident: $var_type:ty) => {
		use x86_64::mm::paging::{BasePageSize, LargePageSize, set_pkey_on_page_table_entry};
		if (&$p.$name as *const $var_type as usize) <= mm::kernel_end_address() {
			set_pkey_on_page_table_entry::<LargePageSize>(&$p.$name as *const $var_type as usize, 1, mm::SHARED_MEM_REGION);
		}
		else {
			set_pkey_on_page_table_entry::<BasePageSize>(&$p.$name as *const $var_type as usize, 1, mm::SHARED_MEM_REGION);
		}
	};

	(let $name:ident: $var_type:ty = $expr:expr) => {
		use x86_64::mm::paging::{BasePageSize, set_pkey_on_page_table_entry};
		let $name: $var_type = $expr;
		set_pkey_on_page_table_entry::<BasePageSize>(&$name as *const $var_type as usize, 1, mm::SHARED_MEM_REGION);
	};

	(let mut $name:ident: $var_type:ty = $expr:expr) => {
		use x86_64::mm::paging::{BasePageSize, set_pkey_on_page_table_entry};
		let mut $name: $var_type = $expr;
		set_pkey_on_page_table_entry::<BasePageSize>(&$name as *const $var_type as usize, 1, mm::SHARED_MEM_REGION);
	};
}

macro_rules! unshare_local_var {
	(let $name:ident: $var_type:ty = $expr:expr) => {
		use x86_64::mm::paging::{BasePageSize, set_pkey_on_page_table_entry};
		let $name: $var_type = $expr;
		set_pkey_on_page_table_entry::<BasePageSize>(&$name as *const $var_type as usize, 1, mm::SAFE_MEM_REGION);
	};

	(let mut $name:ident: $var_type:ty = $expr:expr) => {
		use x86_64::mm::paging::{BasePageSize, set_pkey_on_page_table_entry};
		let mut $name: $var_type = $expr;
		set_pkey_on_page_table_entry::<BasePageSize>(&$name as *const $var_type as usize, 1, mm::SAFE_MEM_REGION);
	};
}

macro_rules! isolate_function_weak {
	($f:ident($($x:tt)*)) => {{
		//unsafe{ ::UNSAFE_COUNTER += 1; }
		use x86_64::kernel::percore::core_scheduler;
		use x86_64::mm::paging::{BasePageSize, set_pkey_on_page_table_entry};
		use mm::{SAFE_MEM_REGION, SHARED_MEM_REGION};
                use config::DEFAULT_STACK_SIZE;

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

		/* or $1, %eax -> Add mm::UNSAFE_PERMISSION to current value of PKRU */
		asm!("mov $0, %rsp;
			  xor %ecx, %ecx;
			  rdpkru;
			  or $1, %eax;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			: 
			: "r"(__isolated_stack),"r"(mm::UNSAFE_PERMISSION_IN)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $f($($x)*);

		asm!("xor %ecx, %ecx;
			  rdpkru;
			  and $0, %eax;		
			  xor %edx, %edx;
			  wrpkru;
			  lfence;
			  mov $1, %rsp"
			:
			: "r"(mm::UNSAFE_PERMISSION_OUT),"r"(__current_rsp)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		set_pkey_on_page_table_entry::<BasePageSize>(align_down!(__current_rsp, 4096), __count, SAFE_MEM_REGION);
		temp_ret
	}};

	($p:tt.$f:ident($($x:tt)*)) => {{
		//unsafe{ ::UNSAFE_COUNTER += 1; }
		use x86_64::kernel::percore::core_scheduler;
		use x86_64::mm::paging::{BasePageSize, set_pkey_on_page_table_entry};
		use mm::{SAFE_MEM_REGION, SHARED_MEM_REGION};
        use config::DEFAULT_STACK_SIZE;

		let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;
		let mut __current_rbp: usize = 0;
		let mut __current_rsp: usize = 0;
		let mut __count:usize = 0;

		asm!("mov %rbp, $0;
			  mov %rsp, $1"
			: "=r"(__current_rbp), "=r"(__current_rsp)
			:
			: "rbp", "rsp"
			: "volatile");

		__count = (align_up!(__current_rbp, 4096) - align_down!(__current_rsp, 4096))/4096;
		set_pkey_on_page_table_entry::<BasePageSize>(align_down!(__current_rsp, 4096), __count, SHARED_MEM_REGION);

		asm!("mov $0, %rsp;
			  xor %ecx, %ecx;
			  rdpkru;
			  or $1, %eax;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			: 
			: "r"(__isolated_stack),"r"(mm::UNSAFE_PERMISSION_IN)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $p.$f($($x)*);

		asm!("xor %ecx, %ecx;
			  rdpkru;
			  and $0, %eax;		
			  xor %edx, %edx;
			  wrpkru;
			  lfence;
			  mov $1, %rsp"
			:
			: "r"(mm::UNSAFE_PERMISSION_OUT),"r"(__current_rsp)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		set_pkey_on_page_table_entry::<BasePageSize>(align_down!(__current_rsp, 4096), __count, SAFE_MEM_REGION);
		temp_ret
	}};
}

macro_rules! isolate_function_strong {
	($f:ident($($x:tt)*)) => {{
		//unsafe{ ::UNSAFE_COUNTER += 1; }
		use x86_64::kernel::percore::core_scheduler;
        use config::DEFAULT_STACK_SIZE;
		let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;
		let mut __current_rsp: usize = 0;

		asm!("mov %rsp, $0;
			  mov $1, %rsp;
			  xor %ecx, %ecx;
			  rdpkru;
			  or $2, %eax;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			: "=r"(__current_rsp)
			: "r"(__isolated_stack),"r"(mm::UNSAFE_PERMISSION_IN)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $f($($x)*);

		asm!("xor %ecx, %ecx;
			  rdpkru;
			  and $0, %eax;
			  xor %edx, %edx;
			  wrpkru;
			  lfence;
			  mov $1, %rsp"
			:
			: "r"(mm::UNSAFE_PERMISSION_OUT),"r"(__current_rsp)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		temp_ret
	}};
		
	($p:tt.$f:ident($($x:tt)*)) => {{
		//unsafe{ ::UNSAFE_COUNTER += 1; }
		use x86_64::kernel::percore::core_scheduler;
        use config::DEFAULT_STACK_SIZE;
		let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;
		let mut __current_rsp: usize = 0;

		asm!("mov %rsp, $0;
			  mov $1, %rsp;
			  xor %ecx, %ecx;
			  rdpkru;
			  or $2, %eax;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			: "=r"(__current_rsp)
			: "r"(__isolated_stack),"r"(mm::UNSAFE_PERMISSION_IN)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $p.$f($($x)*);

		asm!("xor %ecx, %ecx;
			  rdpkru;
			  and $0, %eax;
			  xor %edx, %edx;
			  wrpkru;
			  lfence;
			  mov $1, %rsp"
			:
			: "r"(mm::UNSAFE_PERMISSION_OUT),"r"(__current_rsp)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		temp_ret
	}};

	($p:tt::$f:ident($($x:tt)*)) => {{
		//unsafe{ ::UNSAFE_COUNTER += 1; }
		use x86_64::kernel::percore::core_scheduler;
        use config::DEFAULT_STACK_SIZE;
		let __isolated_stack = core_scheduler().current_task.borrow().stacks.isolated_stack + DEFAULT_STACK_SIZE;
		let mut __current_rsp: usize = 0;

		asm!("mov %rsp, $0;
			  mov $1, %rsp;
			  xor %ecx, %ecx;
			  rdpkru;
			  or $2, %eax;
			  xor %edx, %edx;
			  wrpkru;
			  lfence"
			: "=r"(__current_rsp)
			: "r"(__isolated_stack),"r"(mm::UNSAFE_PERMISSION_IN)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		let temp_ret = $p::$f($($x)*);

		asm!("xor %ecx, %ecx;
			  rdpkru;
			  and $0, %eax;
			  xor %edx, %edx;
			  wrpkru;
			  lfence;
			  mov $1, %rsp"
			:
			: "r"(mm::UNSAFE_PERMISSION_OUT),"r"(__current_rsp)
			: "rsp", "eax", "ecx", "edx"
			: "volatile");

		temp_ret
	}};
}
