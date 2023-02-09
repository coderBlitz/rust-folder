use std::fs::File;
use std::io::Read;
use std::env;

fn main() {
	let argv: Vec<String> = env::args().collect();

	let mut filename = "large.csv";
	if argv.len() > 1 {
		filename = &argv[1];
	}
	let filename = filename;

	let mut fp = match File::open(filename) {
		Ok(f) => f,
		Err(e) => panic!("Error! {:?}", e.kind()),
	};

	let mut line_v = Vec::<u8>::new();
	let mut buf = [0;2048]; // 2048 seems to be better than 1024 or 4096
	let mut line_ct = 0;
	loop {
		let _res = match fp.read(&mut buf){
			Ok(n) => {
				if n == 0 { break }
				n
			},
			Err(_) => break,
		};

		// Iterate over buffer, building lines as we go
		let mut front = 0;
		for i in 0.._res {
			let v = buf[i];

			if v == b'\n' {
				// Append preceding parts of buffer to line
				line_v.extend_from_slice(&buf[front..i]);

				let line = std::str::from_utf8(&line_v).unwrap();

				//println!("Line {line_ct:2}: {line}");
				let fields = line.split(',').collect::<Vec<&str>>(); // Simple CSV split parse
				//println!("Fields: {fields:?}");

				line_ct += 1;
				line_v.clear();

				front = i + 1;
			}
		}

		// Append remaining buffer, if any
		if front < _res {
			line_v.extend_from_slice(&buf[front.._res]);
		}

		//println!("n = {_res}");
	}

	println!("{line_ct} lines read");
}
