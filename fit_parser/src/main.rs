use fit_parser::*;
use std::{fs::File, io::Read, path::Path};

fn main() {
	let path = Path::new("test_data/2025-08-01-13-14-12.fit");

	let Ok(mut fp) = File::open(path) else { return };

	let mut header_bytes = [0; 14];
	_ = fp.read(&mut header_bytes);

	println!("Header bytes: {header_bytes:x?}");
	let hdr = parse_hdr(&header_bytes).unwrap();
	println!("Hdr: {hdr:?}");
}
