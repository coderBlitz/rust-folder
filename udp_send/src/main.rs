use std::net;

fn main() -> std::io::Result<()> {
	let sock = net::UdpSocket::bind("127.0.0.1:1258")?;

	//let data: [u8; 32];
	let msg = "Hi there!";
	let data = msg.as_bytes();

	sock.send_to(data, "127.0.0.1:1257").expect("Send failed");

	Ok(())
}
