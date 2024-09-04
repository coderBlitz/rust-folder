//! PAM service module function macros.
//!
//! Attribute macros that define the exported PAM module functions as well as
//!  providing a nicer function signature to the end user.
//!
//! # Function signature
//! Functions must have the signature
//!  `fn(pam::PamHandle, i32, Vec<&CStr>) -> pam::PamResult` in order to be
//!  valid.

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
					use std::mem::ManuallyDrop;
					use std::ops::DerefMut;

					// Collect arguments into vector
					let args: Vec<&CStr> = unsafe {
						(0..argc).map(|i| CStr::from_ptr(argv.add(i as usize) as _)).collect()
					};

					// Type check (more useful errors than just calling function).
					type Expected = fn(&mut pam::PamHandle, i32, &[&CStr]) -> pam::PamResult;
					let f: Expected = #fn_ident;

					// Create pam handle object. Intentionally do not drop pam handle since modules
					//  should not close the pam handle passed in.
					// SAFETY: Pointer is guaranteed valid as a function parameter.
					let mut pamh = ManuallyDrop::new(unsafe { pam::PamHandle::from_raw(pamh) });

					// Call function with values, and convert return value
					f(pamh.deref_mut(), flags, &args[..]) as i32
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
