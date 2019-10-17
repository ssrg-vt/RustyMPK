// Copyright (c) 2018 Colin Finck, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use arch;
use arch::x86_64::mm::mpk::MpkPerm;

#[no_mangle]
pub extern "C" fn sys_mpk_swap_pkru(new_pkru: u32) -> u32 {
	return arch::mm::mpk::mpk_swap_pkru(new_pkru);
}

#[no_mangle]
pub extern "C" fn sys_mpk_mem_set_key(addr: usize, size: usize, key: u8) -> i32 {
	return arch::mm::mpk::mpk_mem_set_key(addr, size, key);
}

#[no_mangle]
pub extern "C" fn sys_mpk_set_perm(key: u8, perm: MpkPerm) -> i32 {
    return arch::mm::mpk::mpk_set_perm(key, perm);
}

#[no_mangle]
pub extern "C" fn sys_mpk_clear_pkru() {
    return arch::mm::mpk::mpk_clear_pkru();
}

#[no_mangle]
pub extern "C" fn sys_mpk_get_pkru() -> u32 {
    return arch::mm::mpk::mpk_get_pkru();
}

#[no_mangle]
pub extern "C" fn sys_mpk_set_pkru(val: u32) {
    arch::mm::mpk::mpk_set_pkru(val);
}
