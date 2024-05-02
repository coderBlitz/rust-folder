//! The freestanding Rust binary!
//!
//! This is my experiment for working with a libc-free rust binary, which
//!  currently takes the form of a static binary.

#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;

mod barebone;
mod mem;
mod utils;

use utils::*;
use core::{
	fmt::Write,
	write,
};

#[global_allocator]
static ALLOCATOR: mem::SimpleAlloc = mem::SimpleAlloc::new();

fn main() {
	let mut out = Stdout::new();

	for (i,a) in args().iter().enumerate() {
		if let Ok(s) = a {
			_ = write!(out, "Argv[{i}] = {s}\n");
		}
	}

	let params: Vec<&str> = args().iter().filter_map(|r| r.ok()).collect();
	_ = write!(out, "{params:?}\n")
}