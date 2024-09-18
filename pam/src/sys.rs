#![allow(dead_code)]

use std::ffi::*;

/* Return codes
*/
pub const PAM_SUCCESS: i32 = 0;
pub const PAM_OPEN_ERR: i32 = 1;
pub const PAM_SYMBOL_ERR: i32 = 2;
pub const PAM_SERVICE_ERR: i32 = 3;
pub const PAM_SYSTEM_ERR: i32 = 4;
pub const PAM_BUF_ERR: i32 = 5;
pub const PAM_PERM_DENIED: i32 = 6;
pub const PAM_AUTH_ERR: i32 = 7;
pub const PAM_CRED_INSUFFICIENT: i32 = 8;
pub const PAM_AUTHINFO_UNAVAIL: i32 = 9;
pub const PAM_USER_UNKNOWN: i32 = 10;
pub const PAM_MAXTRIES: i32 = 11;
pub const PAM_NEW_AUTHTOK_REQD: i32 = 12;
pub const PAM_ACCT_EXPIRED: i32 = 13;
pub const PAM_SESSION_ERR: i32 = 14;
pub const PAM_CRED_UNAVAIL: i32 = 15;
pub const PAM_CRED_EXPIRED: i32 = 16;
pub const PAM_CRED_ERR: i32 = 17;
pub const PAM_NO_MODULE_DATA: i32 = 18;
pub const PAM_CONV_ERR: i32 = 19;
pub const PAM_AUTHTOK_ERR: i32 = 20;
pub const PAM_AUTHTOK_RECOVERY_ERR: i32 = 21;
pub const PAM_AUTHTOK_LOCK_BUSY: i32 = 22;
pub const PAM_AUTHTOK_DISABLE_AGING: i32 = 23;
pub const PAM_TRY_AGAIN: i32 = 24;
pub const PAM_IGNORE: i32 = 25;
pub const PAM_ABORT: i32 = 26;
pub const PAM_AUTHTOK_EXPIRED: i32 = 27;
pub const PAM_MODULE_UNKNOWN: i32 = 28;
pub const PAM_BAD_ITEM: i32 = 29;
pub const PAM_CONV_AGAIN: i32 = 30;
pub const PAM_INCOMPLETE: i32 = 31;

pub const PAM_SERVICE: i32 = 1;
pub const PAM_USER: i32 = 2;
pub const PAM_TTY: i32 = 3;
pub const PAM_RHOST: i32 = 4;
pub const PAM_CONV: i32 = 5;
pub const PAM_AUTHTOK: i32 = 6;
pub const PAM_OLDAUTHTOK: i32 = 7;
pub const PAM_RUSER: i32 = 8;
pub const PAM_USER_PROMPT: i32 = 9;
// Linux-PAM extensions
pub const PAM_FAIL_DELAY: i32 = 10;
pub const PAM_XDISPLAY: i32 = 11;
pub const PAM_XAUTHDATA: i32 = 12;
pub const PAM_AUTHTOK_TYPE: i32 = 13;

/* Conversation types
*/
pub const PAM_PROMPT_ECHO_OFF: i32 = 1;
pub const PAM_PROMPT_ECHO_ON: i32 = 2;
pub const PAM_ERROR_MSG: i32 = 3;
pub const PAM_TEXT_INFO: i32 = 4;
// Linux-PAM specific
pub const PAM_RADIO_TYPE: i32 = 5;
pub const PAM_BINARY_PROMPT: i32 = 7;

#[repr(C)]
pub struct pam_message {
	pub msg_style: c_int,
	pub msg: *const c_char,
}
#[repr(C)]
pub struct pam_response {
	pub resp: *mut c_char,
	pub _resp_retcode: c_int, // Unused, should be 0.
}
#[derive(Clone, Copy)]
#[repr(C)]
pub struct pam_conv {
	pub conv: unsafe extern "C" fn(
		num_msg: i32,
		msg: *const *const pam_message,
		resp: *mut *mut pam_response,
		appdata_ptr: *const ()
	) -> i32,
	pub appdata_ptr: *const (),
}

#[link(name = "pam")]
extern "C" {
	pub fn pam_start(
		service_name: *const c_char,
		user: *const c_char,
		pam_conversation: *const pam_conv,
		pamh: *mut *const ()
	) -> i32;
	pub fn pam_end(pamh: *const (), pam_status: i32) -> i32;

	pub fn pam_get_user(pamh: *const (), user: *mut *const c_char, prompt: *const c_char) -> i32;

	pub fn pam_get_item(pamh: *const (), item_type: i32, item: *mut *const ()) -> i32;
	pub fn pam_set_item(pamh: *const (), item_type: i32, item: *const ()) -> i32;

	pub fn pam_get_data(pamh: *const (), module_data_name: *const c_char, data: *mut *const ()) -> i32;
	pub fn pam_set_data(
		pamh: *const (),
		module_data_name: *const c_char,
		data: *const (),
		cleanup: extern "C" fn(pamh: *const (), data: *mut (), error_status: i32)
	) -> i32;

	pub fn pam_getenv(pamh: *const (), name: *const c_char) -> *const c_char;
	pub fn pam_getenvlist(pamh: *const ()) -> *const *const c_char;

	pub fn pam_get_authtok(pamh: *const (), item: i32, authtok: *mut *const c_char, prompt: *const c_char) -> i32;
	pub fn pam_get_authtok_noverify(pamh: *const (), item: i32, authtok: *mut *const c_char, prompt: *const c_char) -> i32;
	pub fn pam_get_authtok_verify(pamh: *const (), item: i32, authtok: *mut *const c_char, prompt: *const c_char) -> i32;

	pub fn pam_setcred(pamh: *const (), flags: i32) -> i32;

	/// Linux-PAM extension.
	pub fn pam_prompt(pamh: *const (), style: i32, response: *mut *const c_char, fmt: *const c_char, ...) -> i32;
}
