fn main() {
	println!("cargo::rustc-link-arg=-nostdlib");
	println!("cargo::rustc-link-arg=-ffreestanding");
	println!("cargo::rustc-link-arg=-fno-builtin-memset");
}
