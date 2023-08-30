use std::collections::HashSet;
use std::ops::Deref;
#[derive(Debug)]
pub struct Grid(HashSet<Cell>);
impl Grid {
	pub fn new() -> Self {
		Grid([Cell(0,0), Cell(1,0), Cell(2,0)].into_iter().collect()) // Blinker setup for simple test
	}
	pub fn next_gen(self) -> Grid {
		Grid(
			self.iter().filter(|c| (2..=3).contains(&self.intersection(&c.neighbors()).count()))
				.copied().collect::<HashSet<Cell>>()
				.union(&self.iter().flat_map(|c| c.neighbors().0.into_iter())
					.filter(|p| self.intersection(&p.neighbors()).filter(|&p2| p2 != p).count() == 3).collect()
				).copied().collect()
		)
	}
}
impl Deref for Grid {
	type Target = HashSet<Cell>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Cell(isize, isize);
impl Cell {
	pub fn neighbors(&self) -> Grid {
		Grid((-1..=1).flat_map(|x| (-1..=1).map(move |y| Cell(self.0 + x, self.1 + y) ))
			.filter(|c| c != self)
			.collect()
		)
	}
}
