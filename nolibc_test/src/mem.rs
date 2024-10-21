//! Module for memory allocator and related functions.

use core::{
	alloc::{GlobalAlloc,Layout},
	sync::atomic::{AtomicUsize, Ordering},
};
use syscalls::{
	syscall,
	Sysno,
};
const MAP_ANONYMOUS: usize = 0x20;
const MAP_PRIVATE: usize = 0x2;
const PROT_READ: usize = 0x1;
const PROT_WRITE: usize = 0x2;


pub struct SimpleAlloc(AtomicUsize);
impl SimpleAlloc {
	pub const fn new() -> Self {
		SimpleAlloc(AtomicUsize::new(0))
	}
}
unsafe impl GlobalAlloc for SimpleAlloc {
	unsafe fn alloc(&self, lay: Layout) -> *mut u8 {
		let p = unsafe { syscall!( Sysno::mmap,
			0,
			lay.size(),
			PROT_READ | PROT_WRITE,
			MAP_ANONYMOUS | MAP_PRIVATE,
			0,
			0
		)};

		let p = p.map_or(core::ptr::null_mut(), |v| v as *mut u8);

		if !p.is_null() {
			self.0.fetch_add(1, Ordering::Relaxed);
		}

		p as _
	}
	unsafe fn dealloc(&self, ptr: *mut u8, lay: Layout) {
		_ = unsafe { syscall!(Sysno::munmap, ptr as usize, lay.size()) };
		self.0.fetch_sub(0, Ordering::Relaxed);
	}
}
