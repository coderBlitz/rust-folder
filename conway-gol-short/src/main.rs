use conway_gol_short::*;

fn main() {
	let mut g = Grid::new();
	//_ = g.iter().map(|&c| c.neighbors()).inspect(|c| println!("{c:?}")).count();
	println!("Before: {g:?}");
	/*for c in g.iter() {
		println!("N {c:?} = {:?}", c.neighbors());
	}*/
	g = g.next_gen();
	println!("After: {g:?}");
}
