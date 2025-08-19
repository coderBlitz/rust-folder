//! FIT file protocol parser
//!
//! As defined [here](https://developer.garmin.com/fit/protocol/).

use std::{
	convert::AsRef,
	fmt,
	fs::File,
	io::{self, Read},
	path::Path,
};

#[derive(Clone, Copy, Debug)]
pub struct FitHeader {
	ver: u8,
	profile_ver: u16,
	data_size: u32,
	crc: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordHdrType {
	Normal,
	Cts, // Compressed timestamp
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MsgType {
	Data,
	Definition,
}

pub struct RecordHdr(u8);
impl RecordHdr {
	pub fn hdr_type(&self) -> RecordHdrType {
		if (self.0 & 0x80) == 0 {
			RecordHdrType::Normal
		} else {
			RecordHdrType::Cts
		}
	}

	pub fn msg_type(&self) -> MsgType {
		// If compressed timestamp or normal with data type, then data.
		if (self.0 & 0x80) != 0 || (self.0 & 0x40) == 0 {
			MsgType::Data
		} else {
			MsgType::Definition
		}
	}

	/// Gets the local message type associated with this record.
	///
	/// For normal messages this value is in the range `[0, 15]`, and for
	///  compressed timestamp messages the range is `[0, 3]`.
	pub fn local_msg_type(&self) -> u8 {
		match self.hdr_type() {
			RecordHdrType::Normal => self.0 & 0xF,
			RecordHdrType::Cts => (self.0 & 0x60) >> 5,
		}
	}

	/// Get the relative time offset (in seconds) from this compressed timestamp.
	///
	/// Only applicable to compressed timestamp records. Range is `[0, 31]`.
	pub fn time_offset(&self) -> u8 {
		match self.hdr_type() {
			RecordHdrType::Normal => 0,
			RecordHdrType::Cts => self.0 & 0x1F,
		}
	}
}
impl fmt::Debug for RecordHdr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut st = f.debug_struct("RecordHdr");
		let l = self.local_msg_type();
		match self.hdr_type() {
			RecordHdrType::Normal => {
				let m = match self.msg_type() {
					MsgType::Definition => "Definition",
					MsgType::Data => "Data",
				};
				st.field("type", &format_args!("Normal"));
				st.field("msg_type", &format_args!("{}", m));
				st.field("local_msg_type", &format_args!("{}", l))
			}
			RecordHdrType::Cts => {
				let t = self.time_offset();
				st.field("type", &format_args!("Cts"));
				st.field("local_msg_type", &format_args!("{}", l));
				st.field("offset", &format_args!("{}", t))
			}
		}
		.finish()
	}
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

pub struct FitFile {
	fp: File,
	hdr: FitHeader,
}
impl FitFile {
	pub fn parse_record(&mut self) -> io::Result<()> {
		let mut scratch = [0; 1];
		_ = self.fp.read(&mut scratch);
		let hdr = RecordHdr(scratch[0]);

		println!("Record hdr: {hdr:?}");

		if let MsgType::Definition = hdr.msg_type() {
			let def = self.parse_definition()?;
			println!("Definition: {def:?}");
		};

		Ok(())
	}

	fn parse_definition(&mut self) -> io::Result<MsgDef> {
		let mut fixed = [0; 5];
		_ = self.fp.read(&mut fixed)?;

		let arch = fixed[1];
		let glob_num = u16::from_le_bytes((&fixed[2..4]).try_into().unwrap());
		let num_fields = fixed[4];

		let mut fields = Vec::new();
		for _ in 0..num_fields {
			_ = self.fp.read(&mut fixed[..3])?;
			fields.push(FieldDef {
				field_num: fixed[0],
				size: fixed[1],
				base: fixed[2],
			});
		}

		Ok(MsgDef {
			arch,
			glob_num,
			num_fields,
			fields,
		})
	}
}

#[derive(Clone, Debug)]
struct MsgDef {
	arch: u8,
	glob_num: u16,
	num_fields: u8,
	fields: Vec<FieldDef>,
}

#[derive(Clone, Copy, Debug)]
struct FieldDef {
	field_num: u8,
	size: u8,
	base: u8,
}

pub fn open<P: AsRef<Path>>(p: P) -> io::Result<FitFile> {
	fn inner(path: &Path) -> io::Result<FitFile> {
		let mut fp = File::open(path)?;
		let mut buf = Vec::new();
		buf.push(0);
		_ = fp.read(&mut buf[..1])?; // Read size byte

		buf.resize(buf[0] as usize, 0);

		_ = fp.read(&mut buf[1..])?;

		let hdr = parse_hdr(&buf)?;
		println!("Hdr: {hdr:#x?}");

		Ok(FitFile { fp, hdr })
	}
	inner(p.as_ref())
}
