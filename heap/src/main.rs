mod heap;

use heap::MaxHeap;
use rand::*;
use rand::distributions::Distribution;

fn main() {
	const N: usize = 500;
	let mut rng = rngs::StdRng::from_entropy();
	let uni = rand::distributions::Uniform::new(0, 2*N);

	let mut h = MaxHeap::<usize>::new(N);

	// Insert N stuffs into heap
	println!("Inserting {N} stuffs, and asserting top is max..");
	let mut max = 0;
	for _i in 0..N {
		let val = uni.sample(&mut rng);

		if val > max {
			max = val;
		}

		if let Err(_) = h.push(val) {
			break;
		}

		assert_eq!(*h.top().unwrap(), max);
	}

	// Dump heap (for small N)
	if N < 25 {
		println!("Dumping..");
		let dat = h.dump();
		for i in 0..dat.len() {
			println!("[{i}] = {}", dat[i]);
		}
	}

	// Pop everything and check order
	println!("Removing all and verifying heap property..");
	let mut last = h.pop().unwrap();
	while let Ok(v) = h.pop() {
		assert!(v <= last);

		last = v;
	}
}
