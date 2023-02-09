/*
My attempt to remake the "projectile_motion" program I wrote many years ago.
First attempt used cartesian coordinates, and work fine for your simple physics
 problems. Though I commeted saying orbit was possible, as far as I can tell,
 this was only in theory (gravity + surface drop calculations exist).

This attempt will use polar coordinates (angle may be some linear distance,
 relative to a pre-determined circumference).
Projectile will have a position, velocity, and acceleration for both its
 angular position, and radial distance. Angular acceleration will likely remain
 a constant 0, and radial acceleration will be gravity (altitude-adjusted).
*/

use std::env;
use std::time;

fn main() {
	let argv: Vec<String> = env::args().collect();

	const DT: time::Duration = time::Duration::from_millis(5);
	const G: f64 = 6.67430e-11;
	//const G_E: f64 = 9.80665; // Standard gravity
	const M_E: f64 = 5.972e24;
	//const R_E_SMAJ: f64 = 6_378_137.0; // Earth semi-major (GRS 80) in meters
	//const R_E_SMIN: f64 = 6_356_752.314140; // Earth semi-minor (GRS 80) in meters
	const R_E_AVG: f64 = 6_371_008.7714; // IUGG mean radius in meters
	const GROUND_RAD: f64 = R_E_AVG;

	let (mut ang_p, mut ang_v, ang_a) = (0.0_f64, 0.0001_f64, 0.0_f64); // Angular pose (radians)
	let (mut rad_p, mut rad_v, mut rad_a) = (1.5e6_f64, 0.0_f64, 0.0_f64); // Radius relative to ground (meters)
	//let tot_v = (rad_p * rad_p * ang_v * ang_v + rad_v * rad_v).sqrt(); // (radians times radius for m/s velocity)

	let mut t = time::Duration::from_secs(0);
	let t_max = time::Duration::from_secs(argv.get(1).unwrap_or(&String::new()).parse().unwrap_or(300));

	//println!("Total steps = {}", t_max.as_secs_f64() / DT.as_secs_f64());

	while t < t_max {
		t += DT; // Step time
		let dt = DT.as_secs_f64();
		let r = rad_p + GROUND_RAD;
		let lin_ang_v = r * ang_v;
		//let tot_v = (rad_v * rad_v + lin_ang_v * lin_ang_v).sqrt();

		//println!("r = {r}");

		// Kinematic updates
		// TODO: Consider redoing. May need to use linear parameters to find polar (or to help understand what polar should be)
		// TODO: Verify radial distance calc is correct
		// rad_p is position from angular velocity (in linear direction)
		rad_p = (rad_p * rad_p + dt*dt * lin_ang_v * lin_ang_v).sqrt() + dt * (rad_v + dt * rad_a / 2.0);
		rad_v += dt * rad_a;
		rad_a = -G * M_E / r.powi(2); // Gravity scaled by radial distance (plus ground)

		ang_p += dt * (ang_v + dt * ang_a / 2.0);
		ang_v += dt * ang_a;
		//ang_a = ang_a; // TODO: Check whether this should update

		println!("({ang_p}, {ang_v}, {ang_a})");
		println!("({rad_p}, {rad_v}, {rad_a})");

		// Stop if projectile "collides" with ground
		if rad_p <= 0.0 {
			println!("Ground at t = {}!", t.as_secs_f64());
			break;
		}
	}
}
