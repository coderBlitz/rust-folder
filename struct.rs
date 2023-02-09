struct Point2 {
	x: f64,
	y: f64,
}

fn main() {
	let pt = Point2 { x: 1.5, y: -2.0};

	println!("Point: ({}, {})", pt.x, pt.y);
}
