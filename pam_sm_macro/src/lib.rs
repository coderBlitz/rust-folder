use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{ItemFn, parse_macro_input};
use syn::spanned::Spanned;


#[proc_macro_attribute]
pub fn acct_mgmt(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let input = parse_macro_input!(item as ItemFn);
	let fn_ident = &input.sig.ident;

	let input_span = input.sig.span();
	let output = quote_spanned! {input_span=>
		// Original function verbatim
		#input

		// PAM native function wrapper
		#[no_mangle]
		pub extern "C" fn pam_sm_acct_mgmt(
			pamh: *const (),
			flags: std::ffi::c_int,
			argc: std::ffi::c_int,
			argv: *const *const std::ffi::c_char
		) -> std::ffi::c_int {
			// Collect arguments into vector
			let mut args: std::vec::Vec<&CStr> = std::vec::Vec::with_capacity(argc as usize);
			for i in 0..argc {
				unsafe { args.push(std::ffi::CStr::from_ptr(argv.add(i as usize) as _)) };
			}

			// Type check (more useful errors than just calling function).
			type Expected = fn(pam::PamHandle, i32, std::vec::Vec<&CStr>) -> pam::PamResult;
			let f: Expected = #fn_ident;

			// Call function with values, and convert return value
			f(pamh, flags, args) as i32
		}
	};

	output.into()
}

#[proc_macro_attribute]
pub fn authenticate(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let input = parse_macro_input!(item as ItemFn);
	let fn_ident = &input.sig.ident;

	let input_span = input.sig.span();
	let output = quote_spanned! {input_span=>
		// Original function verbatim
		#input

		// PAM native function wrapper
		#[no_mangle]
		pub extern "C" fn pam_sm_authenticate(
			pamh: *const (),
			flags: std::ffi::c_int,
			argc: std::ffi::c_int,
			argv: *const *const std::ffi::c_char
		) -> std::ffi::c_int {
			// Collect arguments into vector
			let mut args: std::vec::Vec<&CStr> = std::vec::Vec::with_capacity(argc as usize);
			for i in 0..argc {
				unsafe { args.push(std::ffi::CStr::from_ptr(argv.add(i as usize) as _)) };
			}

			// Type check (more useful errors than just calling function).
			type Expected = fn(pam::PamHandle, i32, std::vec::Vec<&CStr>) -> pam::PamResult;
			let f: Expected = #fn_ident;

			// Call function with values, and convert return value
			f(pamh, flags, args) as i32
		}
	};

	output.into()
}

#[proc_macro_attribute]
pub fn chauthtok(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let input = parse_macro_input!(item as ItemFn);
	let fn_ident = &input.sig.ident;

	let input_span = input.sig.span();
	let output = quote_spanned! {input_span=>
		// Original function verbatim
		#input

		// PAM native function wrapper
		#[no_mangle]
		pub extern "C" fn pam_sm_chauthtok(
			pamh: *const (),
			flags: std::ffi::c_int,
			argc: std::ffi::c_int,
			argv: *const *const std::ffi::c_char
		) -> std::ffi::c_int {
			// Collect arguments into vector
			let mut args: std::vec::Vec<&CStr> = std::vec::Vec::with_capacity(argc as usize);
			for i in 0..argc {
				unsafe { args.push(std::ffi::CStr::from_ptr(argv.add(i as usize) as _)) };
			}

			// Type check (more useful errors than just calling function).
			type Expected = fn(pam::PamHandle, i32, std::vec::Vec<&CStr>) -> pam::PamResult;
			let f: Expected = #fn_ident;

			// Call function with values, and convert return value
			f(pamh, flags, args) as i32
		}
	};

	output.into()
}

#[proc_macro_attribute]
pub fn close_session(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let input = parse_macro_input!(item as ItemFn);
	let fn_ident = &input.sig.ident;

	let input_span = input.sig.span();
	let output = quote_spanned! {input_span=>
		// Original function verbatim
		#input

		// PAM native function wrapper
		#[no_mangle]
		pub extern "C" fn pam_sm_close_session(
			pamh: *const (),
			flags: std::ffi::c_int,
			argc: std::ffi::c_int,
			argv: *const *const std::ffi::c_char
		) -> std::ffi::c_int {
			// Collect arguments into vector
			let mut args: std::vec::Vec<&CStr> = std::vec::Vec::with_capacity(argc as usize);
			for i in 0..argc {
				unsafe { args.push(std::ffi::CStr::from_ptr(argv.add(i as usize) as _)) };
			}

			// Type check (more useful errors than just calling function).
			type Expected = fn(pam::PamHandle, i32, std::vec::Vec<&CStr>) -> pam::PamResult;
			let f: Expected = #fn_ident;

			// Call function with values, and convert return value
			f(pamh, flags, args) as i32
		}
	};

	output.into()
}

#[proc_macro_attribute]
pub fn open_session(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let input = parse_macro_input!(item as ItemFn);
	let fn_ident = &input.sig.ident;

	let input_span = input.sig.span();
	let output = quote_spanned! {input_span=>
		// Original function verbatim
		#input

		// PAM native function wrapper
		#[no_mangle]
		pub extern "C" fn pam_sm_open_session(
			pamh: *const (),
			flags: std::ffi::c_int,
			argc: std::ffi::c_int,
			argv: *const *const std::ffi::c_char
		) -> std::ffi::c_int {
			// Collect arguments into vector
			let mut args: std::vec::Vec<&CStr> = std::vec::Vec::with_capacity(argc as usize);
			for i in 0..argc {
				unsafe { args.push(std::ffi::CStr::from_ptr(argv.add(i as usize) as _)) };
			}

			// Type check (more useful errors than just calling function).
			type Expected = fn(pam::PamHandle, i32, std::vec::Vec<&CStr>) -> pam::PamResult;
			let f: Expected = #fn_ident;

			// Call function with values, and convert return value
			f(pamh, flags, args) as i32
		}
	};

	output.into()
}

#[proc_macro_attribute]
pub fn setcred(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let input = parse_macro_input!(item as ItemFn);
	let fn_ident = &input.sig.ident;

	let input_span = input.sig.span();
	let output = quote_spanned! {input_span=>
		// Original function verbatim
		#input

		// PAM native function wrapper
		#[no_mangle]
		pub extern "C" fn pam_sm_setcred(
			pamh: *const (),
			flags: std::ffi::c_int,
			argc: std::ffi::c_int,
			argv: *const *const std::ffi::c_char
		) -> std::ffi::c_int {
			// Collect arguments into vector
			let mut args: std::vec::Vec<&CStr> = std::vec::Vec::with_capacity(argc as usize);
			for i in 0..argc {
				unsafe { args.push(std::ffi::CStr::from_ptr(argv.add(i as usize) as _)) };
			}

			// Type check (more useful errors than just calling function).
			type Expected = fn(pam::PamHandle, i32, std::vec::Vec<&CStr>) -> pam::PamResult;
			let f: Expected = #fn_ident;

			// Call function with values, and convert return value
			f(pamh, flags, args) as i32
		}
	};

	output.into()
}
