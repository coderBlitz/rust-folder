use core::slice;
use std::ops::{Add, Sub, Mul, Div};
use rand::{self, distributions::uniform::*};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3(pub f64, pub f64, pub f64);

impl Add for Vec3 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self (self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
	}
}
impl Add<f64> for Vec3 {
	type Output = Self;

	fn add(self, rhs: f64) -> Self {
		Self (self.0 + rhs, self.1 + rhs, self.2 + rhs)
	}
}

impl Sub for Vec3 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self (self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
	}
}
impl Sub<f64> for Vec3 {
	type Output = Self;

	fn sub(self, rhs: f64) -> Self {
		Self (self.0 - rhs, self.1 - rhs, self.2 - rhs)
	}
}

impl Mul for Vec3 {
	type Output = f64;

	fn mul(self, rhs: Self) -> f64 {
		(self.0 * rhs.0) + (self.1 * rhs.1) + (self.2 * rhs.2)
	}
}
impl Mul<f64> for Vec3 {
	type Output = Vec3;

	fn mul(self, rhs: f64) -> Self {
		Self (self.0 * rhs, self.1 * rhs, self.2 * rhs)
	}
}

impl Div<f64> for Vec3 {
	type Output = Self;

	fn div(self, rhs: f64) -> Self {
		Self (self.0 / rhs, self.1 / rhs, self.2 / rhs)
	}
}

impl rand::Fill for Vec3 {
	fn try_fill<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), rand::Error> {
		let slice = unsafe {
			slice::from_raw_parts_mut(std::ptr::from_mut(self) as *mut u8, std::mem::size_of::<Self>())
		};

		rng.try_fill_bytes(slice)
	}
}
impl rand::distributions::Distribution<Vec3> for rand::distributions::Standard {
	fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		Vec3(self.sample(rng), self.sample(rng), self.sample(rng))
	}
}
pub struct VecF64Sampler(UniformFloat<f64>, UniformFloat<f64>, UniformFloat<f64>);
impl UniformSampler for VecF64Sampler {
	type X = Vec3;

	fn new<B1, B2>(low: B1, high: B2) -> Self
		where
			B1: SampleBorrow<Self::X> + Sized,
			B2: SampleBorrow<Self::X> + Sized {
		Self(
			UniformFloat::new(low.borrow().0, high.borrow().0),
			UniformFloat::new(low.borrow().1, high.borrow().1),
			UniformFloat::new(low.borrow().2, high.borrow().2),
		)
	}
	fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
		where
			B1: SampleBorrow<Self::X> + Sized,
			B2: SampleBorrow<Self::X> + Sized {
		Self(
			UniformFloat::new_inclusive(low.borrow().0, high.borrow().0),
			UniformFloat::new_inclusive(low.borrow().1, high.borrow().1),
			UniformFloat::new_inclusive(low.borrow().2, high.borrow().2),
		)
	}
	fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
		Vec3(
			self.0.sample(rng),
			self.1.sample(rng),
			self.2.sample(rng),
		)
	}
}
impl rand::distributions::uniform::SampleUniform for Vec3 {
	type Sampler = VecF64Sampler;
}

impl Vec3 {
	pub fn rand<R: rand::Rng>(gen: &mut R) -> Self {
		Self(gen.gen(), gen.gen(), gen.gen())
	}

	pub fn norm(&self) -> f64 {
		(self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
	}

	pub fn cross(&self, rhs: Vec3) -> Self {
		Self (self.1 * rhs.2 - self.2 * rhs.1, self.2 * rhs.0 - self.0 * rhs.2, self.0 * rhs.1 - self.1 * rhs.0)
	}
}
