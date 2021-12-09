use rgk_core::{
	rtag4,
	tag4
};

pub const MAGICS: [u32; 2] = [tag4!(b"CHN2"), rtag4!(b"CHMP")];

pub struct Header {
	pub magic: [u32; 2],
	pub unknown08: u32,
	pub unknown0c: u32,
	pub size: u32,
	padding: [u8; 96],
}
