//! fanotify - File Access (FA) notify
//!
//! A personal project to create Rust bindings for the fanotify API, since the
//!  nix crate lacks them, and libc isn't complete.
//!
//! Information derived from the following man pages:
//! * fanotify
//! * fanotify_init
//! * fanotify_mark

pub mod sys;
pub mod flags;

use std::convert::TryFrom;
use std::{ffi, fs, mem};
use std::io::{self, Read};
use std::ops::Range;
use std::os::fd::{self, AsFd, AsRawFd, FromRawFd, IntoRawFd};
use std::path::Path;
use flags::*;

type Result<T> = std::result::Result<T, io::Error>;

/// Represents a single event returned through fanotify.
///
/// TODO: Figure out how to fit all event types in this, or expand with an enum
///  or similar.
/// TODO: Needs to handle directory events (and those with files associated).
const EVT_META_SIZE: usize = mem::size_of::<sys::event_metadata>();
#[derive(Debug)]
pub struct Event {
	pub mask: EventFlags,
	pub file: EventFile,
	pub pid: u32
}

/// Should represent the various "file" references that fanotify returns.
#[derive(Debug)]
pub enum EventFile {
	Fd(fs::File),
	Fh(FileHandle) // Unable to be used for permission responses (PRE_CONTENT or CONTENT)
}

/// Represents a file handle returned by fanotify.
#[derive(Debug)]
pub struct FileHandle {
	fh: sys::file_handle,
	handle: Vec<u8>
}

/// Should represent the various extra info that can be returned.
pub enum Info {
	Fid(sys::fsid_t, Vec<u8>),
	Dfid(sys::fsid_t, Vec<u8>),
	DfidName(sys::fsid_t, Vec<u8>, ffi::OsString),
	PidFd(fd::RawFd),
	Error(u32, u32)
}
pub enum InfoType {
	Fid,
	Dfid,
	DfidName,
	PidFd,
	Error,
}
impl TryFrom<i32> for InfoType {
	type Error = ();

	fn try_from(n: i32) -> std::result::Result<Self, ()> {
		match n {
			sys::FAN_EVENT_INFO_TYPE_FID => Ok(Self::Fid),
			sys::FAN_EVENT_INFO_TYPE_DFID_NAME => Ok(Self::DfidName),
			sys::FAN_EVENT_INFO_TYPE_DFID => Ok(Self::Dfid),
			sys::FAN_EVENT_INFO_TYPE_PIDFD => Ok(Self::PidFd),
			sys::FAN_EVENT_INFO_TYPE_ERROR => Ok(Self::Error),
			_ =>Err(())
		}
	}
}

/// Fanotify instance
// `valid_buf` only exists because streaming iterator not possible. Struct
//  cannot create a lifetime for purposes of slices/borrows.
#[derive(Debug)]
pub struct Fanotify {
	/// Hold the fd returned by fanotify. Converted to OwnedFd for Drop trait.
	fan_fd: fs::File,
	/// Buffer used when reading from `fan_fd`
	evt_buffer: Box<[u8; 4096]>,
	/// Valid buffer range.
	valid_buf: Range<usize>,
}


// TODO: Create has_pending_events() using poll() (needs lib/crate/implement).
// TODO: Add allow()/deny() or similar for responses to PERM events.
impl Fanotify {
	/// Creates an fanotify instance with the given flags.
	///
	/// Passes the given flag parameters directly to `fanotify_init()`, and
	///  if successful, returns an `Fanotify` instance for further
	///  interactions.
	pub fn init(flags: &InitFlags, event_fd_flags: &EventFdFlags) -> Result<Self> {
		let fid = unsafe {
			sys::fanotify_init(flags.to_bits(), event_fd_flags.to_bits())
		};
		let err = io::Error::last_os_error();

		if fid == -1 {
			return Err(err);
		}

		Ok(Self {
			fan_fd: unsafe { fs::File::from_raw_fd(fid) },
			evt_buffer: Box::new([0; 4096]),
			valid_buf: Range { start: 0, end: 0 },
		})
	}

	/// Mark a path for which notification events are desired.
	///
	/// Passes the given flag parameters directly to `fanotify_mark()`.
	pub fn add_mark<P: AsRef<Path>>(&self, path: P, mtype: &MarkType, flags: &MarkFlags, mask: &EventFlags) -> Result<()> {
		fn inner(slf: &Fanotify, path: &Path, mtype: &MarkType, flags: &MarkFlags, mask: &EventFlags) -> Result<()> {
			if let Some(p) = path.to_str() {
				let c_path = ffi::CString::new(p).expect("Path to str will error if null byte.");

				// All bits except first three (FAN_MARK_{ADD,REMOVE,FLUSH})
				let add_flags = (flags.to_bits() & 0x7FFFFE6C) | sys::FAN_MARK_ADD | mtype.to_bits();

				// Call mark
				let res = unsafe {
					sys::fanotify_mark(slf.fan_fd.as_raw_fd(), add_flags, mask.to_bits() as u64, 0, c_path.as_ptr())
				};

				// If mark failed, return error
				let err = io::Error::last_os_error();
				if res == -1 {
					return Err(err);
				}

				return Ok(());
			}

			// Return this error to be consistent with error types.
			Err(io::Error::new(io::ErrorKind::InvalidInput, "Path contains invalid character(s)."))
		}

		inner(self, path.as_ref(), mtype, flags, mask)
	}

	/// Return the next [Event], or None.
	///
	/// Since the behavior is that of a streaming iterator, which isn't
	///  possible, the return permits use in a `while let` loop.
	pub fn events(&mut self) -> Option<Event> {
		// If slice too small (or empty), read new buffer.
		if self.valid_buf.len() < EVT_META_SIZE {
			debug_assert_eq!(self.valid_buf.len(), 0);

			// Read contents into buffer and update length.
			if let Ok(n) = self.fan_fd.read(&mut self.evt_buffer[..]) {
				self.valid_buf = 0..n;
			} else {
				return None
			}
		}

		// Get event metadata from buffer
		// SAFETY: `valid_buf` maintains Range invariant, and range end is never greater than `evt_buffer` length.
		let evt_start = unsafe {
			self.evt_buffer.as_ptr().add(self.valid_buf.start) as *const sys::event_metadata
		};
		// SAFETY: Initialized above. Unaligned read required since packed array.
		let evt = & unsafe { evt_start.read_unaligned() };

		// If event (somehow) extends beyond buffer length, empty range and return.
		if (evt.event_len as usize) > self.valid_buf.len() {
			self.valid_buf.start = self.valid_buf.end;
			return None
		}

		// If metadata version mismatch, panic
		debug_assert_eq!(evt.vers, sys::FANOTIFY_METADATA_VERSION as u8);

		// Slice for ease of parsing supplementary info.
		let full_evt = &self.evt_buffer[self.valid_buf.start .. (self.valid_buf.start + evt.event_len as usize)];

		// If there is metadata
		if evt.event_len as usize > EVT_META_SIZE {
			eprintln!("Long event len: {}", evt.event_len);

			// Loop over additional info
			let mut info_remain = &full_evt[EVT_META_SIZE as usize..]; // Start with full event to guarantee first loop
			eprintln!("Additional info length: {}", info_remain.len());

			// TODO: Check what maximum extra info structures is and possibly remove loop.
			//  Since there should be info for distinct things (file, parent dir, etc.),
			//  a loop might not make as much sense as just parsing N many times.
			while !info_remain.is_empty() {
				// Get common header
				let info_hdr = unsafe {
					&(info_remain.as_ptr() as *const sys::event_info_header).read_unaligned()
				};
				eprintln!("Info type {}, with len {}", info_hdr.info_type, info_hdr.len);

				// Get full info struct based on header
				// TODO: Change to normal int comparison so enum with values can be used
				// TODO: Possibly move this into separate function, and have that return enum with values.
				if let Ok(info_type) = InfoType::try_from(info_hdr.info_type as i32) { match info_type {
					InfoType::Fid | InfoType::Dfid => {
						eprintln!("\tINFO_(D)FID:");
						let info = unsafe {
							&*(info_remain.as_ptr() as *const sys::event_info_fid)
						};
						// Get handle bytes
						let _handle: &[u8] = unsafe {
							std::slice::from_raw_parts(info.file_handle.handle.as_ptr(), info.file_handle.handle_bytes as usize)
						};

						// Dump info for dev/debug
						eprintln!("\t\tfsid({:?})\n\t\tfile_handle({:?})", info.fsid, info.file_handle);
					},
					InfoType::DfidName => {
						eprintln!("\tINFO_(D)FID_NAME");
						let info = unsafe {
							&*(info_remain.as_ptr() as *const sys::event_info_fid)
						};

						// Get handle bytes
						let _handle: &[u8] = unsafe {
							std::slice::from_raw_parts(info.file_handle.handle.as_ptr(), info.file_handle.handle_bytes as usize)
						};

						// Filename guaranteed null-terminated by fanotify API
						let name_ptr = unsafe {
							(&info.file_handle.handle as *const _ as *const i8).offset(info.file_handle.handle_bytes as isize)
						};
						let fname = unsafe {
							ffi::CStr::from_ptr(name_ptr)
						};

						eprintln!("\t\tfsid({:X?})\n\t\tfile_handle({:?})\n\t\tname({:X?})", info.fsid, info.file_handle, fname);
					},
					_ => {}
				}} else {
					eprintln!("\tUnrecognized info type.");
				}

				// Move info slice forward by current length
				info_remain = &info_remain[info_hdr.len as usize..];
			}
		}

		// Event valid by this point. Move slice start to end of this event.
		self.valid_buf.start += evt.event_len as usize;

		// Final check of FD for proper event type
		if evt.fd == sys::FAN_NOFD || evt.fd == sys::FAN_NOPIDFD || evt.fd == sys::FAN_EPIDFD {
			eprintln!("File handle vs descriptor not properly handled.");
			return None;
		}

		/* Return the event
		File descriptor guaranteed valid by fanotify API.
		*/
		Some(Event {
			mask: EventFlags::from_bits(evt.mask as i32),
			file: EventFile::Fd(unsafe {
				fs::File::from_raw_fd(evt.fd as i32)
			}),
			pid: evt.pid
		})
	}

	/// Clear all marks for mounts.
	pub fn clear_mnt_marks(&mut self) -> Result<()> {
		// Set flags and create valid pathname (flushing still requires pathname be valid).
		let flags = sys::FAN_MARK_FLUSH | sys::FAN_MARK_MOUNT;
		let root = ffi::CString::new("/").expect("String literal should not contain null bytes.");

		// Make call to flush
		let res = unsafe {
			sys::fanotify_mark(self.fan_fd.as_raw_fd(), flags, 0, 0, root.as_ptr())
		};
		let err = io::Error::last_os_error();

		if res == -1 {
			return Err(err);
		}

		Ok(())
	}
	/// Clear all marks for filesystems.
	pub fn clear_fs_marks(&mut self) -> Result<()> {
		// Set flags and create valid pathname (flushing still requires pathname be valid).
		let flags = sys::FAN_MARK_FLUSH | sys::FAN_MARK_FILESYSTEM;
		let root = ffi::CString::new("/").expect("String literal should not contain null bytes.");

		// Make call to flush
		let res = unsafe {
			sys::fanotify_mark(self.fan_fd.as_raw_fd(), flags, 0, 0, root.as_ptr())
		};
		let err = io::Error::last_os_error();

		if res == -1 {
			return Err(err);
		}

		Ok(())
	}
	/// Clear all marks on specific files and directories.
	pub fn clear_file_marks(&mut self) -> Result<()> {
		// Set flags and create valid pathname (flushing still requires pathname be valid).
		let flags = sys::FAN_MARK_FLUSH;
		let root = ffi::CString::new("/").expect("String literal should not contain null bytes.");

		// Make call to flush
		let res = unsafe {
			sys::fanotify_mark(self.fan_fd.as_raw_fd(), flags, 0, 0, root.as_ptr())
		};
		let err = io::Error::last_os_error();

		if res == -1 {
			return Err(err);
		}

		Ok(())
	}
	/// Clear all marks (mounts, filesystem, and specific files/dirs).
	///
	/// Equivalent to calling each of [clear_mnt_marks()], [clear_fs_marks()],
	///  and [clear_file_marks()].
	pub fn clear_all_marks(&mut self) -> Result<()> {
		if let Err(e) = self.clear_mnt_marks() {
			return Err(e);
		}
		if let Err(e) = self.clear_fs_marks() {
			return Err(e);
		}
		if let Err(e) = self.clear_file_marks() {
			return Err(e);
		}

		Ok(())
	}
}
impl AsFd for Fanotify {
	fn as_fd(&self) -> fd::BorrowedFd {
		self.fan_fd.as_fd()
	}
}
impl AsRawFd for Fanotify {
	fn as_raw_fd(&self) -> fd::RawFd {
		self.fan_fd.as_raw_fd()
	}
}
impl IntoRawFd for Fanotify {
	fn into_raw_fd(self) -> fd::RawFd {
		self.fan_fd.into_raw_fd()
	}
}
