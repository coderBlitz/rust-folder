//! Connection status logger.
//!
//! Client/server intended to test connectivity properties over an extended period.
//!  Specifically, after some interval a connection test for a duration will
//!  be used to determine upload, download, and latency (unloaded, and loaded) as
//!  observed from the client.
// TODO: Calculate and record variance, mean, median, etc.

use std::{
	io::{self, Read, Write},
	net::{SocketAddr, TcpStream},
	time::{Duration, Instant},
	thread::sleep,
};

const TRIAL_DURATION: Duration = Duration::from_secs(12);
const NUM_TRIALS: usize = 50;
const TRIAL_DELAY: Duration = Duration::from_millis(200);

fn idle_latency(conn: &mut TcpStream) -> io::Result<f64> {
	let mut sum = 0.0;
	let buf = [];
	for _ in 0..NUM_TRIALS {
		let start = Instant::now();
		_ = conn.write(&buf)?;
		sum += start.elapsed().as_secs_f64();

		sleep(TRIAL_DELAY);
	}

	Ok(sum / NUM_TRIALS as f64)
}

fn upload_small(conn: &mut TcpStream) -> io::Result<f64> {
	let mut sum = 0;
	let buf: Vec<u8> = (0..255).cycle().take(516).collect(); // Minimum required MSS size, assuming max IP header size (60 bytes).
	let start = Instant::now();
	while start.elapsed() < TRIAL_DURATION {
		sum += conn.write(&buf)?;
	}

	Ok(sum as f64 / TRIAL_DURATION.as_secs_f64())
}

fn upload_large(conn: &mut TcpStream) -> io::Result<f64> {
	let mut sum = 0;
	let buf: Vec<u8> = (0..255).cycle().take(1440).collect(); // Max non-fragmented IP packet
	let start = Instant::now();
	while start.elapsed() < TRIAL_DURATION {
		sum += conn.write(&buf)?;
	}

	Ok(sum as f64 / TRIAL_DURATION.as_secs_f64())
}

fn download(conn: &mut TcpStream) -> io::Result<f64> {
	let mut sum = 0;
	let mut buf = [0; 4096];
	let start = Instant::now();
	while start.elapsed() < TRIAL_DURATION {
		sum += conn.read(&mut buf)?;
	}

	Ok(sum as f64 / TRIAL_DURATION.as_secs_f64())
}

// Reverse order of client function.
fn server(sock_addr: SocketAddr) {
}

fn client(sock_addr: SocketAddr) {
	let sock = TcpStream::connect_timeout(&sock_addr, Duration::from_secs(5)).expect("Connect failed");
	sock.set_read_timeout(Some(Duration::from_secs(1))).expect("Failed to set timeout.");
	sock.set_write_timeout(Some(Duration::from_secs(1))).expect("Failed to set timeout.");

	loop {
		/* Do test */
	}
}

fn main() {
	let args: Vec<String> = std::env::args().collect();

	if args.len() < 2 {
		eprintln!("Usage: conn_logger [options] IP_ADDR");
		eprintln!("    -l    Bind to IP_ADDR instead of connecting.");
		return
	}

	let mut bind = false;
	let mut it = args.iter().skip(1); // Argument iterator
	while let Some(arg) = it.next() {
		println!("Arg: {arg}");
		match arg.as_str() {
			"-l" => bind = true,
			_ => {},
		};
	}
}
