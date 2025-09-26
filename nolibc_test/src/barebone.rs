#![allow(dead_code)]

use crate::main;
use crate::utils::*;
use core::{
	any::{Any, TypeId},
	arch::global_asm,
	fmt::Write,
	unreachable, write,
};
use syscalls::{syscall, x86_64::Sysno};

// Global for easy static state that gets initialized upon entry point.
static mut ARGS: Args = Args::new(0, core::ptr::null());

// Create the entry point with argc, argv, envp
global_asm!(
	//".text",
	".globl _start",
	"_start:",                 // _start is the entry point known to the linker
	"xor ebp, ebp",            // effectively RBP := 0, mark the end of stack frames
	"mov edi, [rsp]",          // get argc from the stack (implicitly zero-extended to 64-bit)
	"lea rsi, [rsp+8]",        // take the address of argv from the stack
	"lea rdx, [rsp+rdi*8+16]", // take the address of envp from the stack
	"xor eax, eax",            // per ABI and compatibility with icc
	"call _main" // %edi, %rsi, %rdx are the three args (of which first two are C standard) to main
);

#[panic_handler]
fn panic(pi: &core::panic::PanicInfo) -> ! {
	let mut out = Stdout::new();

	if let Some(loc) = pi.location() {
		_ = write!(
			out,
			"Panic in {} at line {} column {}.\n",
			loc.file(),
			loc.line(),
			loc.column()
		);
	}

	exit(255)
}

// To get the compiler to stop complaining.
#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

/// Exit syscall.
pub fn exit(status: usize) -> ! {
	_ = unsafe { syscall!(Sysno::exit, status as u8) };
	unreachable!();
}

#[unsafe(no_mangle)]
pub extern "C" fn strlen(s: *const u8) -> usize {
	let mut w = s;
	unsafe {
		while *w != 0 {
			w = w.add(1);
		}
	}

	(w as usize) - (s as usize)
}

#[unsafe(no_mangle)]
pub extern "C" fn memset(s: *mut u8, c: u8, len: usize) -> *mut u8 {
	let mut ls = s as *mut usize;
	let lc = (c as usize) << 56
		| (c as usize) << 48
		| (c as usize) << 40
		| (c as usize) << 32
		| (c as usize) << 24
		| (c as usize) << 16
		| (c as usize) << 8
		| (c as usize) << 0;
	let mut i = len;
	const STEP: usize = core::mem::size_of::<usize>();

	// Copy in largest steps possible for as long as possible.
	while i >= STEP {
		unsafe {
			*ls = lc;

			// Increment pointer
			ls = ls.add(1);
		}

		i -= STEP;
	}

	// Shrink pointers back to bytes, then finish copy
	let mut ls = ls as *mut u8;

	while i > 0 {
		unsafe {
			*ls = c;

			// Increment pointers
			ls = ls.add(1);
		}

		i -= 1;
	}

	s
}

#[unsafe(no_mangle)]
pub extern "C" fn memcpy(d: *mut u8, s: *const u8, len: usize) -> *mut u8 {
	let mut i = len;
	let mut ls = s as *mut usize;
	let mut ld = d as *mut usize;
	const STEP: usize = core::mem::size_of::<usize>();

	// Copy in largest steps possible for as long as possible.
	while i >= STEP {
		unsafe {
			*ld = *ls;

			// Increment pointers
			ls = ls.add(1);
			ld = ld.add(1);
		}

		i -= STEP;
	}

	// Shrink pointers back to bytes, then finish copy
	let mut ls = ls as *mut u8;
	let mut ld = ld as *mut u8;

	while i > 0 {
		unsafe {
			*ld = *ls;

			// Increment pointers
			ls = ls.add(1);
			ld = ld.add(1);
		}

		i -= 1;
	}

	d
}

/// Maintain a copy of the argc and argv passed into this program.
#[derive(Clone, Copy)]
pub struct Args {
	argc: usize,
	argv: *const *const u8,
}
impl Args {
	const fn new(argc: i32, argv: *const *const u8) -> Self {
		Self {
			argc: argc as usize,
			argv,
		}
	}

	pub const fn iter(&self) -> ArgsIter<'_> {
		ArgsIter(0, self)
	}

	pub const fn len(&self) -> usize {
		self.argc
	}
}

/// Iterator over the arguments in [Args].
pub struct ArgsIter<'a>(usize, &'a Args);
impl<'a> core::iter::Iterator for ArgsIter<'a> {
	type Item = Result<&'static str, ()>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.0 < self.1.argc {
			// Make slice for arg at current index, then increment index for next time.
			let ss = unsafe {
				core::slice::from_raw_parts(
					*self.1.argv.add(self.0),
					strlen(*self.1.argv.add(self.0)),
				)
			};
			self.0 += 1;

			match core::str::from_utf8(ss) {
				Ok(s) => Some(Ok(s)),
				_ => Some(Err(())),
			}
		} else {
			None
		}
	}
}

/// Return a copy of the executable arguments array struct.
pub fn args() -> Args {
	unsafe { ARGS }
}

/// Entry point for binary.
#[unsafe(no_mangle)]
extern "C" fn _main(argc: i32, argv: *const *const u8, _envp: *const *const u8) -> ! {
	// Set the global arguments for use by whoever.
	unsafe {
		ARGS = Args::new(argc, argv);
	}

	// Call main.
	// Permit any return type but provide a mapped exit code for integer/usize
	//  and rustix result.
	let res: &dyn Any = &main();

	// Map return type to exit code.
	let ty = res.type_id();
	let ret = if ty == TypeId::of::<usize>() {
		*res.downcast_ref::<usize>().unwrap()
	} else if ty == TypeId::of::<rustix::io::Result<()>>() {
		let r = res.downcast_ref::<rustix::io::Result<()>>().unwrap();
		match r {
			Ok(_) => 0,
			Err(e) => e.raw_os_error() as usize,
		}
	} else {
		0
	};

	// Exit with given code.
	exit(ret)
}
