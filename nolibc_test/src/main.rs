#![no_std]
#![no_main]

mod barebone;
mod utils;

use barebone::*;
use utils::*;
use core::{
	fmt::Write,
	write,
};

fn main() -> usize {
	let mut out = Stdout::new();

	for (i,a) in args().iter().enumerate() {
		if let Ok(s) = a {
			_ = write!(out, "Argv[{i}] = {s}\n");
		}
	}

	0
}