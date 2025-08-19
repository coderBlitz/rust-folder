use fit_parser::*;
use std::path::Path;

fn main() {
	let path = Path::new("test_data/2025-08-01-13-14-12.fit");

	let mut f = open(path).unwrap();
	f.parse_record();
}
