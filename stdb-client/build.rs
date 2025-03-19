use std::process::Command;

fn main() {
	// Should really do some sort of cargo::rerun-if-changed before running this.
	let res = Command::new("spacetime")
		.arg("generate")
		.arg("--lang")
		.arg("rust")
		.arg("-o")
		.arg("src/bindings")
		.arg("-p")
		.arg("../stdb-module")
		.status().unwrap();

	assert!(res.success());
}