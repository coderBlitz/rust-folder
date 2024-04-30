#![no_std]
#![no_main]

use core::{
	arch::global_asm,
	fmt::Write,
	write,
	unreachable,
};
use rustix;
use syscalls::{
	syscall,
	x86_64::Sysno,
};

#[panic_handler]
fn panic(pi: &core::panic::PanicInfo) -> ! {
	let mut out = Stdout::new();

	if let Some(loc) = pi.location(){
		_ = write!(out, "Panic in {} at line {} column {}.\n", loc.file(), loc.line(), loc.column());
	}

	// Pretty sure in core the payload is always empty.
	if let Some(s) = pi.payload().downcast_ref::<&str>(){
		_ = write!(out, "{s}\n");
	}

	exit(255)
}

fn exit(status: usize) -> ! {
	_ = unsafe { syscall!(Sysno::exit, status as u8) };
	unreachable!();
}

struct Stdout<'a>(rustix::fd::BorrowedFd<'a>);
impl<'a> Stdout<'a> {
	fn new() -> Self {
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

// To get the compiler to stop complaining.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

fn strlen(s: *const u8) -> usize {
	let mut w = s;
	unsafe {
		while *w != 0 {
			w = w.add(1);
		}
	}

	(w as usize) - (s as usize)
}

#[no_mangle]
pub extern "C" fn main(argc: i32, argv: *const *const u8, _envp: *const *const u8) -> ! {
	let mut out = Stdout::new();
	_ = write!(out, "Argc is {argc:x} argv is {:x}\n", (argv as u64));

	for i in 0..argc as usize {
		let ss = unsafe { core::slice::from_raw_parts(*argv.add(i), strlen(*argv.add(i))) };
		let s = unsafe { core::str::from_utf8_unchecked(ss) };
		_ = write!(out, "Argv[{i}] = {s}\n");
	}

	exit(0)
}

// Create the entry point with argc, argv, envp
global_asm!(
	//".text",
	".globl _start",
	"_start:", // _start is the entry point known to the linker
	"xor ebp, ebp", // effectively RBP := 0, mark the end of stack frames
	"mov edi, [rsp]", // get argc from the stack (implicitly zero-extended to 64-bit)
	"lea rsi, [rsp+8]", // take the address of argv from the stack
	"lea rdx, [rsp+rdi*8+16]", // take the address of envp from the stack
	"xor eax, eax", // per ABI and compatibility with icc
	"call main" // %edi, %rsi, %rdx are the three args (of which first two are C standard) to main
);