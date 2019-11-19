// Copyright (c) 2018 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use arch;
use synch::spinlock::Spinlock;
use mm;

safe_global_var!(static PARK_MILLER_LEHMER_SEED: Spinlock<u32> = Spinlock::new(0));

fn generate_park_miller_lehmer_random_number() -> u32 {
	let mut seed = PARK_MILLER_LEHMER_SEED.lock();
	let random = ((u64::from(*seed) * 48271) % 2_147_483_647) as u32;
	*seed = random;
	random
}

#[no_mangle]
pub extern "C" fn sys_rand() -> u32 {
    kernel_enter!("sys_rand");        
	if let Some(value) = arch::processor::generate_random_number() {
        kernel_exit!("sys_rand");
		value
	} else {
		let random = generate_park_miller_lehmer_random_number();
		kernel_exit!("sys_rand");
        random
	}
}

pub fn random_init() {
	*PARK_MILLER_LEHMER_SEED.lock() = arch::processor::get_timestamp() as u32;
}
