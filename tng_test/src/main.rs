use futures::{SinkExt, StreamExt};
use std::{
	fmt::Write,
	sync::{
		Arc,
		atomic::{AtomicUsize, Ordering},
	},
	time::{Duration, Instant},
};
use tokio::{
	net::{TcpListener, TcpStream},
	sync::Mutex,
	time::timeout,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, accept_async, connect_async};
use tungstenite::{Bytes, Message, Result};

const WS_PORT: u16 = 8080;

async fn respond_to_server(
	websock: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
	count: Arc<AtomicUsize>,
) -> ! {
	let reply_base = "Reply #";
	let mut message = String::new();
	loop {
		// Wait for receipt and verify it's a Ping.
		let item = websock.next().await;
		if let Some(Ok(Message::Text(msg))) = item {
			// Client will always increment.
			count.fetch_add(1, Ordering::Relaxed);

			// Parse value and send back value+1
			if let Some((_, s)) = msg.as_str().split_once('#') {
				if let Ok(n) = u64::from_str_radix(s, 10) {
					message.clear();
					_ = message.write_fmt(format_args!("{reply_base}{}", n + 1));
					_ = websock.send(Message::text(message.clone())).await;
					continue;
				}
			}
		}

		_ = websock.send(Message::Pong(Bytes::new())).await;
	}

	//Ok(())
}
async fn client(dur: &Duration) -> std::io::Result<usize> {
	let addr = ("localhost", WS_PORT);
	let url = format!("ws://{}:{}", addr.0, addr.1);
	//let sock = TcpStream::connect(addr).await?;

	let start = Instant::now();

	if let Ok((mut websock, _)) = connect_async(url).await {
		let count = Arc::new(AtomicUsize::new(0));
		println!("Beginning responding to server..");
		_ = timeout(
			*dur - start.elapsed(),
			respond_to_server(&mut websock, count.clone()),
		)
		.await;

		// Close so server knows we've ended
		_ = websock.close(None).await;

		return Ok(count.load(Ordering::Acquire));
	}

	Ok(0)
}

async fn handle_client(
	//websock: &mut WebSocketStream<TcpStream>,
	ws: Arc<Mutex<WebSocketStream<TcpStream>>>,
	count: Arc<AtomicUsize>,
) -> Result<()> {
	let message_base = "Msg #";
	let mut message = String::new();
	loop {
		// Construct message
		let val = count.load(Ordering::Relaxed);
		message.clear();
		_ = message.write_fmt(format_args!("{message_base}{val}"));

		// Lock socket, send, and wait for reply.
		let mut websock = ws.lock().await;
		if websock.send(Message::text(message.clone())).await.is_ok() {
			// Wait for receipt and verify it's a Pong.
			let item = websock.next().await;
			match item {
				// If text, check for valid reply (previous val + 1)
				Some(Ok(Message::Text(t))) => {
					// Parse value
					if let Some((_, s)) = t.as_str().split_once('#') {
						if let Ok(n) = usize::from_str_radix(s, 10) {
							// Check value
							if n == (val + 1) {
								count.fetch_add(1, Ordering::Relaxed);
							}
						}
					}
				}
				// Ignore Pong replies.
				Some(Ok(Message::Pong(_))) => continue,
				_ => break,
			};
		}
	}

	Ok(())
}

async fn server(dur: &Duration) -> std::io::Result<usize> {
	let addr = ("localhost", WS_PORT);
	let sock = TcpListener::bind(addr).await?;

	let start = Instant::now();
	if let Ok(Ok((stream, _))) = timeout(*dur, sock.accept()).await {
		println!("Server stream started.");
		if let Ok(mut websock) = accept_async(stream).await {
			let count = Arc::new(AtomicUsize::new(0));
			println!("Beginning client handling..");
			let ws = Arc::new(Mutex::new(websock));
			_ = timeout(
				*dur - start.elapsed(),
				//handle_client(&mut websock, count.clone()),
				handle_client(ws.clone(), count.clone()),
			)
			.await;

			// Close so client knows we've ended.
			//_ = websock.close(None).await;
			let mut locked_sock = ws.lock().await;
			_ = locked_sock.close(None).await;

			return Ok(count.load(Ordering::Acquire));
		}
	}

	Ok(0)
}

#[derive(Debug)]
enum Mode {
	Client,
	Server,
}

fn main() {
	// Run parameters
	let mut mode = Mode::Client;
	let mut dur = Duration::from_secs(3);
	for arg in std::env::args() {
		match arg.as_str() {
			"-c" => mode = Mode::Client,
			"-s" => mode = Mode::Server,
			a if a.starts_with('-') => {
				eprintln!("Unknown flag {a}");
				return;
			}
			a => {
				if let Ok(n) = u64::from_str_radix(a, 10) {
					dur = Duration::from_secs(n);
				}
			}
		};
	}

	let mode_name = match mode {
		Mode::Client => "client",
		Mode::Server => "server",
	};
	println!("Running {mode_name} for {}s.", dur.as_secs());

	// Start in whichever mode.
	let rt = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap();
	let total = match mode {
		Mode::Client => rt.block_on(client(&dur)),
		Mode::Server => rt.block_on(server(&dur)),
	}
	.unwrap();

	let mode_str = match mode {
		Mode::Client => "received",
		Mode::Server => "sent",
	};
	println!("----------\n{mode_name} {mode_str} {total} messages.\n----------\n");
}
