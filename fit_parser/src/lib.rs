//! FIT file protocol parser
//!
//! As defined [here](https://developer.garmin.com/fit/protocol/).

use std::io;

#[derive(Clone, Copy, Debug)]
pub struct FitHeader {
	ver: u8,
	profile_ver: u16,
	data_size: u32,
	crc: u16,
}

pub fn parse_hdr(bytes: &[u8]) -> io::Result<FitHeader> {
	if bytes.len() != 12 && bytes.len() != 14 {
		return Err(io::ErrorKind::UnexpectedEof.into());
	}

	let ver = bytes[1];
	let profile_ver = u16::from_le_bytes((&bytes[2..4]).try_into().unwrap());
	let data_size = u32::from_le_bytes((&bytes[4..8]).try_into().unwrap());
	let data_type = &bytes[8..12];

	if data_type != b".FIT" {
		return Err(io::ErrorKind::InvalidData.into());
	}

	let crc = match bytes.len() {
		14 => u16::from_le_bytes((&bytes[12..14]).try_into().unwrap()),
		_ => 0,
	};

	Ok(FitHeader {
		ver,
		profile_ver,
		data_size,
		crc,
	})
}
