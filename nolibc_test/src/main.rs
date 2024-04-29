#![no_std]
#![no_main]

use core::unreachable;

use rustix;
use syscalls::{
	syscall,
	x86_64::Sysno,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
	exit()
}

fn exit() -> ! {
	_ = unsafe { syscall!(Sysno::exit, 255) };
	unreachable!();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
	let s = "Hello world!\n";
	let out = unsafe { rustix::stdio::stdout() };
	_ = rustix::io::write(&out, s.as_bytes());


	exit()
}