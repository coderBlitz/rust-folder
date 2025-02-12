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
	let data: Vec<f32> = rng().random_iter().take(N).collect();

	let mut graph = CsrGraph::new(0.);

	for (i, v) in data.iter().enumerate() {
		_ = graph.insert(*v, (0, i));
	}

	for (i, (pos, v)) in graph.row_iter(0).enumerate() {
		assert_eq!(*v, data[i]);
		assert_eq!(pos.0, 0);
		assert_eq!(pos.1, i);
	}
}