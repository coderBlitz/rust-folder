pub use pam_sm_macro::*;

use types::*;

use std::{
	borrow::Cow,
	ffi::{self, CString, CStr},
	marker,
	mem::MaybeUninit,
};
use libc;

mod macros;
mod sys;
pub mod types;

type Result<T> = std::result::Result<T, PamResult>;


#[derive(Clone, Debug, Hash, PartialEq)]
pub struct PamMessage<'s> {
	pub style: PamConvType,
	pub msg: Cow<'s, str>,
}

// TODO: Change string slice to some custom type that tries to allocate as
//        responses are added, allowing user to handle allocation errors.
struct ConvData<'s, T: Sized> {
	fun: fn(&[PamMessage<'s>], &mut [String], &mut T) -> PamResult,
	data: T,
}

/// Wrapper for user conv function, used in [PamConv] as the `pam_conv.conv`
///  function.
fn conv_wrapper<'s, T>(
	count: i32,
	msgs: *const *const sys::pam_message,
	responses: *mut *mut sys::pam_response,
	data: *const ()
 ) -> i32 {
	// Early check of response pointer.
	if responses.is_null() {
		return sys::PAM_BUF_ERR
	}

	// Allocate responses array early and return PAM_BUF_ERR if so. Saves
	//  effort of doing anything if this fails.
	// SAFETY: Responses is non-null after above conditional.
	unsafe {
		*responses = libc::malloc(count as usize * std::mem::size_of::<*mut sys::pam_response>()) as _;
		if (*responses).is_null() {
			return sys::PAM_BUF_ERR
		}
	}

	// TODO: Box ConvData.
	let conv_data = unsafe { &mut (data as *mut ConvData<'s, T>).read() };

	// Initialize arrays for user fn
	let mut pam_messages = Vec::new();
	let pam_responses = &mut Vec::with_capacity(count as usize);
	pam_responses.resize(count as usize, String::new());

	// Convert input data to rust data for user fn.
	for i in (0..count) {
		let msg: &sys::pam_message = unsafe {
			&(*msgs).add(i as usize).read()
		};

		// Convert the style.
		let style = match PamConvType::try_from(msg.msg_style) {
			Ok(v) => v,
			Err(_) => PamConvType::TextInfo,
		};

		// Get message as string slice
		let text = unsafe {
			String::from_utf8_lossy(CStr::from_ptr(msg.msg).to_bytes())
		};

		// Construct message wrapper.
		let message = PamMessage {
			style,
			msg: text,
		};

		// Put wrapper into vector for user.
		pam_messages.push(message);
	}

	// Call user fn.
	let res = (conv_data.fun)(&pam_messages, pam_responses, &mut conv_data.data);

	// If successful, convert responses to libc-allocated array of strings.
	// Else leave `*responses` untouched.
	if res == PamResult::Success {
		unsafe {
			// Iterate responses, then allocate and copy each.
			for i in (0..count) {
				// Allocate response
				let resp = &pam_responses[i as usize];
				let c_resp = libc::malloc(resp.len() + 1) as *mut u8; // THIS IS BAD. See TODO on [ConvData].

				// Copy response
				std::ptr::copy_nonoverlapping(resp.as_ptr(), c_resp, resp.len());
				c_resp.add(resp.len()).write(0); // Null-terminator.

				// Add response to `*responses`
				*(*responses).add(i as usize) = sys::pam_response {
					resp: c_resp as *mut i8,
					_resp_retcode: 0,
				};
			}
		}
	}

	res as i32
 }

/// Newtype wrapper for [pam_conv].
///
// TODO: Allow construction of PamConv from function/closure + data.
// TODO: Implement [Drop].
pub struct PamConv(sys::pam_conv);
impl PamConv {
	// TODO: Complete. Change return type to [Self].
	pub fn new<'s, T>(conv_fn: fn(&[PamMessage<'s>],&mut [String],&mut T), data: T) {

	}

	/// Safe call method for contained PAM conversation function.
	///
	pub fn call(&self, conv_type: PamConvType, prompt: &str) -> Result<String> {
		// Convert prompt to CString for function call
		let text = CString::new(prompt).unwrap();
		let msg = sys::pam_message {
			msg_style: conv_type as i32,
			msg: text.as_ptr(),
		};
		let msg_p = &msg as *const sys::pam_message;

		// Create response pointer
		let mut resp: MaybeUninit<*mut sys::pam_response> = MaybeUninit::uninit();

		// Call conv and then extract response if successful
		let res = unsafe { (self.0.conv)(1, &msg_p, resp.as_mut_ptr(), self.0.appdata_ptr) };
		if res == PamResult::Success as i32 {
			// SAFETY: Valid pointer on success.
			let resp = unsafe { resp.assume_init() };

			// Extract/copy response to owned string
			// SAFETY: `resp` points to a valid struct because successful return code.
			let s = unsafe {
				if (*resp).resp.is_null() {
					String::new()
				} else {
					CStr::from_ptr((*resp).resp).to_str().unwrap_or_default().to_owned()
				}
			};

			// Free the given response string.
			// NOTE: Since resp was allocated with malloc (libc), can't just hold in Box or CString.
			unsafe {
				libc::free((*resp).resp as _);
				libc::free(resp as _);
			}

			// Return response.
			Ok(s)
		} else {
			Err(PamResult::try_from(res).unwrap())
		}
	}
}

/// PAM handle for applications.
///
/// Because the PAM library is not thread-safe with respect to a given handle,
///  all functions require mutable references to ensure safe usage.
pub struct PamHandle<'d>(*const (), PamResult, marker::PhantomData<&'d i32>);
impl<'d> PamHandle<'d> {
	/// Construct an instance using a raw handle as obtained through the
	///  `pam_sm_*` functions.
	///
	/// # Safety
	/// `ptr` must be the `pamh` argument given to any `pam_sm_*` function.
	// TODO: Get the conv item and store for uniformity with new() and ease of
	//        dropping.
	pub unsafe fn from_raw(ptr: *const ()) -> Self {
		Self(ptr, PamResult::Success, marker::PhantomData)
	}

	/// Start a new pam session.
	pub fn new(svc: &str, user: &str, conv: &PamConv) -> Result<Self> {
		let svc_c = CString::new(svc).unwrap();
		let user_c = CString::new(user).unwrap();
		let mut handle: MaybeUninit<*const ()> = MaybeUninit::uninit();
		let res = unsafe {
			sys::pam_start(svc_c.as_ptr(), user_c.as_ptr(), &conv.0 as *const _, handle.as_mut_ptr())
		};

		if res == sys::PAM_SUCCESS {
			unsafe { Ok(Self (handle.assume_init(), PamResult::Success, marker::PhantomData)) }
		} else {
			Err(PamResult::try_from(res).unwrap())
		}
	}

	/// Update last return value of self, and return converted value.
	fn ret(&mut self, rv: i32) -> PamResult {
		let r = PamResult::try_from(rv).unwrap();
		self.1 = r;
		r
	}

	/// Gets the item [PamItemType] associated with this handle.
	///
	pub fn pam_get_item(&mut self, item_type: PamItemType) -> Result<PamItem> {
		match item_type {
			PamItemType::Conv => {
				let mut item: MaybeUninit<*const sys::pam_conv> = MaybeUninit::uninit();
				let res = unsafe { self.ret(
					sys::pam_get_item(self.0, item_type as i32, item.as_mut_ptr() as _)
				)};

				if res == PamResult::Success {
					// SAFETY: Successful return implies struct pointer and struct are valid.
					let conv_struct = unsafe { item.assume_init() };

					unsafe { Ok(PamItem::Conv(PamConv(*conv_struct))) }
				} else {
					Err(res)
				}
			},
			// For everything else (all c-string values).
			_ => {
				let mut item: MaybeUninit<*const ffi::c_char> = MaybeUninit::uninit();
				let res = unsafe { self.ret(
					sys::pam_get_item(self.0, item_type as i32, item.as_mut_ptr() as _)
				)};

				if res == PamResult::Success {
					// SAFETY: Successful return code implies data is valid C-string.
					let item = unsafe { item.assume_init() };
					let s = match item.is_null() {
						true => String::new(),
						false => {
							let item_cstr = unsafe { CStr::from_ptr(item) };
							item_cstr.to_str().unwrap_or_default().to_string()
						},
					};

					match item_type {
						PamItemType::Service => Ok(PamItem::Service(s)),
						PamItemType::User => Ok(PamItem::User(s)),
						PamItemType::UserPrompt => Ok(PamItem::UserPrompt(s)),
						PamItemType::Tty => Ok(PamItem::Tty(s)),
						PamItemType::Ruser => Ok(PamItem::Ruser(s)),
						PamItemType::Rhost => Ok(PamItem::Rhost(s)),
						PamItemType::AuthTok => Ok(PamItem::AuthTok(s)),
						PamItemType::OldAuthTok => Ok(PamItem::OldAuthTok(s)),
						_ => panic!("Unexpected PamItemType."),
					}
				} else {
					Err(res)
				}
			}
		}
	}

	/// Sets the [PamItem] value associated with this handle.
	///
	// TODO: Logic for conv function
	pub fn pam_set_item(&mut self, item: PamItem) -> Result<()> {
		match item {
			PamItem::Service(ref s)
			| PamItem::User(ref s)
			| PamItem::UserPrompt(ref s)
			| PamItem::Tty(ref s)
			| PamItem::Ruser(ref s)
			| PamItem::Rhost(ref s)
			| PamItem::AuthTok(ref s)
			| PamItem::OldAuthTok(ref s) => {
				let s_cstr = CString::new(&s[..]).unwrap();
				let res = unsafe { self.ret(
					sys::pam_set_item(self.0, PamItemType::from(item) as i32, s_cstr.as_ptr() as _)
				)};

				if res == PamResult::Success {
					Ok(())
				} else {
					Err(res)
				}
			},
			PamItem::Conv(ref _c) => todo!(),
		}
	}

	/// Gets the username passed to `pam_start(3)`, or [pam_get_item] with [PamItemType::User], or prompts the user.
	///
	// TODO: Return str/OsStr/CStr instead?
	pub fn pam_get_user<S: AsRef<str>>(&mut self, prompt: S) -> Result<String> {
		let p = prompt.as_ref();
		let c_prompt = CString::new(p).expect("Prompt should not contain null bytes.");
		let mut user_p: MaybeUninit<*const ffi::c_char> = MaybeUninit::uninit();
		let res = unsafe { self.ret(
			sys::pam_get_user(self.0, user_p.as_mut_ptr(), c_prompt.as_ptr())
		)};

		if res == PamResult::Success {
			let user_p = unsafe { user_p.assume_init() };
			let user_cstr = unsafe { CStr::from_ptr(user_p) };
			Ok(user_cstr.to_str().unwrap_or_default().to_string())
		} else {
			Err(res)
		}
	}

	pub fn pam_prompt(&mut self, conv_type: PamConvType, prompt: &str) -> Result<String> {
		let c_prompt = CString::new(prompt).unwrap();
		let mut resp: MaybeUninit<*const ffi::c_char> = MaybeUninit::uninit();
		let res = unsafe { self.ret(
			sys::pam_prompt(self.0, conv_type as i32, resp.as_mut_ptr() as _, c_prompt.as_ptr())
		)};

		if res == PamResult::Success {
			// SAFETY: Successful return code implies data is valid C-string.
			let resp = unsafe { resp.assume_init() };
			let s = match resp.is_null() {
				true => String::new(),
				false => {
					let resp_cstr = unsafe { CStr::from_ptr(resp) };
					resp_cstr.to_str().unwrap_or_default().to_string()
				},
			};

			Ok(s)
		} else {
			Err(res)
		}
	}

	/// Cleanup function for [sys::pam_set_data].
	///
	/// Simply re-boxes data, then drops.
	// Turbofish syntax required to match cleanup function signature for `pam_set_data`.
	extern "C" fn data_cleanup<D>(_pamh: *const (), data: *mut (), _error_status: i32) {
		// SAFETY: [pam_set_data] uses [Box::into_raw], and [pam_get_data] returns
		//          references to avoid prematurely dropping/invalidating data.
		_ = unsafe { Box::<D>::from_raw(data as _) };
	}

	/// Get data associated with this handle. Service module only.
	///
	/// # Safety
	/// If an entry for `data_name` exists, the type must match `D` and the data must
	///  have been set using [pam_set_data].
	pub unsafe fn pam_get_data<D, S: AsRef<str>>(&self, data_name: S) -> Result<&'d D> {
		fn inner(pamh: *const (), name: &str) -> Result<*const ()> {
			let c_name = CString::new(name).unwrap();
			let mut dat: MaybeUninit<*const ()> = MaybeUninit::uninit();
			let res = unsafe { sys::pam_get_data(pamh, c_name.as_ptr(), dat.as_mut_ptr()) };

			if res == PamResult::Success as i32 {
				let dat = unsafe { dat.assume_init() };
				Ok(dat)
			} else {
				Err(PamResult::try_from(res).unwrap())
			}
		}

		match inner(self.0, data_name.as_ref()) {
			Ok(d) => {
				// SAFETY: Caller invariant to use [pam_set_data], which uses [Box::into_raw].
				unsafe { Ok((d as *const D).as_ref().unwrap()) }
			},
			Err(e) => Err(e),
		}
	}

	/// Associate an object with this handle, retrievable using [pam_get_data]. Service module only.
	///
	pub fn pam_set_data<D, S: AsRef<str>>(&self, data_name: S, data: D) -> Result<()> {
		let c_name = CString::new(data_name.as_ref()).unwrap();
		let dat = Box::into_raw(Box::new(data));
		let res = unsafe { sys::pam_set_data(self.0, c_name.as_ptr(), dat as _, Self::data_cleanup::<D>) };

		match PamResult::try_from(res).unwrap() {
			PamResult::Success => Ok(()),
			e => Err(e),
		}
	}
}
impl<'d> Drop for PamHandle<'d> {
	fn drop(&mut self) {
		// TODO: Get conv function pointer to drop [PamConv], if not stored.
		unsafe { sys::pam_end(self.0, self.1 as i32); }
	}
}
