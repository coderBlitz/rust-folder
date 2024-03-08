/* Notes
TODO: Make integer parsing trait (next_u8, next_u16, etc.) on byte slice for easier parsing.
*/

use std::{
	fs::File,
	io::{self, Read},
	path::Path,
};

//const H5MAGIC: u64 = 0x89484446_0D0A1A0A; // "\211HDF\r\n\032\n"
const H5MAGIC: u64 = 0x0A1A0A0D_46444889; // "\211HDF\r\n\032\n"

#[derive(Debug, Default)]
struct H5Super0 {
	magic: u64,
	version: u8,
	freespace_version: u8, // Only valid value is 0
	sym_version: u8, // Only valid value is 0
	shm_version: u8, // Only valid value is 0
	offset_size: u8,
	length_size: u8,
	leaf_k: u16,
	internal_k: u16,
	_flags: u32, // Unused, should be 0
	indexed_internal_k: u16,
	base_address: u64, // Size specified in "size of offsets" (`offsets_size` above).
	freespace_address: u64, // "", not persistent, always undefined (0xFFFF...)
	eof_address: u64, // ""
	dib_address: u64, // ""
}

#[derive(Debug, Default)]
struct H5Super2 {
	magic: u64,
	version: u8,
	offsets_size: u8,
	lengths_size: u8,
	consistency_flags: u8,
	base_address: u64, // Size specified in "size of offsets" (`offsets_size` above).
	extension_address: u64, // ""
	eof_address: u64, // ""
	root_goh_address: u64, // ""
	checksum: u32,
}

#[derive(Debug)]
enum H5Super {
	V0(H5Super0),
	V1(H5Super0),
	V2(H5Super2),
	V3(H5Super2),
}

pub struct H5File {
}
impl H5File {
	pub fn open<P: AsRef<Path>>(p: P) -> io::Result<Self> {
		fn inner(path: &Path) -> io::Result<H5File> {
			let mut f = File::open(path)?;
			let mut buf = [0; 4096];
			let cnt = f.read(&mut buf)?;
			let contents = &buf[..cnt];
			let mut processed_size;

			if contents.len() < 9 {
				return Err(io::ErrorKind::InvalidData.into())
			}
			processed_size = 9;

			let magic = u64::from_le_bytes((&contents[..8]).try_into().unwrap());
			println!("File magic: 0x{magic:X}");
			if magic != H5MAGIC {
				return Err(io::ErrorKind::InvalidData.into())
			}

			let version = contents[8];
			println!("Superblock version: {version}");

			match version {
				v @ (0 | 1) => {
					// Check first 24 bytes (28 if version 1)
					processed_size = (24 + v*4) as usize;
					if contents.len() < processed_size {
						return Err(io::ErrorKind::InvalidData.into())
					}

					/* Get remaining (fixed-size) contents of superblock
					*/
					let fss_v = contents[9];
					let sym_v = contents[10];
					let shm_v = contents[12];
					let off_size = contents[13] as usize;
					let len_size = contents[14];
					let leaf_k = u16::from_le_bytes((&contents[16..18]).try_into().unwrap());
					let internal_k = u16::from_le_bytes((&contents[18..20]).try_into().unwrap());
					let fcf = u32::from_le_bytes((&contents[20..24]).try_into().unwrap());
					println!("Free-space version: {fss_v}");
					println!("Root group sym table version: {sym_v}");
					println!("Shared header version: {shm_v}");
					println!("Size of offsets: {off_size}");
					println!("Size of Lenghts: {len_size}");
					println!("Group leaf k: {leaf_k}");
					println!("Group internal k: {internal_k}");
					println!("File consistency flags: {fcf}");

					if v == 1 {
						let idx_k = u16::from_le_bytes((&contents[24..26]).try_into().unwrap());
						println!("Indexed internal k: {idx_k}");
					}

					// Size check for remainder of superblock (addresses + root symbol table entry)
					processed_size += 6 * off_size + 24;
					if contents.len() < processed_size {
						return Err(io::ErrorKind::InvalidData.into())
					}

					// Get the various addresses
					let remainder = (24 + v*4) as usize;
					let mut scratch = [0; 8];
					scratch[0..off_size].copy_from_slice(&contents[remainder .. (remainder+off_size)]);
					let base = u64::from_le_bytes(scratch);
					scratch[0..off_size].copy_from_slice(&contents[(remainder + off_size) .. (remainder + 2*off_size)]);
					let freespace = u64::from_le_bytes(scratch);
					scratch[0..off_size].copy_from_slice(&contents[(remainder + 2*off_size) .. (remainder + 3*off_size)]);
					let eof = u64::from_le_bytes(scratch);
					scratch[0..off_size].copy_from_slice(&contents[(remainder + 3*off_size) .. (remainder + 4*off_size)]);
					let driver = u64::from_le_bytes(scratch);

					println!("Base address: 0x{base:X} ({base})");
					println!("Freespace address: 0x{freespace:X} ({freespace})");
					println!("EOF address: 0x{eof:X} ({eof})");
					println!("Driver address: 0x{driver:X} ({driver})");

					// Get root symbol table entry
					scratch[0..off_size].copy_from_slice(&contents[(remainder + 4*off_size) .. (remainder + 5*off_size)]);
					let link_name_off = u64::from_le_bytes(scratch);
					scratch[0..off_size].copy_from_slice(&contents[(remainder + 5*off_size) .. (remainder + 6*off_size)]);
					let obj_header = u64::from_le_bytes(scratch);

					let remainder = remainder + 6 * off_size;

					let cache_type = u32::from_le_bytes((&contents[remainder .. (remainder + 4)]).try_into().unwrap());
					let scratch_pad = &contents[(remainder + 8) .. (remainder + 24)];

					println!("Link name offset: 0x{link_name_off:X} ({link_name_off})");
					println!("Object header address: 0x{obj_header:X} ({obj_header})");
					println!("Cache type: 0x{cache_type:X} ({cache_type})");
					println!("Scratch pad: {scratch_pad:X?}");

					/* Get file data
					*/
					let mut block_start = processed_size;
					//loop {
						let sig = &contents[block_start .. (block_start + 4)];
						println!("Next sig: {sig:?} ({})", std::str::from_utf8(sig).unwrap());

						assert_eq!(std::str::from_utf8(sig).unwrap(), "OHDR");
						let obj_v = contents[block_start + 4];
						let flags = contents[block_start + 5];
					//}
				},
				2 | 3 => {
				},
				_ => return Err(io::ErrorKind::InvalidData.into()),
			};

			Ok(H5File {})
		}
		inner(p.as_ref())
	}
}
