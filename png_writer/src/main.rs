//! Write out a PNG
//!
//! TODO: Add CRC32 code

const PNG_SIG: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

struct PngIHDR {
	width: u32,
	height: u32,
	depth: u8,
	color: PngColor,
	compression: PngCompression,
	filter: PngFilter,
	interlace: PngInterlace,
	crc: u32,
}

/// Color palette. Contains, at most, 256 entries. Each entry is a 3-byte tuple (RGB).
struct PngPLTE {
	palette: Vec<[u8; 3]>
}

struct PngChunk {
	length: u32,
	_type: u32,
	crc: u32,
	data: Vec<u8>
}

struct ZlibStream {
	compression: ZlibCMF,
	flags: ZlibFLG,
	data: Vec<u8>,
	adler: u32,
}

/// ZLIB "Compression Method and flags" (CMF) field.
///
/// Method is always 0x8, and info is (log_2(win_size) - 8). Max window size is
///  32K, min is 256.
struct ZlibCMF {
	method: ZlibCM,
	info: u8,
}
struct ZlibFLG {
	check: u8,
	dict: u8,
	level: ZlibLevel,
}
enum ZlibCM {
	Deflate,
}
enum ZlibLevel {
	Fastest,
	Fast,
	Default,
	Slowest,
}
	
enum PngColor {
	Indexed,
	Grayscale,
	GrayscaleAlpha,
	Truecolor,
	TruecolorAlpha,
}
enum PngInterlace {
	None,
	Adam7,
}
enum PngFilter {
	None,
}
enum PngCompression {
	Deflate,
}

fn main() {
    println!("Hello, world!");
}
