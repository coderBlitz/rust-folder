//! fanotify - File Access (FA) notify
//!
//! A personal project to create Rust bindings for the fanotify API, since the
//!  nix crate lacks them, and an existing crate isn't complete.
//!
//! Information derived from the following man pages:
//! * fanotify
//! * fanotify_init
//! * fanotify_mark

pub mod sys;
pub mod flags;

use std::ffi;
use std::fs;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::Path;
use flags::*;

type Result<T> = std::result::Result<T, io::Error>;

#[derive(Debug)]
pub struct Event {
	pub mask: EventFlags,
	pub file: fs::File,
	pub pid: u32
}

/// Fanotify instance
// `event_buffer_len` only exists because streaming iterator not possible.
#[derive(Debug)]
pub struct Fanotify {
	/// Hold the fd returned by fanotify. Converted to OwnedFd for Drop trait.
	fan_fd: fs::File,
}

/// Iterator returned by [Fanotify::iter()] which iterates the supplied event
/// buffer.
///
/// Since this iterate the supplied buffer, this iterator will not continuously
///  supply events. If continuous events are desired, [Fanotify::iter()] must
///  be called repeatedly.
#[derive(Debug)]
pub struct EventIter<'a> {
	/// Buffer used when reading from `fan_fd`
	evt_buffer: Box<[u8; 4096]>,
	/// Slice of valid buffer remaining
	next_buf: &'a [u8]
}

// TODO: Create has_pending_events() using poll() (needs lib/crate/implement).
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
		})
	}

	/// Mark a path for which notification events are desired.
	///
	/// Passes the given flag parameters directly to `fanotify_mark()`.
	pub fn add_mark<P: AsRef<Path>>(&self, path: P, flags: &MarkFlags, mask: &EventFlags) -> Result<()> {
		if let Some(p) = path.as_ref().to_str() {
			let c_path = ffi::CString::new(p).expect("Path to str will error if null byte.");

			// All bits except first three (FAN_MARK_{ADD,REMOVE,FLUSH})
			let add_flags = (flags.to_bits() & 0x7FFFFFF8) | sys::FAN_MARK_ADD;

			// Call mark
			let res = unsafe {
				sys::fanotify_mark(self.fan_fd.as_raw_fd(), add_flags, mask.to_bits() as u64, 0, c_path.as_ptr())
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

	/// Returns an [EventIter] with an iterable buffer of some notify events.
	///
	/// Since streaming iterators aren't (cleanly) possible, the returned
	///  iterator only contains a limited number of notify events. If more
	///  events are desired, this function must be called again.
	pub fn iter(&mut self) -> EventIter {
		let mut evti = EventIter {
			evt_buffer: Box::new([0; 4096]),
			next_buf: &[]
		};

		/* Read contents into buffer and update length.
		Unsafe required since lifetime of evti differs from &self, but
		 returned struct owns the boxed array. Since the boxed array will
		 live as long as the slice, slice will always be valid if it uses
		 said array.
		*/
		if let Ok(n) = self.fan_fd.read(&mut evti.evt_buffer[..]) {
			evti.next_buf = unsafe {
				std::slice::from_raw_parts(evti.evt_buffer.as_ptr(), n)
			};
		}

		evti
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

impl<'a> Iterator for EventIter<'a> {
	type Item = Event;

	/// Iterates through events in the current buffer.
	fn next(&mut self) -> Option<Self::Item> {
		// If slice too small (or empty), end of iterator reached.
		if self.next_buf.len() < std::mem::size_of::<sys::event_metadata>() {
				return None
		}

		/* Get event metadata from buffer
		Pointer guaranteed to be valid, since next_buf always points to a valid
		 region of evt_buffer.
		*/
		let evt = unsafe {
			*(self.next_buf.as_ptr() as *const sys::event_metadata)
		};

		// If event (somehow) extends beyond buffer length, return.
		if (evt.event_len as usize) > self.next_buf.len() {
			return None
		}

		// Event valid by this point. Move slice start to end of this event.
		self.next_buf = &self.next_buf[evt.event_len as usize..];

		/* Return the event
		File descriptor guaranteed valid by fanotify API.
		*/
		Some(Event {
			mask: EventFlags::from_bits(evt.mask as i32),
			file: unsafe {
				fs::File::from_raw_fd(evt.fd as i32)
			},
			pid: evt.pid
		})
	}

	/// Provide upper bound based on fanotify event minimum size.
	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, Some(self.next_buf.len() / std::mem::size_of::<sys::event_metadata>()))
	}
}
