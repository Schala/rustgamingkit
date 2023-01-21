use bitflags::bitflags;

use byteorder::{
	LE,
	ReadBytesExt
};

use std::io;
use thiserror::Error;

pub const MAGIC: u16 = 16;

bitflags! {
	pub struct Flags: u32 {
		const BPP_4 = 0;
		const BPP_8 = 1;
		const BPP_16 = 2;
		const BPP_24 = 3;
		const MIXED = 4;
		const INDEXED = 8;
	}
}

#[cfg(feature = "import")]
#[derive(Debug, Error)]
pub enum TIMImportError {
	#[error("I/O error")]
	IO {
		#[from]
		source: io::Error,
	},
	#[error("Not a PlayStation texture: {0}")]
	Magic(u16),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u16,
	pub version: u16,
	pub flags: Flags,
}

impl Header {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Header, TIMImportError>
	where
		R: ReadBytesExt,
	{
		let magic = buf.read_u16::<LE>()?;
		if magic != MAGIC {
			return Err(TIMImportError::Magic(magic));
		}

		Ok(Header {
			magic: magic,
			version: buf.read_u16::<LE>()?,
			flags: Flags::from_bits_truncate(buf.read_u32::<LE>()?),
		})
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubHeader {
	pub size: u32,
	pub x: u16,
	pub y: u16,
	pub width: u16,
	pub height: u16,
}

impl SubHeader {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<SubHeader, TIMImportError>
	where
		R: ReadBytesExt,
	{
		Ok(SubHeader {
			size: buf.read_u32::<LE>()?,
			x: buf.read_u16::<LE>()?,
			y: buf.read_u16::<LE>()?,
			width: buf.read_u16::<LE>()?,
			height: buf.read_u16::<LE>()?,
		})
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageData {
	Indexed(Vec<u8>),
	BPP16(Vec<u16>),
	BPP24(Vec<u32>),
}

impl ImageData {
	#[cfg(feature = "import")]
	fn read<'a, 'b, R>(header: &'a Header, img_header: &'a SubHeader, buf: &mut R) -> Result<ImageData, TIMImportError>
	where
		R: ReadBytesExt,
	{
		if header.flags.contains(Flags::INDEXED) {
			let mut indices = vec![];

			for _ in 0..((img_header.width * img_header.height) as usize) {
				indices.push(buf.read_u8()?);
			}

			Ok(ImageData::Indexed(indices))
		} else if header.flags.contains(Flags::BPP_16) {
			let mut rgb = vec![];

			for _ in 0..((img_header.width * img_header.height) as usize) {
				rgb.push(buf.read_u16::<LE>()?);
			}

			Ok(ImageData::BPP16(rgb))
		} else { // 24 BPP
			let mut rgb = vec![];

			for _ in 0..((img_header.width * img_header.height) as usize) {
				rgb.push(buf.read_u24::<LE>()?);
			}

			Ok(ImageData::BPP24(rgb))
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct PSXTexture {
	pub header: Header,
	pub palette_header: Option<SubHeader>,
	pub palette: Option<Vec<u16>>,
	pub img_header: SubHeader,
	pub data: ImageData,
}

impl PSXTexture {
	#[cfg(feature = "import")]
	pub fn read<R>(buf: &mut R) -> Result<PSXTexture, TIMImportError>
	where
		R: ReadBytesExt,
	{
		let header = Header::read(buf)?;

		let clut_header: Option<SubHeader>;
		let mut clut: Option<Vec<u16>>;

		if header.flags.contains(Flags::INDEXED) {
			clut_header = Some(SubHeader::read(buf)?);
			clut = Some(vec![]);

			if let Some(ref mut c) = clut {
				// 8 BPP check must come before 4 BPP because Flags.contains() considers both present
				if header.flags.contains(Flags::BPP_8) {
					for _ in 0..256 {
						c.push(buf.read_u16::<LE>()?);
					}
				} else { // 4 BPP
					for _ in 0..16 {
						c.push(buf.read_u16::<LE>()?);
					}
				}
			}
		} else {
			clut_header = None;
			clut = None;
		}

		let img_header = SubHeader::read(buf)?;
		let img_data = ImageData::read(&header, &img_header, buf)?;

		Ok(PSXTexture {
			header: header,
			palette_header: clut_header,
			palette: clut,
			img_header: img_header,
			data: img_data,
		})
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_tim() {
		let tim = crate::read_tim("test_data/3DFX.TIM");
		println!("{:#?}", tim);
	}
}
