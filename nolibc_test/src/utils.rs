use rustix;
pub use crate::barebone::{exit, strlen};


pub struct Stdout<'a>(rustix::fd::BorrowedFd<'a>);
impl<'a> Stdout<'a> {
	pub fn new() -> Self {
		Self(unsafe { rustix::stdio::stdout() })
	}
}
impl<'a> core::fmt::Write for Stdout<'a> {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		match rustix::io::write(&self.0, s.as_bytes()) {
			Ok(_) => Ok(()),
			Err(_) => Err(core::fmt::Error),
		}
	}

	// Default/provided implementation calls memset somewhere, which causes compilation error.
	fn write_char(&mut self, c: char) -> core::fmt::Result {
		match rustix::io::write(&self.0, &(c as u32).to_be_bytes()) {
			Ok(_) => Ok(()),
			Err(_) => Err(core::fmt::Error),
		}
	}
}