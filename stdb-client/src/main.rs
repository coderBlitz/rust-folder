use bindings::*;
use spacetimedb_sdk::{DbContext, event::Status};

mod bindings;

use std::sync::mpsc;

fn main() {
	let (send, recv) = mpsc::channel();
	let open_send = send.clone();
	let dbcon = DbConnection::builder()
		.with_uri("127.0.0.1:3000")
		.with_module_name("stuff")
		.on_connect( move |_,_,_| open_send.send(()).unwrap())
		.on_disconnect( move |_,_| send.send(()).unwrap())
		.build().unwrap();

	// Wait for connection to open.
	while let Err(_) = recv.try_recv() {
		_ = dbcon.frame_tick();
	}

	println!("Connection open! Adding stuff..");

	// Call reducer to add stuff.
	const N: usize = 100;
	let mut insert_count = 0;
	let mut thing = Vec2 { x: 0., y: 0. };
	let (evt_send, evt_recv) = mpsc::channel();
	let callback = dbcon.reducers.on_add(move |ctx, _,_,_| evt_send.send(ctx.event.status.clone()).unwrap());
	for i in 0..N {
		//thing.x = i as f64;
		thing.y = i as f64;
		_ = dbcon.reducers.add(i.to_string(), thing.clone(), thing.clone());
	}

	// Tick while all adds have not been called at least once. Ignoring status (and retry) for simplicity.
	println!("Ticking till adds complete..");
	while insert_count != N {
		_ = dbcon.frame_tick();

		// Receive reducer statuses, ignoring actual value.
		while let Ok(status) = evt_recv.try_recv() {
			insert_count += 1;

			if let Status::Failed(_) = status {
				eprintln!("Reducer failed.");
			};
		}
	}

	// Remove unneeded callback
	dbcon.reducers.remove_on_add(callback);


	// Disconnect and wait for connection to close.
	println!("Disconnecting..");
	_ = dbcon.disconnect();
	while let Err(_) = recv.try_recv() {
		_ = dbcon.frame_tick();
	}
}
