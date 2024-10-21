use kmeans::vec3::Vec3;
use std::{cmp::Ordering, time};
use rand::{self, distributions::Distribution};

#[derive(Clone)]
struct Centroid<'a> {
	pos: Vec3,
	points: Vec<&'a Vec3>,
}
impl<'a> Centroid<'a> {
	fn new() -> Self {
		Centroid {
			pos: Vec3::default(),
			points: Vec::new(),
		}
	}

	// Update center and return magnitude of change.
	fn update_center(&mut self) -> f64 {
		let orig = self.pos;

		if self.points.len() > 0 {
			self.pos = self.points.iter()
				.fold(Vec3::default(), |acc, v| acc + **v) / self.points.len() as f64;
		}

		self.points.clear();

		(orig - self.pos).norm()
	}
}
impl<'a> PartialEq for Centroid<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.pos == other.pos
	}
}
impl<'a> PartialEq<Vec3> for Centroid<'a> {
	fn eq(&self, other: &Vec3) -> bool {
		self.pos == *other
	}
}
impl<'a> std::ops::Sub<Vec3> for &Centroid<'a> {
	type Output = Vec3;
	fn sub(self, rhs: Vec3) -> Self::Output {
		self.pos - rhs
	}
}

fn main() {
	const N: usize = 100;
	const K: usize = 2;

	let r = &mut rand::thread_rng();
	let d1 = &rand::distributions::Uniform::new(Vec3(0.,0.,0.), Vec3(1., 1., 1.));
	let d2 = &rand::distributions::Uniform::new(Vec3(1.,1.,1.), Vec3(3.,3.,3.));

	let mut points: Vec<Vec3> = (0..N/2).map(|_| d1.sample(r)).collect();
	points.extend((0..N/2).map(|_| d2.sample(r)));

	let mut centroids: [Centroid; K] = std::array::from_fn(|_| Centroid::new());

	// Do loop
	let t = time::Instant::now();

	// Until convergence:
	// 1. Loop over all points.
	// 2. Find closest centroid, and assign that to each point.
	// 3. Update centroid position by averaging all points assigned to it.
	let mut changes = [f64::INFINITY; K];
	let mut iters = 0;
	while *changes.iter().max_by(|v1, v2| v1.partial_cmp(v2).unwrap_or(Ordering::Less)).unwrap() > 0.01 {
		for point in points.iter() {
			let mut closest = (0, (&centroids[0] - *point).norm());
			for (i, ctrd) in centroids.iter().enumerate() {
				// Do distance calculation.
				let dist = (ctrd - *point).norm();
				//println!("Dist to {i} is {dist}");
				if dist < closest.1 {
					closest = (i, dist);
				}
			}

			centroids[closest.0].points.push(point);
		}

		for (i, ctrd) in centroids.iter_mut().enumerate() {
			//println!("Centroids {i} has {} points", ctrd.points.len());
			changes[i] = ctrd.update_center();
		}

		iters += 1;
	}

	println!("{N} points and {K} clusters.");
	println!("Took {iters} iterations over {}s.", t.elapsed().as_secs_f64());
	for (i, ctrd) in centroids.iter().enumerate() {
		println!("Centroids {i} has center {:?}", ctrd.pos);
	}
}
