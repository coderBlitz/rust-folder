fn main() {
	let argv: Vec<String> = std::env::args().collect();
	let val: &str = &String::from("10");
	let val: &str = if argv.len() > 1 {&argv[1]} else {val};

	let N: usize = val.parse().expect("Must give integer for bound.");
	//const N: usize = 10000;

	println!("Computing up to {N}");

	let mut prev = Vec::with_capacity(N);
	unsafe { prev.set_len(N); }
	let mut prev = prev.into_boxed_slice();
	prev[0] = 0;
	prev[1] = 0;
	//let mut prev = vec![0;N];
	//prev.push(0);
	//prev.push(0);

	let mut max = 0;
	let mut max_i = 1;
	for i in 2..=N {
		let mut j = i;
		let mut count = 0;
		while j >= i {
			if (j % 2) == 0 {
				j /= 2;
			}else{
				j = 3*j + 1;
			}

			count += 1;
		}

		count += prev[j-1];
		prev[i-1] = count;
		//prev.push(count);

		if count > max {
			max = count;
			max_i = i;
		}

		//println!("{i}: {count}");
	}

	println!("Longest chain is {max} from {max_i}");
} 
