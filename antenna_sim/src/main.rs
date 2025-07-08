//! Antenna radiation pattern sim
//!
//! Goal is to model the radiation pattern of antennas, in a very
//!  ideal/abstract way. Focus is on the "shape", i.e. where the regions of
//!  high constructive interference are.
//!
//! TODO: Model simple geometric constructions (point/lines, circles)
//! TODO: Output PPM file with results.

use std::{
	convert,
	f64::consts::PI,
	fs,
	io::{self, BufWriter, Write},
	ops,
	time::Instant,
};
use png;

#[derive(Copy, Clone, Default, PartialEq)]
struct Vec2 (f64,f64);
#[derive(Copy, Clone, Default, PartialEq)]
struct Vec3 (f64,f64,f64);

impl Vec3 {
	pub fn norm(&self) -> f64 {
		(self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
	}
}
impl convert::From<Vec2> for Vec3 {
	fn from(val: Vec2) -> Vec3 {
		Vec3 (val.0, val.1, 0.)
	}
}
impl convert::From<f64> for Vec3 {
	fn from(val: f64) -> Vec3 {
		Vec3 (val, val, val)
	}
}
impl ops::Add for Vec3 {
	type Output = Self;

	fn add(self, rhs: Vec3) -> Vec3 {
		Vec3 (self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
	}
}
impl ops::Add<f64> for Vec3 {
	type Output = Self;

	fn add(self, rhs: f64) -> Vec3 {
		Vec3 (self.0 + rhs, self.1 + rhs, self.2 + rhs)
	}
}
impl ops::Sub for Vec3 {
	type Output = Self;

	fn sub(self, rhs: Vec3) -> Vec3 {
		Vec3 (self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
	}
}
impl ops::Sub<f64> for Vec3 {
	type Output = Self;

	fn sub(self, rhs: f64) -> Vec3 {
		Vec3 (self.0 - rhs, self.1 - rhs, self.2 - rhs)
	}
}
impl ops::Mul for Vec3 {
	type Output = f64;

	fn mul(self, rhs: Vec3) -> f64 {
		self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
	}
}
impl ops::Mul<f64> for Vec3 {
	type Output = Vec3;

	fn mul(self, rhs: f64) -> Vec3 {
		Vec3 (self.0 * rhs, self.1 * rhs, self.2 * rhs)
	}
}
impl ops::Div<f64> for Vec3 {
	type Output = Vec3;

	fn div(self, rhs: f64) -> Vec3 {
		Vec3 (self.0 / rhs, self.1 / rhs, self.2 / rhs)
	}
}

/// Saves image data to the given file, using PPM (P6) binary format
///
/// * `fname` - File path to save to.
/// * `dimensions` - Tuple where (width, height) are the dimensions of the image
/// * `colors` - Slice of Vec3 tuple, where each tuple represents
///  normalized (red, green, blue) values for a given pixel.
#[allow(dead_code)]
fn write_ppm(fname: &str, dimensions: (usize, usize), colors: &[Vec3]) -> io::Result<()> {
	let fp = &mut BufWriter::new(fs::File::create(fname)?);

	let header = format!("P6 {} {} {}\n", dimensions.0, dimensions.1, 255);

	fp.write(header.as_bytes())?;
	for e in colors {
		fp.write(&[e.0 as u8, e.1 as u8, e.2 as u8])?;
	}

	Ok(())
}

 // Speed of light in a vacuum.
const SOL: f64 = 299_792_458.0;

/// Represents any radiating antenna
trait Antenna {
	/// Returns the signal strengh contributed by this antenna at `point` on
	///  `freq`, assuming a cosine signal.
	///
	/// * `freq` - Frequency in herz.
	/// * `phase` - Phase of signal in radians
	/// * `point` - Point in grid space
	///
	/// TODO: Determine if point should be passed as reference
	fn signal_at(&self, freq: f64, phase: f64, point: &Vec3) -> f64;

	/// Returns all points where the antenna is present
	/// TODO: Remove this and do drawing a better way. Just have explicit primitives and use associated type
	fn footprint(&self) -> Vec<Vec3>;
}

/// Point antenna
///
/// Antenna that exists at a single point in space. `delay` is a phase offset
///  from 0.
struct PointAnt {
	pos: Vec3,
	delay: f64,
	cutoff: f64,
}
impl PointAnt {
	#[allow(dead_code)]
	fn new() -> Self {
		Self {
			pos: Vec3 (0., 0., 0.),
			delay: 0.,
			cutoff: 0.
		}
	}
	fn new_at(pos: Vec3, delay: f64, cutoff: f64) -> Self {
		Self {
			pos,
			delay,
			cutoff
		}
	}
	#[allow(dead_code)]
	fn pos(&self) -> Vec3 {
		self.pos
	}
}
impl Antenna for PointAnt {
	fn signal_at(&self, freq_hz: f64, phase: f64, point: &Vec3) -> f64 {
		//let freq_r: f64 = freq_hz * 2. * PI;
		let dist = (*point - self.pos).norm();
		const POWER: f64 = 2000.;
		if dist <= self.cutoff {
			// dist / wavelength == dist / (SOL / freq) == (dist * freq) / SOL
			// Equation with inverse square radiation power calculation.
			(2. * PI * (dist * freq_hz / SOL) + phase + self.delay).sin() * POWER / (dist * dist)
			// Equation without power calculation
			//(2. * PI * (dist * freq_hz / SOL) + phase + self.delay).sin()
		} else {
			0.
		}
	}

	fn footprint(&self) -> Vec<Vec3> {
		let mut v = Vec::with_capacity(1);
		v.push(self.pos);
		v
	}
}

struct LineAnt {
	start: Vec3,
	end: Vec3,
	phase: f64
}
impl Antenna for LineAnt {
	fn signal_at(&self, freq: f64, phase: f64, point: &Vec3) -> f64 {
		// Equation for strength at a point is (co)sine of distance, then
		//  summed for entire line. Becomes integral from start to end of line.
		// Below is are integrals for sine and cosine
		// Sine: 2. * (((x).sqrt()).sin() - (x).sqrt() * ((x).sqrt()).cos())
		// Cosine: 2. * ((x).sqrt * ((x).sqrt()).sin() + ((x).sqrt()).cos())
		0.
	}

	fn footprint(&self) -> Vec<Vec3> {
		let mut v = Vec::with_capacity(1);
		v.push(self.start);
		v.push(self.end);
		v
	}
}

fn main() {
	let argv: Vec<String> = std::env::args().collect();

	let mut phi: f64 = 0.; // For ease of use, phi is in degrees.
	if argv.len() > 2 {
		phi = argv[2].parse().unwrap_or(0.);
	}

	// Dimensions is (width, height) in meters.
	let dims = (600, 600);
	const DIST_SCALE: f64 = 1.0;
	let pix_count = dims.0 * dims.1;
	let pixels = &mut Vec::with_capacity(pix_count);

	let center = Vec3 ((dims.0-1) as f64 / 2., (dims.1-1) as f64 / 2., 0.);
	let freq_hz = 10_000_000.;
	phi = phi * PI / 180.; // Scale phase to frequency so the phase period is always 2pi
	println!("Setting phi to {phi}");

	let sources = &mut Vec::<Box<dyn Antenna>>::new();

	const N: usize = 4;
	let offset = Vec3 (0., 6., 0.);
	let base_offset = center - offset * (N-1) as f64 / 2.;
	for i in 0..N {
		let p_pos = base_offset + offset * i as f64;
		let p = PointAnt::new_at(p_pos, phi * i as f64, f64::INFINITY);
		sources.push(Box::new(p));
	}

	/*
	let p1 = PointAnt::new_at(center, -phi, 200.);
	let p2 = PointAnt::new_at(center + half_offset, 0., 200.);
	let p3 = PointAnt::new_at(center + half_offset * 2., phi, 200.);
	sources.push(Box::new(p1));
	sources.push(Box::new(p2));
	sources.push(Box::new(p3));
	*/

	// Calculate each pixel
	let mut pix_pos;
	pixels.resize(pix_count, 0.);
	let start = Instant::now();
	// Iterate antennas and compute pixels (faster than iterating pixels as outer loop).
	for s in sources.iter() {
		for i in 0..pix_count {
			pix_pos = Vec3 (DIST_SCALE * (i / dims.0) as f64, DIST_SCALE * (i % dims.0) as f64, 0.);

			pixels[i] += s.signal_at(freq_hz, 0., &pix_pos);
		}
	}

	// Normalize the pixel values
	let pix_max: f64 = 255. / sources.len() as f64;
	let mut colors: Vec<Vec3> = pixels.iter().map(|x|
		if *x >= 0. {
			Vec3 (x * pix_max, x * pix_max, 0.)
		} else {
			Vec3 (-x * pix_max, 0., -x * pix_max)
		}
	).collect();

	let end = start.elapsed();
	println!("Compute time (+ normalize) = {}", end.as_secs_f64());

	/* Overlay drawing
	*/
	// Debug draw red cross through center
	let row_start = (center.1 as usize) * dims.0;
	for i in 0..dims.0 {
		colors[row_start + i] = Vec3 (255.0, 0., 0.);
	}
	let col = center.0 as usize;
	for i in 0..dims.1 {
		colors[i * dims.0 + col] = Vec3 (255.0, 0., 0.);
	}

	// Draw emitters (TODO: Change when more than point antenna are used)
	for s in sources {
		let f = s.footprint();
		for p in f {
			let (x,y) = (p.0 as usize, p.1 as usize);
			if x < dims.1 && y < dims.0 {
				colors[x * dims.1 + y] = Vec3 (0., 0., 255.);
			}
		}
	}

	/* Write out image data
	*/
	let mut base = String::from("/tmp/out");
	if argv.len() > 1 {
		base = argv[1].clone();
		println!("Outputting to {base}");
	}

	// Save image to PPM
	//write_ppm(&format!("{base}.ppm"), dims, &colors).expect("Write failed");

	// Save image to PNG (smaller than PPM for larger resolutions, but only by 10-15%)
	let out = &mut fs::File::create(format!("{base}.png")).expect("PNG file creation failed");
	let mut ping = png::Encoder::new(out, dims.0 as u32, dims.1 as u32);
	ping.set_color(png::ColorType::Rgb);
	ping.set_depth(png::BitDepth::Eight);
	let mut ping_writer = ping.write_header().expect("PNG writer failed");
	let mut ping_streamer = ping_writer.stream_writer().expect("Streamer failed");
	for c in colors {
		//ping_writer.write_image_data(&[c.0 as u8, c.1 as u8, c.2 as u8]).unwrap();
		ping_streamer.write(&[c.0 as u8, c.1 as u8, c.2 as u8]).unwrap();
	}
	ping_streamer.finish().expect("PNG write failed");
	ping_writer.finish().expect("PNG write failed");
}
