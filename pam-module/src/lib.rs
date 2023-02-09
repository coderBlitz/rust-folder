/*
Using any of the Rust std crate requires fairly recent GLIBC versions. As of
 writing, the latest glibc version is ~2.36, and the Rust std requires has
 dependencies for 2.34+.
*/

use std::{ffi, fs, ptr};
use libc::{openlog, syslog, closelog};
use libc::{LOG_CONS, LOG_PID, LOG_NDELAY, LOG_LOCAL1, LOG_DEBUG};

#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, PartialEq)]
enum PamResultCode {
	PAM_SUCCESS = 0,
	PAM_OPEN_ERR = 1,
	PAM_SYMBOL_ERR = 2,
	PAM_SERVICE_ERR = 3,
	PAM_SYSTEM_ERR = 4,
	PAM_BUF_ERR = 5,
	PAM_PERM_DENIED = 6,
	PAM_AUTH_ERR = 7,
	PAM_CRED_INSUFFICIENT = 8,
	PAM_AUTHINFO_UNAVAIL = 9,
	PAM_USER_UNKNOWN = 10,
	PAM_MAXTRIES = 11,
	PAM_NEW_AUTHTOK_REQD = 12,
	PAM_ACCT_EXPIRED = 13,
	PAM_SESSION_ERR = 14,
	PAM_CRED_UNAVAIL = 15,
	PAM_CRED_EXPIRED = 16,
	PAM_CRED_ERR = 17,
	PAM_NO_MODULE_DATA = 18,
	PAM_CONV_ERR = 19,
	PAM_AUTHTOK_ERR = 20,
	PAM_AUTHTOK_RECOVERY_ERR = 21,
	PAM_AUTHTOK_LOCK_BUSY = 22,
	PAM_AUTHTOK_DISABLE_AGING = 23,
	PAM_TRY_AGAIN = 24,
	PAM_IGNORE = 25,
	PAM_ABORT = 26,
	PAM_AUTHTOK_EXPIRED = 27,
	PAM_MODULE_UNKNOWN = 28,
	PAM_BAD_ITEM = 29,
	PAM_CONV_AGAIN = 30,
	PAM_INCOMPLETE = 31,
}

#[link(name = "pam")]
extern "C" {
	fn pam_get_user(pamh: *const (), user: *mut *const ffi::c_char, prompt: *const ffi::c_char) -> ffi::c_int;
}

#[no_mangle]
pub extern "C" fn pam_sm_authenticate(
		_pamh: *const (),
		_flags: ffi::c_int,
		_argc: ffi::c_int,
		_argv: *const *const ffi::c_char
	) -> ffi::c_int {

	// TODO: Log creds or whatever to file
	//let f = fs::OpenOptions::new().create(true).append(true).open("/tmp/log.txt");

	/* Get the username
	*/
	// TODO: Use pam_get_item instead
	let mut us = ptr::null();
	let prompt = ffi::CString::new("MyPAM").unwrap();
	let res = unsafe { pam_get_user(_pamh, &mut us, prompt.as_ptr()) };

	/* Start syslog
	*/
	let service_name: ffi::CString = ffi::CString::new("MyPAM").unwrap();
	unsafe {
		openlog(service_name.as_ptr(), LOG_CONS | LOG_PID | LOG_NDELAY, LOG_LOCAL1);
	}

	/* Create some syslog messages
	*/
	let msg = ffi::CString::new("Callback was successful!").unwrap();
	let ptr_msg = ffi::CString::new(format!("user ptr = {us:p}")).unwrap();
	
	let res_text = match res {
		0 => "User call success!",
		_ => "User call fail!",
	};
	let res_text = ffi::CString::new(res_text).unwrap();

	/* Send syslog messages
	*/
	unsafe {
		syslog(LOG_DEBUG, msg.as_ptr());
		syslog(LOG_DEBUG, ptr_msg.as_ptr());
		syslog(LOG_DEBUG, res_text.as_ptr());
	}

	/* If user pointer was successfully grabbed, extract the username from it,
	then print syslog message with username.
	*/
	let user;
	if let Some(nn) = ptr::NonNull::new(us as *mut _) {
		user = unsafe {
			ffi::CStr::from_ptr(nn.as_ptr()).to_str().unwrap_or_default()
		};
	} else {
		user = "";
	}

	let msg = ffi::CString::new(format!("username = {user}")).unwrap();
	unsafe {
		syslog(LOG_DEBUG, msg.as_ptr());
	}

	unsafe {
		closelog();
	}

	PamResultCode::PAM_SUCCESS as i32
}
