use rgk_core::tag4;

pub const MAGIC: u32 = tag4!(b"TIM2");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub version: u16,
	pub num_images: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Depth {
	BPP16 = 1,
	BPP24,
	BPP32
	BPP4,
	BPP8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ImageFormat {
	BPP8Paletted = 0,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageHeader {
	pub total_size: u32,
	pub palette_size: u32,
	pub data_size: u32,
	pub header_size: u16,
	pub num_colors: u16,
	pub format: ImageFormat,
	pub num_mipmaps: u8,
	pub clut_format: u8,
	pub bpp: Depth,
	pub width: u16,
	pub height: u16,

}
