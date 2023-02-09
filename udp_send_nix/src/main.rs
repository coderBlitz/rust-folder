use nix::sys::socket;
use nix::sys::socket::AddressFamily as AF;

fn main() {
	let sock = socket::socket(AF::Inet, socket::SockType::Datagram, socket::SockFlag::empty(), None)
		.expect("Socket initialization failed");

	//let data: [u8; 32];
	let msg = "Hi there!";
	let data = msg.as_bytes();

	let addr = socket::SockaddrIn::new(127, 0, 0, 1, 1257);
	match socket::sendto(sock, data, &addr, socket::MsgFlags::empty()) {
		Ok(size) => println!("Sent {size} bytes!"),
		Err(errno) => println!("Send failed: {}", errno.desc())
	};
}
