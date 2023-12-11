use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{ItemFn, parse_macro_input};
use syn::spanned::Spanned;

// NOTE: Two parameters required because anything_$bla will always fail (leading anything), so $bla is required.
macro_rules! pam_wrapper {
	($attr_name:ident, $lib_name:ident) => {
		#[proc_macro_attribute]
		pub fn $attr_name(_attr: TokenStream, item: TokenStream) -> TokenStream {
			let input = parse_macro_input!(item as ItemFn);
			let fn_ident = &input.sig.ident;

			let input_span = input.sig.span();
			let output = quote_spanned! {input_span=>
				// Original function verbatim
				#input

				// PAM native function wrapper
				#[no_mangle]
				pub extern "C" fn $lib_name(
					pamh: *const (),
					flags: std::ffi::c_int,
					argc: std::ffi::c_int,
					argv: *const *const std::ffi::c_char
				) -> std::ffi::c_int {
					use std::ffi::CStr;
					use std::vec::Vec;

					// Collect arguments into vector
					let args: Vec<&CStr> = unsafe {
						(0..argc).map(|i| CStr::from_ptr(argv.add(i as usize) as _)).collect()
					};

					// Type check (more useful errors than just calling function).
					type Expected = fn(pam::PamHandle, i32, Vec<&CStr>) -> pam::PamResult;
					let f: Expected = #fn_ident;

					// Call function with values, and convert return value
					f(pamh, flags, args) as i32
				}
			};

			output.into()
		}
	};
}

pam_wrapper!(acct_mgmt, pam_sm_acct_mgmt);
pam_wrapper!(authenticate, pam_sm_authenticate);
pam_wrapper!(chauthtok, pam_sm_chauthtok);
pam_wrapper!(close_session, pam_sm_close_session);
pam_wrapper!(open_session, pam_sm_open_session);
pam_wrapper!(setcred, pam_sm_setcred);
