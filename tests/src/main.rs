extern crate http;
extern crate rayon;

mod tests;

use tests::*;

fn test_result<T>(result: Result<(), T>) -> &'static str {
	match result {
		Ok(_) => "ok",
		Err(_) => "failed!",
	}
}

fn main() {
	println!("Test {} ... {}", stringify!(hello), test_result(hello()));
	println!(
		"Test {} ... {}",
		stringify!(test_pkru_context_switch),
		test_result(test_pkru_context_switch())
	);
/*
        unsafe {
            let addr = 0x405000;
            let p_addr: *mut u8 = addr as *mut u8;
            println!("p: {}", *p_addr);
        }

	println!(
		"Test {} ... {}",
		stringify!(print_argv),
		test_result(print_argv())
	);
	println!(
		"Test {} ... {}",
		stringify!(print_env),
		test_result(print_env())
	);
	println!(
		"Test {} ... {}",
		stringify!(read_file),
		test_result(read_file())
	);
	println!(
		"Test {} ... {}",
		stringify!(create_file),
		test_result(create_file())
	);
	println!(
		"Test {} ... {}",
		stringify!(threading),
		test_result(threading())
	);
	println!(
		"Test {} ... {}",
		stringify!(pi_sequential),
		test_result(pi_sequential(5000000))
	);
	println!(
		"Test {} ... {}",
		stringify!(pi_parallel),
		test_result(pi_parallel(2, 5000000))
	);
	/*println!(
		"Test {} ... {}",
		stringify!(laplace),
		test_result(laplace(128, 128))
	);
	println!(
		"Test {} ... {}",
		stringify!(test_matmul_strassen),
		test_result(test_matmul_strassen())
	);
	println!(
		"Test {} ... {}",
		stringify!(thread_creation),
		test_result(thread_creation())
	);*/
	println!(
		"Test {} ... {}",
		stringify!(bench_sched_one_thread),
		test_result(bench_sched_one_thread())
	);
	println!(
		"Test {} ... {}",
		stringify!(bench_sched_two_threads),
		test_result(bench_sched_two_threads())
	);
	println!(
		"Test {} ... {}",
		stringify!(test_http_request),
		test_result(test_http_request())
	);
*/
}
