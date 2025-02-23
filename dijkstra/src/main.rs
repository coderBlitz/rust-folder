use csr::CsrGraph;
use std::{
	cmp::{Ordering, Reverse},
	collections::{BinaryHeap, HashSet},
};

/// Node(ID, shortest_dist, previous_node)
#[derive(Clone, Copy, Debug)]
struct Node(usize, f64, usize);
impl PartialEq for Node {
	fn eq(&self, rhs: &Self) -> bool {
		self.0 == rhs.0
	}
}
impl Eq for Node {}
impl PartialOrd for Node {
	fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
		Some(self.cmp(rhs))
	}
}
impl Ord for Node {
	fn cmp(&self, rhs: &Self) -> Ordering {
		self.1.partial_cmp(&rhs.1).unwrap()
	}
}

pub fn dijkstra(graph: &CsrGraph<f64>, start: usize, end: usize) -> Option<(f64, Vec<usize>)> {
	let mut path = BTreeMap::new();
	let mut nodes = BinaryHeap::new();
	let mut visited = HashSet::new();

	// Modify structures with starting position.
	nodes.push(Reverse(Node(start, 0., 0)));

	// Iterate while heap is not empty.
	let mut count = 0;
	while let Some(node) = nodes.pop() {
		// Insert node to visited set, and skip if already visited.
		if !visited.insert(node.0.0) {
			continue;
		}

		// Insert node to tracking map.
		path.insert(node.0.0, (node.0.1, node.0.2));

		// If current node is target, stop.
		if node.0.0 == end {
			break;
		}

		// Add neighbors of current node.
		for ((_, nbr), weight) in graph.row_iter(node.0.0) {
			// Don't add visited nodes to heap.
			if visited.contains(&nbr) {
				continue;
			}

			// Otherwise insert to heap.
			nodes.push(Reverse(Node(nbr, node.0.1 + weight, node.0.0)));
		}

		count += 1;
	}
	println!("Count was {count}.");

	// Go through visited nodes to construct reversed path.
	let Some(end_node) = path.get(&end) else {
		return None
	};
	let total_cost = end_node.0;
	let mut rev_path = vec![end];
	let mut prev = end_node.1;
	while prev != start {
		// Push previous node then get next previous.
		rev_path.push(prev);
		let node = path.get(&prev).unwrap();
		prev = node.1;
	}
	rev_path.push(start); // Add start node

	// Return real path as sequences of IDs.
	rev_path.reverse();
	Some((total_cost, rev_path))
}

/** Create graph 1.

Graph is as follows:

```
A---B
| \ |
C---D
```

The weights are such that the shortest path is `A-C-D-B`.
*/
fn graph1() -> (CsrGraph<f64>, (usize,usize)) {
	let mut gr = CsrGraph::new(f64::INFINITY);

	gr.insert(3.75, (0, 1));
	gr.insert(1.0, (0, 2));
	gr.insert(2.5, (0, 3));
	gr.insert(1.5, (2, 0));
	gr.insert(1.0, (2, 3));
	gr.insert(1.0, (3, 1));
	gr.insert(2.0, (3, 0));
	gr.insert(2.0, (3, 2));

	(gr, (0, 1))
}

/**	Computerphile graph (Corect answer is 19->2->8->7->5, or 'S'->'B'->'H'->'G'->'E')
*/
fn computerphile() -> (CsrGraph<f64>, (usize, usize)) {
	let mut gr = CsrGraph::new(f64::INFINITY);

	gr.insert( 3.0, (1,  2)); // AB
	gr.insert( 4.0, (1,  4)); // AD
	gr.insert( 7.0, (1,  19)); // AS
	gr.insert( 3.0, (2,  1)); // BA
	gr.insert( 2.0, (2,  19)); // BS
	gr.insert( 4.0, (2,  4)); // BD
	gr.insert( 1.0, (2,  8)); // BH
	gr.insert( 3.0, (3,  19)); // CS
	gr.insert( 2.0, (3,  12)); // CL
	gr.insert( 4.0, (4,  1)); // DA
	gr.insert( 4.0, (4,  2)); // DB
	gr.insert( 5.0, (4,  6)); // DF
	gr.insert( 2.0, (5,  7)); // EG
	gr.insert( 5.0, (5,  11)); // EK
	gr.insert( 5.0, (6,  4)); // FD
	gr.insert( 3.0, (6,  8)); // FH
	gr.insert( 2.0, (7,  8)); // GH
	gr.insert( 2.0, (7,  5)); // GE
	gr.insert( 3.0, (8,  6)); // HF
	gr.insert( 1.0, (8,  2)); // HB
	gr.insert( 2.0, (8,  7)); // HG
	gr.insert( 4.0, (9,  12)); // IL
	gr.insert( 6.0, (9,  10)); // IJ
	gr.insert( 4.0, (9,  11)); // IK
	gr.insert( 4.0, (10,  11)); // JK
	gr.insert( 4.0, (10,  12)); // JL
	gr.insert( 6.0, (10,  9)); // JI
	gr.insert( 4.0, (11,  9)); // KI
	gr.insert( 4.0, (11,  10)); // KJ
	gr.insert( 5.0, (11,  5)); // KE
	gr.insert( 2.0, (12,  3)); // LC
	gr.insert( 4.0, (12,  9)); // LI
	gr.insert( 4.0, (12,  10)); // LJ
	gr.insert( 7.0, (19,  1)); // SA
	gr.insert( 2.0, (19,  2)); // SB
	gr.insert( 3.0, (19,  3)); // SC

	(gr, (19, 5))
}

fn main() {
	let (gr, (start, end)) = computerphile();
	let (len, path) = dijkstra(&gr, start, end);
	println!("Path with len {len} is {path:?}");
}
