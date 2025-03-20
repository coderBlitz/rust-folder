use bindings::*;
use rand;
use spacetimedb_sdk::{DbContext, event::Status, Table};
use std::{
	time::Instant,
	sync::mpsc,
};

mod bindings;

fn insert_batched(n: usize, batch: usize, dbcon: &DbConnection) {
	let mut total = 0;
	let mut thing = Vec2 { x: 0., y: 0. };
	let (evt_send, evt_recv) = mpsc::channel();
	let callback = dbcon.reducers.on_add(move |ctx, _,_,_| evt_send.send(ctx.event.status.clone()).unwrap());

	while total < n {
		// Insert batch of entries.
		for i in 0..batch {
			thing.x = rand::random();
			thing.y = rand::random();
			_ = dbcon.reducers.add(i.to_string(), thing.clone(), thing.clone());
		}

		// Tick while all adds have not been called at least once. Ignoring status (and retry) for simplicity.
		let mut batch_count = 0;
		while batch_count < batch {
			_ = dbcon.frame_tick();

			// Receive reducer statuses, ignoring actual value.
			while let Ok(status) = evt_recv.try_recv() {
				batch_count += 1;
				total += 1;

				if let Status::Failed(_) = status {
					eprintln!("Reducer failed.");
				};
			}
		}
	}

	// Remove unneeded callback
	dbcon.reducers.remove_on_add(callback);
}

fn insert_nobatch(n: usize, dbcon: &DbConnection) {
	let mut total = 0;
	let mut thing = Vec2 { x: 0., y: 0. };
	let (evt_send, evt_recv) = mpsc::channel();
	let callback = dbcon.reducers.on_add(move |ctx, _,_,_| evt_send.send(ctx.event.status.clone()).unwrap());

	while total < n {
		thing.x = rand::random();
		thing.y = rand::random();
		_ = dbcon.reducers.add(total.to_string(), thing.clone(), thing.clone());
		_ = dbcon.frame_tick();

		// Receive reducer statuses, ignoring actual value.
		while let Ok(status) = evt_recv.try_recv() {
			total += 1;

			if let Status::Failed(_) = status {
				eprintln!("Reducer failed.");
			};
		}
	}

	if n > total {
		eprintln!("Problem!!!");
	}

	// Remove unneeded callback
	dbcon.reducers.remove_on_add(callback);
}

fn insert_bulk(n: usize, dbcon: &DbConnection) {
	let mut total = 0;
	let mut thing = Vec2 { x: 0., y: 0. };
	let (evt_send, evt_recv) = mpsc::channel();
	let callback = dbcon.reducers.on_add(move |ctx, _,_,_| evt_send.send(ctx.event.status.clone()).unwrap());

	// Queue all adds at once
	for i in 0..n {
		thing.x = rand::random();
		thing.y = rand::random();
		_ = dbcon.reducers.add(i.to_string(), thing.clone(), thing.clone());
	}

	// Tick everything at once.
	while total < n {
		_ = dbcon.frame_tick();

		// Receive reducer statuses, ignoring actual value.
		while let Ok(status) = evt_recv.try_recv() {
			total += 1;

			if let Status::Failed(_) = status {
				eprintln!("Reducer failed.");
			};
		}
	}

	// Remove unneeded callback
	dbcon.reducers.remove_on_add(callback);
}

fn insert_vec(n: usize, dbcon: &DbConnection) {
	let mut thing = Vec2 { x: 0., y: 0. };
	let (evt_send, evt_recv) = mpsc::channel();
	let callback = dbcon.reducers.on_add_batch(move |ctx, _| evt_send.send(ctx.event.status.clone()).unwrap());

	let mut things = Vec::with_capacity(n);

	// Create all entries.
	println!("Creating vec..");
	for i in 0..n {
		thing.x = rand::random();
		thing.y = rand::random();
		things.push(Thing {
			id: 0, // Must be 0 for autoinc to work
			name: i.to_string(),
			one: thing.clone(),
			two: thing.clone()
		});
	}

	// Batch insert entire vec.
	println!("Batch inserting vec..");
	_ = dbcon.reducers.add_batch(things);

	// Tick everything at once.
	while let Err(_) = evt_recv.try_recv() {
		if let Err(e) = dbcon.frame_tick() {
			eprintln!("Frame tick error: {e}");
			break;
		}
	}

	// Remove unneeded callback
	dbcon.reducers.remove_on_add_batch(callback);
}

fn insert_vec_batch(n: usize, batch: usize, dbcon: &DbConnection) {
	let mut thing = Vec2 { x: 0., y: 0. };
	let (evt_send, evt_recv) = mpsc::channel();
	let callback = dbcon.reducers.on_add_batch(move |ctx, _| evt_send.send(ctx.event.status.clone()).unwrap());

	let mut total = n;

	for _ in (0..n).step_by(batch) {
		let mut things = Vec::with_capacity(batch);

		// Create batch vec entries.
		let remain = batch.min(total);
		for i in 0 .. remain {
			thing.x = rand::random();
			thing.y = rand::random();
			things.push(Thing {
				id: 0, // Must be 0 for autoinc to work
				name: i.to_string(),
				one: thing.clone(),
				two: thing.clone()
			});
		}

		// Batch insert entire vec.
		//println!("Batch inserting vec..");
		_ = dbcon.reducers.add_batch(things);

		total -= remain;
	}

	// Tick everything.
	debug_assert_eq!(total, 0);
	let num_batches = n.div_ceil(batch);
	while total < num_batches {
		_ = dbcon.frame_tick();

		// Receive reducer statuses, ignoring actual value.
		while let Ok(status) = evt_recv.try_recv() {
			total += 1;

			if let Status::Failed(_) = status {
				eprintln!("Reducer failed.");
			};
		}
	}

	// Remove unneeded callback
	dbcon.reducers.remove_on_add_batch(callback);
}

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

	println!("Connection open!");

	// Add `N` total entries in batches of `BATCH`.
	const N: usize = 1000000;
	const BATCH: usize = 100000;
	println!("Inserting {N} entries..");
	let start = Instant::now();
	//insert_batched(N, BATCH, &dbcon);
	//insert_nobatch(N, &dbcon);
	//insert_bulk(N, &dbcon);
	//insert_vec(N, &dbcon); // Somewhere between 600k and 700k websocket "Space limit exceeded" error.
	insert_vec_batch(N, BATCH, &dbcon);
	let time_taken = Instant::now() - start;
	println!("Total insertion time with batch size {BATCH} took {}ms.", time_taken.as_millis());
	//println!("Total insertion time was {}ms.", time_taken.as_millis());

	// Disconnect and wait for connection to close.
	println!("Disconnecting..");
	_ = dbcon.disconnect();
	while let Err(_) = recv.try_recv() {
		_ = dbcon.frame_tick();
	}
}
