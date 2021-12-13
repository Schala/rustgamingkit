use byteorder::{
	LE,
	ReadBytesExt
};

use std::io::Result;

use rgk_core::rtag4;

pub const MAGIC: u32 = rtag4!(b"lzss");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub dcmp_size: u32,
	pub unknown_8: u32,
}

impl Header {
	#[cfg(feature = "import")]
	pub fn read<R>(buf: &mut R) -> Result<Header>
	where
		R: ReadBytesExt,
	{
		let magic = buf.read_u32::<LE>()?;

	}
}
