use pam::*;
use std::ffi::CStr;

// Auth
#[authenticate]
fn auth(_handle: PamHandle, _flags: i32, _args: Vec<&CStr>) -> PamResult {
	PamResult::Ignore
}
#[setcred]
fn sc(_handle: PamHandle, _flags: i32, _args: Vec<&CStr>) -> PamResult {
	PamResult::Ignore
}

// Session
#[open_session]
fn open(_handle: PamHandle, _flags: i32, _args: Vec<&CStr>) -> PamResult {
	PamResult::Ignore
}
#[close_session]
fn close(_handle: PamHandle, _flags: i32, _args: Vec<&CStr>) -> PamResult {
	PamResult::Ignore
}

// Account
#[acct_mgmt]
fn acct(_handle: PamHandle, _flags: i32, _args: Vec<&CStr>) -> PamResult {
	PamResult::Ignore
}

// Password
#[chauthtok]
fn passwd(_handle: PamHandle, _flags: i32, _args: Vec<&CStr>) -> PamResult {
	PamResult::Ignore
}
