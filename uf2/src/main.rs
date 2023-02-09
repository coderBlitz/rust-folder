use std::fs;
use std::io::Write;
use uf2::create_uf2;

fn main() {
	let mut data: [u8; 256] = [0;256];
	// 2 bytes data
	data[0] = 0xFE;
	data[1] = 0xE7;
	// Checksum (data + all zeros)
	data[252] = 0x2A;
	data[253] = 0x25;
	data[254] = 0x7E;
	data[255] = 0xF4;
	let offset = 0x1000_0000;

	let out = create_uf2(offset, &data);

	let f = &mut fs::File::create("/tmp/out.uf2").expect("Could not open file to write");
	f.write(&out).expect("Write failed");
}
