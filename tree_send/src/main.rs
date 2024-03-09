//! Demonstrate a binary tree send broadcast algorithm.
//!
//! # Algorithm
//! ## Context
//! The goal of this algorithm is to distribute a piece of information in less
//!  than linear time, specifically, logarithmic. The "ticks" required for N
//!  processes should be `ceil(log2(N))`.
//!
//! The process can be visualized as a tree of nodes sequentially labelled from
//!  1 to N, left-to-right row-wise. This is the same scheme as used by binary
//!  heaps. Such a tree would look like the following:
//!
//! ```text
//!        1
//!      /   \
//!     2     3
//!    / \   / \
//!   4   5 6   7
//!       ...
//! ```
//!
//! ## Formal process
//! Let `n` nodes have IDs `[1,n]`. Each node will track its current "position"
//!  starting from its own ID, and serves as a reference when referring to
//!  "left" or "right" child nodes. A node can "move" its position to either
//!  child by multiplying by 2, then adding 0 or 1 for the left or right child,
//!  respectively.
//!
//! Steps:
//! 1. If not root node, receive the data.
//! 2. Send to left child (ID 2n), if it exists.
//! 3. If ID of self != 2^k for some k, then move left.
//! 4. While the right child (ID 2n+1) exists:
//!   A. Send to right child.
//!   B. Move right.

use std::sync::{
	Arc,
	Barrier,
	mpsc::channel,
};
use std::thread;


fn main() {
	const NTHREADS: usize = 62;
	assert!(NTHREADS > 0); // Main assumes at least 1 worker exists.

	// Overhead variables.
	let mut senders = Vec::with_capacity(NTHREADS);
	let mut threads = Vec::with_capacity(NTHREADS);
	let start_bar = Arc::new(Barrier::new(NTHREADS+1)); // Plus 1 for main thread

	// Channel for main thread.
	let (main_send, main_recv) = channel();

	// Create channels, push send side to above vec, save recv side here.
	let r_channels: Vec<_> = (0..NTHREADS).map(|_| {
		let (s,r) = channel();
		senders.push(s);
		r
	}).collect();

	// Spawn NTHREADS worker threads (nodes).
	for (id, r) in r_channels.into_iter().enumerate() {
		// Make copies of barrier, root channel, and worker channels.
		let bar = start_bar.clone();
		let root_send = main_send.clone();
		let sends = senders.clone();

		// Spawn worker.
		let t = thread::spawn(move || {
			// Wait for main to release workers.
			bar.wait();

			// Position tracker.
			let mut pos = id + 1;

			// Receive data.
			let data = match r.recv() {
				Ok(d) => d,
				Err(_) => return,
			};

			// SPECIAL: Send data back to root for verification purposes.
			_ = root_send.send(data);

			// Send to left (if it exists).
			if 2*pos <= NTHREADS {
				_ = sends[2*pos-1].send(data);
			}

			// Conditionally move left.
			if pos.count_ones() != 1 {
				pos *= 2;
			}

			// Loop send right.
			pos = 2*pos + 1;
			while pos <= NTHREADS {
				_ = sends[pos-1].send(data);

				pos = 2*pos + 1; // Move right.
			}
		});

		threads.push(t);
	}

	// Send initial data to root, then drop so receiving end closes properly.
	let base = 1;
	println!("Sending data {base} to root..");
	_ = senders[0].send(base);
	std::mem::drop(main_send);

	// Off to the races!
	start_bar.wait();
	println!("Launch!");

	// Receive values and sum.
	let mut sum = 0;
	while let Ok(v) = main_recv.recv() {
		sum += v;
	}

	println!("Sum is {sum}");

	// Join threads.
	for t in threads {
		_ = t.join();
	}

	// Assert correctness.
	assert_eq!(NTHREADS * base, sum);
	println!("Success!");
}
