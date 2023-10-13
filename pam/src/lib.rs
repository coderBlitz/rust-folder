mod macros;
mod sys;

use macros::*;

use std::{
	ffi::{self, CString, CStr},
	mem::MaybeUninit,
};

use libc;


/*** Types
***/
map_enum_i32! (
	/// PAM result/return codes.
	///
	PamResult,

	Success => sys::PAM_SUCCESS,
	OpenErr => sys::PAM_OPEN_ERR,
	SymbolErr => sys::PAM_SYMBOL_ERR,
	ServiceErr => sys::PAM_SERVICE_ERR,
	SystemErr => sys::PAM_SYSTEM_ERR,
	BufErr => sys::PAM_BUF_ERR,
	PermDenied => sys::PAM_PERM_DENIED,
	AuthErr => sys::PAM_AUTH_ERR,
	CredInsufficient => sys::PAM_CRED_INSUFFICIENT,
	AuthInfoUnavail => sys::PAM_AUTHINFO_UNAVAIL,
	UserUnknown => sys::PAM_USER_UNKNOWN,
	MaxTries => sys::PAM_MAXTRIES,
	NewAuthTokReqd => sys::PAM_NEW_AUTHTOK_REQD,
	AcctExpired => sys::PAM_ACCT_EXPIRED,
	SessionErr => sys::PAM_SESSION_ERR,
	CredUnavail => sys::PAM_CRED_UNAVAIL,
	CredExpired => sys::PAM_CRED_EXPIRED,
	CredErr => sys::PAM_CRED_ERR,
	NoModuleData => sys::PAM_NO_MODULE_DATA,
	ConvErr => sys::PAM_CONV_ERR,
	AuthTokErr => sys::PAM_AUTHTOK_ERR,
	AuthTokRecoveryErr => sys::PAM_AUTHTOK_RECOVERY_ERR,
	AuthTokLockBusy => sys::PAM_AUTHTOK_LOCK_BUSY,
	AuthTokDisableAging => sys::PAM_AUTHTOK_DISABLE_AGING,
	TryAgain => sys::PAM_TRY_AGAIN,
	Ignore => sys::PAM_IGNORE,
	Abort => sys::PAM_ABORT,
	AuthTokExpired => sys::PAM_AUTHTOK_EXPIRED,
	ModuleUnknown => sys::PAM_MODULE_UNKNOWN,
	BadItem => sys::PAM_BAD_ITEM,
	ConvAgain => sys::PAM_CONV_AGAIN,
	Incomplete => sys::PAM_INCOMPLETE,
);

map_enum_i32!(
	/// Items associated with a pam transaction.
	///
	PamItemType,

	/// PAM service name, as given to `pam_start(3)`.
	Service => sys::PAM_SERVICE,
	/// The username (post-authentication) that is allowed to use a service.
	User => sys::PAM_USER,
	/// Terminal name of client application, prefixed by `/dev/` for device files.
	Tty => sys::PAM_TTY,
	/// The requesting hostname (machine from which `Ruser` is requesting service).
	Rhost => sys::PAM_RHOST,
	/// The conversation function.
	Conv => sys::PAM_CONV,
	/// Authentication token (often a password).
	AuthTok => sys::PAM_AUTHTOK,
	/// Old authentication token (such as when changing password).
	OldAuthTok => sys::PAM_OLDAUTHTOK,
	/// The requesting user name (local for local requester, remote for remote).
	Ruser => sys::PAM_RUSER,
	/// The string used when prompting for a user's name. Defaults to localized "login: ".
	UserPrompt => sys::PAM_USER_PROMPT,

	// Linux-PAM extensions
	/// Function pointer to redirect centrally managed failure delays.
	FailDelay => sys::PAM_FAIL_DELAY,
	XDisplay => sys::PAM_XDISPLAY,
	XAuthData => sys::PAM_XAUTHDATA,
	AuthTokType => sys::PAM_AUTHTOK_TYPE,
);
impl From<PamItem> for PamItemType {
	fn from(item: PamItem) -> Self {
		match item {
			PamItem::Service(_) => PamItemType::Service,
			PamItem::User(_) => PamItemType::User,
			PamItem::UserPrompt(_) => PamItemType::UserPrompt,
			PamItem::Tty(_) => PamItemType::Tty,
			PamItem::Ruser(_) => PamItemType::Ruser,
			PamItem::Rhost(_) => PamItemType::Rhost,
			PamItem::AuthTok(_) => PamItemType::AuthTok,
			PamItem::OldAuthTok(_) => PamItemType::OldAuthTok,
			PamItem::Conv(_) => PamItemType::Conv,
		}
	}
}

map_enum_i32!(
	/// All supported conversation types.
	///
	PamConvType,

	PromptEchoOff => sys::PAM_PROMPT_ECHO_OFF,
	PromptEchoOn => sys::PAM_PROMPT_ECHO_ON,
	ErrorMsg => sys::PAM_ERROR_MSG,
	TextInfo => sys::PAM_TEXT_INFO,
	// Linux-PAM specific
	RadioType => sys::PAM_RADIO_TYPE,
	BinaryPrompt => sys::PAM_BINARY_PROMPT,
);

/// A PAM item used with [pam_get_item] and [pam_set_item].
///
pub enum PamItem {
	Service(String),
	User(String),
	UserPrompt(String),
	Tty(String),
	Ruser(String),
	Rhost(String),
	AuthTok(String),
	OldAuthTok(String),
	Conv(PamConv),
}

/// Newtype wrapper for [pam_conv].
///
// TODO: Allow construction of PamConv from function/closure + data.
pub struct PamConv(sys::pam_conv);
impl PamConv {
	/// Safe call method for contained PAM conversation function.
	///
	pub fn call(&self, conv_type: PamConvType, prompt: &str) -> Result<String, PamResult> {
		// Convert prompt to CString for function call
		let text = CString::new(prompt).unwrap();
		let msg = sys::pam_message {
			msg_style: conv_type as i32,
			msg: text.as_ptr(),
		};
		let msg_p = &msg as *const sys::pam_message;

		// Create response pointer
		let mut resp: MaybeUninit<*const sys::pam_response> = MaybeUninit::uninit();

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

/*** Functions
***/
/// Gets the item [PamItemType] associated with the PAM handle `pamh`.
///
pub fn pam_get_item(pamh: *const (), item_type: PamItemType) -> Result<PamItem, PamResult> {
	match item_type {
		PamItemType::Conv => {
			let mut item: MaybeUninit<*const sys::pam_conv> = MaybeUninit::uninit();
			let res = unsafe { sys::pam_get_item(pamh, item_type as i32, item.as_mut_ptr() as _) };

			if res == PamResult::Success as i32 {
				// SAFETY: Successful return implies struct pointer and struct are valid.
				let conv_struct = unsafe { item.assume_init() };

				unsafe { Ok(PamItem::Conv(PamConv(*conv_struct))) }
			} else {
				Err(PamResult::try_from(res).unwrap())
			}
		},
		// For everything else (all c-string values).
		_ => {
			let mut item: MaybeUninit<*const ffi::c_char> = MaybeUninit::uninit();
			let res = unsafe { sys::pam_get_item(pamh, item_type as i32, item.as_mut_ptr() as _) };

			if res == PamResult::Success as i32 {
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
				Err(PamResult::try_from(res).unwrap())
			}
		}
	}
}

/// Sets the [PamItem] value associated with the given PAM handle `pamh`.
///
// TODO: Logic for conv function
pub fn pam_set_item(pamh: *const (), item: PamItem) -> Result<(), PamResult> {
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
			let res = unsafe { sys::pam_set_item(pamh, PamItemType::from(item) as i32, s_cstr.as_ptr() as _) };

			if res == PamResult::Success as i32 {
				Ok(())
			} else {
				Err(PamResult::try_from(res).unwrap())
			}
		},
		PamItem::Conv(ref _c) => todo!(),
	}
}

/// Gets the username passed to `pam_start(3)`, or [pam_get_item] with [PamItemType::User], or prompts the user.
///
// TODO: Return str/OsStr/CStr instead?
pub fn pam_get_user<S: AsRef<str>>(pamh: *const (), prompt: S) -> Result<String, PamResult> {
	fn inner(pamh: *const (), p: &str) -> Result<String, PamResult> {
		let c_prompt = CString::new(p).expect("Prompt should not contain null bytes.");
		let mut user_p: MaybeUninit<*const ffi::c_char> = MaybeUninit::uninit();
		let res = unsafe { sys::pam_get_user(pamh, user_p.as_mut_ptr(), c_prompt.as_ptr()) };

		match PamResult::try_from(res).unwrap() {
			PamResult::Success => {
				let user_p = unsafe { user_p.assume_init() };
				let user_cstr = unsafe { CStr::from_ptr(user_p) };
				Ok(user_cstr.to_str().unwrap_or_default().to_string())
			},
			e => Err(e)
		}
	}
	inner(pamh, prompt.as_ref())
}

/// Cleanup function for [sys::pam_set_data].
///
/// Simply re-boxes data, then drops.
// Turbofish syntax required to match cleanup function signature for `pam_set_data`.
extern "C" fn data_cleanup<D>(_pamh: *const (), data: *mut (), _error_status: i32) {
	// SAFETY: [pam_set_data] uses [Box::into_raw], and [pam_get_data] returns
	//          references to avoid prematurely dropping/invalidating data.
	unsafe { Box::<D>::from_raw(data as _) };
}

/// Get data associated with the handle `pamh`.
///
/// # Safety
/// If an entry for `data_name` exists, the type must match `D` and the data must
///  have been set using [pam_set_data]. Additionally, the underlying data will be
///  consumed unless [std::mem::forget] or similar steps are taken to avoid
///  invalidating the data.
pub unsafe fn pam_get_data<D, S: AsRef<str>>(pamh: *const (), data_name: S) -> Result<&'static D, PamResult> {
	fn inner(pamh: *const (), name: &str) -> Result<*const (), PamResult> {
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

	match inner(pamh, data_name.as_ref()) {
		Ok(d) => {
			// SAFETY: Caller invariant to use [pam_set_data], which uses [Box::into_raw].
			unsafe { Ok((d as *const D).as_ref().unwrap()) }
		},
		Err(e) => Err(e),
	}
}

/// Associate an object with the handle `pamh`, retrievable using [pam_get_data].
///
pub fn pam_set_data<D, S: AsRef<str>>(pamh: *const (), data_name: S, data: D) -> Result<(), PamResult> {
	let c_name = CString::new(data_name.as_ref()).unwrap();
	let dat = Box::into_raw(Box::new(data));
	let res = unsafe { sys::pam_set_data(pamh, c_name.as_ptr(), dat as _, data_cleanup::<D>) };

	match PamResult::try_from(res).unwrap() {
		PamResult::Success => Ok(()),
		e => Err(e),
	}
}

pub fn pam_prompt(pamh: *const (), conv_type: PamConvType, prompt: &str) -> Result<String, PamResult> {
	let c_prompt = CString::new(prompt).unwrap();
	let mut resp: MaybeUninit<*const ffi::c_char> = MaybeUninit::uninit();
	let res = unsafe { sys::pam_prompt(pamh, conv_type as i32, resp.as_mut_ptr() as _, c_prompt.as_ptr()) };

	if res == PamResult::Success as i32 {
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
		Err(PamResult::try_from(res).unwrap())
	}
}
