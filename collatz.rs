fn main() {
	const N: i64 = 10;
	println!("Computing up to {N}");

	for i in 2..=N {
		let mut j = i;
		let mut count = 0;
		while j != 1 {
			if (j % 2) == 0 {
				j /= 2;
			}else{
				j = 3*j + 1;
			}

			count += 1;
		}

		println!("{i}: {count}");
	}
} 
