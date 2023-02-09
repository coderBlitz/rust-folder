use std::vec::Vec;

pub fn create_uf2(offset: u32, data: &[u8]) -> Vec<u8> {
	const PAYLOAD_SIZE: u32 = 256;
	const PAYLOAD_BYTES: [u8; 4] = PAYLOAD_SIZE.to_le_bytes();
	const MAGIC1: [u8; 4] = [0x55, 0x46, 0x32, 0x0A];
	const MAGIC2: [u8; 4] = [0x57, 0x51, 0x5D, 0x9E];
	const MAGIC3: [u8; 4] = [0x30, 0x6F, 0xB1, 0x0A];
	const FLAGS: [u8; 4] = [0x00, 0x20, 0x00, 0x00]; // familyID present flag
	const FAMILYID: [u8; 4] = [0x56, 0xFF, 0x8B, 0xE4]; // Pi pico family
	const PADDING: [u8; 220] = [0; 220];

	let num_blocks = (data.len() as u32) / PAYLOAD_SIZE;
	let nblocks: [u8; 4] = num_blocks.to_le_bytes();

	// Scratch vector per loop
	let block = &mut Vec::new();
	block.reserve_exact(512);

	// Output vector
	let mut out = Vec::new();
	out.reserve_exact((num_blocks * 512) as usize);

	for i in 0..num_blocks {
		// Offset of this particular block (assuming all data is contiguous)
		let block_offset = offset + i * PAYLOAD_SIZE;

		// Add header info
		// TODO: Move as much out as possible, and use copy_from_slice inside loop
		block.extend_from_slice(&MAGIC1);
		block.extend_from_slice(&MAGIC2);
		block.extend_from_slice(&FLAGS);
		block.extend_from_slice(&block_offset.to_le_bytes());
		block.extend_from_slice(&PAYLOAD_BYTES);
		block.extend_from_slice(&(i as u32).to_le_bytes());
		block.extend_from_slice(&nblocks);
		block.extend_from_slice(&FAMILYID);
		assert_eq!(block.len(), 32);

		// Add data
		block.extend_from_slice(&data[(256*i as usize) .. (256*(i+1) as usize)]);

		// Pad and add final magic
		block.extend_from_slice(&PADDING);
		block.extend_from_slice(&MAGIC3);

		// End of loop stuff
		assert_eq!(block.len(), 512);
		out.extend_from_slice(&block);
		block.clear();
		println!("Created block {}", i+1);
	}

	out
}
