/// Converts a 2-byte string into a 16-bit big endian integer.
/// Byte strings longer than 2 bytes are truncated.
macro_rules! tag2 {
	($b2: literal) => {
		u16::from_be_bytes([$b2[0], $b2[1]])
	}
}

pub const HEADER_SIZE: u32 = 40;
pub const NUM_COLOR_PANES: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum Magic {
	OS2BitmapArray = tag2!(b"BA"),
	Windows = tag2!(b"BM"),
	OS2ColorIcon = tag2!(b"CI"),
	OS2ColorPointer = tag2!(b"CP"),
	OS2Icon = tag2!(b"IC"),
	OS2Pointer = tag2!(b"PT"),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: Magic,
	pub size: u32,
	reserved6: u16, // safe to be 0
	reserved8: u16, // safe to be 0
	pub pixel_offset: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Compression {
	RGB = 0,
	RLE8,
	RLE4,
	Huffman1D,
	JPEG,
	PNG,
	AlphaBitfields,
	CMYK,
	CMYKRLE8,
	CMYKRLE4,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InfoHeader {
	pub header_size: u32, // should be 40
	pub width: i32,
	pub height: i32,
	pub num_color_panes: u16, // should be 1
	pub bpp: u16,
	pub compression: Compression,
	pub img_size: u32,
	pub h_res: i32,
	pub v_res: i32,
	pub num_colors: u32, // defaults to 0 for power of 2
	pub num_used_colors: u32, // ignored, should be 0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bitmap {
	pub header: Header,
	pub info_header: InfoHeader,
	pub palette: Vec<u32>,
}
