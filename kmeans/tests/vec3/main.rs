use kmeans::vec3::Vec3;

#[test]
fn cross_prod() {
	let x = Vec3 (1.,0.,0.);
	let y = Vec3 (0.,1.,0.);
	let z = Vec3 (0.,0.,1.);

	assert_eq!(x.cross(y), z);
}
