use std::ffi;

extern {
	fn printf(fmt: *const ffi::c_char, ...) -> ffi::c_int;
}

fn main() {
	let s = ffi::CString::new("Hello world! Hello %s!\n").expect("Convert failed");
	let d = ffi::CString::new("Test").expect("Convert failed");
	unsafe {
		printf(s.as_ptr(), d.as_ptr());
	}
}
