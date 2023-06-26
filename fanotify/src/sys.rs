#![allow(dead_code, non_camel_case_types)]

use std::ffi;

/* Standard open flags that are usable with fanotify */
pub const O_RDONLY: i32 = 0o00000000;
pub const O_WRONLY: i32 = 0o00000001;
pub const O_RDWR: i32 = 0o00000002;
pub const O_LARGEFILE: i32 = 0o00100000;
pub const O_CLOEXEC: i32 = 0o02000000;
pub const O_APPEND: i32 = 0o00002000;
pub const O_DSYNC: i32 = 0o00010000;
pub const O_NOATIME: i32 = 0o01000000;
pub const O_NONBLOCK: i32 = 0o00004000;
pub const O_SYNC: i32 = 0o04010000; // (__O_SYNC=04000000 | O_DSYNC)

/* Events that user-space can register for */
/// File was accessed
pub const FAN_ACCESS: i32 = 0x00000001;
/// File was modified
pub const FAN_MODIFY: i32 = 0x00000002;
/// Metadata changed
pub const FAN_ATTRIB: i32 = 0x00000004;
/// Writtable file closed
pub const FAN_CLOSE_WRITE: i32 = 0x00000008;
/// Unwrittable file closed
pub const FAN_CLOSE_NOWRITE: i32 = 0x00000010;
/// File was opened
pub const FAN_OPEN: i32 = 0x00000020;
/// File was moved from X
pub const FAN_MOVED_FROM: i32 = 0x00000040;
/// File was moved to Y
pub const FAN_MOVED_TO: i32 = 0x00000080;
/// Subfile was created
pub const FAN_CREATE: i32 = 0x00000100;
/// Subfile was deleted
pub const FAN_DELETE: i32 = 0x00000200;
/// Self was deleted
pub const FAN_DELETE_SELF: i32 = 0x00000400;
/// Self was moved
pub const FAN_MOVE_SELF: i32 = 0x00000800;
/// File was opened for exec
pub const FAN_OPEN_EXEC: i32 = 0x00001000;

/// Event queued overflowed
pub const FAN_Q_OVERFLOW: i32 = 0x00004000;
/// Filesystem error
pub const FAN_FS_ERROR: i32 = 0x00008000;

/// File open in perm check
pub const FAN_OPEN_PERM: i32 = 0x00010000;
/// File accessed in perm check
pub const FAN_ACCESS_PERM: i32 = 0x00020000;
/// File open/exec in perm check
pub const FAN_OPEN_EXEC_PERM: i32 = 0x00040000;

/// Interested in child events
pub const FAN_EVENT_ON_CHILD: i32 = 0x08000000;

/// File was renamed
pub const FAN_RENAME: i32 = 0x10000000;

/// Event occurred against dir
pub const FAN_ONDIR: i32 = 0x40000000;

/// Helper constant for close
pub const FAN_CLOSE: i32 = FAN_CLOSE_WRITE | FAN_CLOSE_NOWRITE;
/// Helper constant for moves
pub const FAN_MOVE: i32 = FAN_MOVED_FROM | FAN_MOVED_TO;

/* flags used for fanotify_init() */
pub const FAN_CLOEXEC: i32 = 0x00000001;
pub const FAN_NONBLOCK: i32 = 0x00000002;

/* These are NOT bitwise flags.  Both bits are used together.  */
pub const FAN_CLASS_NOTIF: i32 = 0x00000000;
pub const FAN_CLASS_CONTENT: i32 = 0x00000004;
pub const FAN_CLASS_PRE_CONTENT: i32 = 0x00000008;

pub const FAN_UNLIMITED_QUEUE: i32 = 0x00000010;
pub const FAN_UNLIMITED_MARKS: i32 = 0x00000020;
pub const FAN_ENABLE_AUDIT: i32 = 0x00000040;

/* Flags to determine fanotify event format */
/// Report pidfd for event->pid
pub const FAN_REPORT_PIDFD: i32 = 0x00000080;
/// event->pid is thread id
pub const FAN_REPORT_TID: i32 = 0x00000100;
/// Report unique file id
pub const FAN_REPORT_FID: i32 = 0x00000200;
/// Report unique directory id
pub const FAN_REPORT_DIR_FID: i32 = 0x00000400;
/// Report events with name
pub const FAN_REPORT_NAME: i32 = 0x00000800;
/// Report dirent target id
pub const FAN_REPORT_TARGET_FID: i32 = 0x00001000;

/* Convenience macro - FAN_REPORT_NAME requires FAN_REPORT_DIR_FID */
pub const FAN_REPORT_DFID_NAME: i32 = FAN_REPORT_DIR_FID | FAN_REPORT_NAME;
/* Convenience macro - FAN_REPORT_TARGET_FID requires all other FID flags */
pub const FAN_REPORT_DFID_NAME_TARGET: i32 = FAN_REPORT_DFID_NAME | FAN_REPORT_FID | FAN_REPORT_TARGET_FID;


/* flags used for fanotify_modify_mark() */
pub const FAN_MARK_ADD: i32 = 0x00000001;
pub const FAN_MARK_REMOVE: i32 = 0x00000002;
pub const FAN_MARK_DONT_FOLLOW: i32 = 0x00000004;
pub const FAN_MARK_ONLYDIR: i32 = 0x00000008;

/* FAN_MARK_MOUNT is        0x00000010 */
pub const FAN_MARK_IGNORED_MASK: i32 = 0x00000020;
pub const FAN_MARK_IGNORED_SURV_MODIFY: i32 = 0x00000040;
pub const FAN_MARK_FLUSH: i32 = 0x00000080;

/* FAN_MARK_FILESYSTEM is   0x00000100 */
pub const FAN_MARK_EVICTABLE: i32 = 0x00000200;

/* This bit is mutually exclusive with FAN_MARK_IGNORED_MASK bit */
pub const FAN_MARK_IGNORE: i32 = 0x00000400;

/* These are NOT bitwise flags.  Both bits can be used togther.  */
pub const FAN_MARK_INODE: i32 = 0x00000000;
pub const FAN_MARK_MOUNT: i32 = 0x00000010;
pub const FAN_MARK_FILESYSTEM: i32 = 0x00000100;

pub const FAN_MARK_IGNORE_SURV: i32 = FAN_MARK_IGNORE | FAN_MARK_IGNORED_SURV_MODIFY;

pub const FANOTIFY_METADATA_VERSION: i32 = 3;

pub const FAN_EVENT_INFO_TYPE_FID: i32 = 1;
pub const FAN_EVENT_INFO_TYPE_DFID_NAME: i32 = 2;
pub const FAN_EVENT_INFO_TYPE_DFID: i32 = 3;
pub const FAN_EVENT_INFO_TYPE_PIDFD: i32 = 4;
pub const FAN_EVENT_INFO_TYPE_ERROR: i32 = 5;

/* Special info types for FAN_RENAME */
pub const FAN_EVENT_INFO_TYPE_OLD_DFID_NAME: i32 = 10;
/* Reserved for FAN_EVENT_INFO_TYPE_OLD_DFID    11 */
pub const FAN_EVENT_INFO_TYPE_NEW_DFID_NAME: i32 = 12;
/* Reserved for FAN_EVENT_INFO_TYPE_NEW_DFID    13 */

pub const FAN_RESPONSE_INFO_NONE: i32 = 0;
pub const FAN_RESPONSE_INFO_AUDIT_RULE: i32 = 1;

/* Legit userspace responses to a _PERM event */
pub const FAN_ALLOW: i32 = 0x01;
pub const FAN_DENY: i32 = 0x02;
/// Bitmask to create audit record for result
pub const FAN_AUDIT: i32 = 0x10;
/// Bitmask to indicate additional information
pub const FAN_INFO: i32 = 0x20;

/* No fd set in event */
pub const FAN_NOFD: i32 = -1;
pub const FAN_NOPIDFD: i32 = FAN_NOFD;
pub const FAN_EPIDFD: i32 = -2;


extern "C" {
	pub fn fanotify_init(flags: ffi::c_int, event_f_flags: ffi::c_int) -> ffi::c_int;
	pub fn fanotify_mark(
		fanotify_fd: ffi::c_int,
		flags: ffi::c_int,
		mask: u64,
		dirfd: ffi::c_int,
		pathname: *const ffi::c_char
	) -> ffi::c_int;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct event_metadata {
	pub event_len: u32,
	pub vers: u8,
	pub _reserved: u8,
	/// No optional headers exist as implemented, so likely unused
	pub _metadata_len: u16,
	pub mask: u64,
	// MUST close when finished using
	pub fd: i32,
	pub pid: u32
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct event_info_header {
	pub info_type: u8,
	pub _pad: u8,
	/// Size of info record, including event_info_header.
	pub len: u16
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct event_info_fid {
	pub hdr: event_info_header,
	// Kernel struct thing, same value as f_fsid when calling statfs.
	// Struct containing `int val[2]` for deprecated statfs(), unsigned long for statvfs()
	pub fsid: fsid_t,
	// variable length struct file_handle, opaque handle (as from name_to_handle_at)
	// May contain null-terminated string if FAN_EVENT_INFO_TYPE_DFID_NAME.
	//pub file_handle: *const u8 // zero-sized array
	pub file_handle: file_handle,
	//f_handle: [u8; file_handle.handle_bytes] // Kernel fanotify inline file handle bytes
	// If filename present, filename is null-terminated bytes following
}
#[derive(Clone, Copy, Debug)]
pub struct fsid_t {
	val: [u32; 2] // May need something to convert to 64 bit
}
#[derive(Clone, Copy, Debug)]
pub struct file_handle {
	pub handle_bytes: u32,
	_handle_type: i32
	//f_handle: *const u8 // zero-sized array (technically handle_bytes in length, so dynamic)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct event_info_pidfd {
	pub hdr: event_info_header,
	// No different from value as obtained by pidfd_open on event_metadata.pid
	// May be FAN_NOPIDFD
	// MUST be closed once event is dealth with
	pub pidfd: u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct event_info_error {
	/// `info_type` set to `FAN_EVENT_INFO_TYPE_ERROR`
	pub hdr: event_info_header,
	/// Type of error that occurred
	pub error: u32,
	/// Count of errors since last error was read
	pub error_count: u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct response {
	/// The `fd` as given in the `event_metadata` structure.
	pub fd: u32,
	/// Must be one of `FAN_ALLOW` or `FAN_DENY`. If `FAN_ENABLE_AUDIT`, then
	///  `FAN_AUDIT` can be set to log decision to audit subsystem.
	pub response: u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct response_info_header {
	pub typ: u8,
	pub pad: u8,
	pub len: u16
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct response_info_audit_rule {
	pub hdr: response_info_header,
	pub rule_number: u32,
	pub subj_trust: u32,
	pub obj_trust: u32
}
