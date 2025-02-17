use csr::CsrGraph;
use rand::{
	distr::{Distribution, Uniform},
	rng, Rng,
};
use std::collections::HashMap;

#[test]
fn usage_test() {
	const N: usize = 500;
	let range = Uniform::new(0, 10).unwrap();
	let rows: Vec<usize> = range.sample_iter(rng()).take(N).collect();
	let cols: Vec<usize> = range.sample_iter(rng()).take(N).collect();
	let data: Vec<f32> = rng().random_iter().take(N).collect();
	let mut nodes = HashMap::new();

	let mut graph = CsrGraph::new(0.);
	for i in 0..N {
		// Insert data, and only if not already present, add to hashmap.
		if graph.insert(data[i], (rows[i], cols[i])) {
			nodes.insert((rows[i], cols[i]), data[i]);
		}
	}

	for (p, v) in nodes.iter() {
		assert_eq!(graph[(p.0, p.1)], *v, "Expected {v} for position {p:?}");
	}
}

#[test]
fn row_iter_test() {
	const N: usize = 500;
	const ROW: usize = 1;
	let data: Vec<f32> = rng().random_iter().take(N).collect();

	let mut graph = CsrGraph::new(0.);

	for (i, v) in data.iter().enumerate() {
		_ = graph.insert(*v, (ROW, i));
	}

	let row = graph.row_iter(ROW);

	// Check row len.
	assert_eq!(row.count(), N);
	assert_eq!(row.size_hint(), (N, Some(N)));

	let mut count = 0;
	for (i, (pos, v)) in row.enumerate() {
		assert_eq!(*v, data[i]);
		assert_eq!(pos.0, ROW);
		assert_eq!(pos.1, i);
		count += 1;
	}
	assert_eq!(count, N);
}

#[test]
fn col_iter_test() {
	const N: usize = 500;
	const COL: usize = 1;
	let data: Vec<f32> = rng().random_iter().take(N).collect();

	let mut graph = CsrGraph::new(0.);

	for (i, v) in data.iter().enumerate() {
		_ = graph.insert(*v, (i, COL));
	}

	let col = graph.col_iter(COL);

	let mut count = 0;
	for (i, (pos, v)) in col.enumerate() {
		assert_eq!(*v, data[i], "Failed to match entry {i}");
		assert_eq!(pos.0, i);
		assert_eq!(pos.1, COL);
		count += 1;
	}
	assert_eq!(count, N);
}