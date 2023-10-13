fn main(){
	println!("cargo:rustc-link-lib=pam");
	println!("cargo:rustc-link-arg=-zdefs");
}
