//! The freestanding Rust binary!
//!
//! This is my experiment for working with a libc-free rust binary, which
//!  currently takes the form of a static binary.

#![no_std]
#![no_main]

mod barebone;
mod utils;

use utils::*;
use core::{
	fmt::Write,
	write,
};

fn main() {
	let mut out = Stdout::new();

	for (i,a) in args().iter().enumerate() {
		if let Ok(s) = a {
			_ = write!(out, "Argv[{i}] = {s}\n");
		}
	}
}