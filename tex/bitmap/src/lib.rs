use byteorder::{
	BE,
	LE,
	ReadBytesExt,
	WriteBytesExt
};

use std::io;
use thiserror::Error;

use rgk_core::{
	tag2,
	texture::Texture
};

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

impl Magic {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Magic, BitmapImportError>
	where
		R: ReadBytesExt,
	{
		let magic = buf.read_u16::<BE>()?;
		match magic {
			0x4241 => Ok(Magic::OS2BitmapArray),
			0x424D => Ok(Magic::Windows),
			0x4349 => Ok(Magic::OS2ColorIcon),
			0x4350 => Ok(Magic::OS2ColorPointer),
			0x4943 => Ok(Magic::OS2Icon),
			0x5054 => Ok(Magic::OS2Pointer),
			_ => Err(BitmapImportError::Magic(magic)),
		}
	}

	#[cfg(feature = "export")]
	fn write<W>(self, buf: &mut W) -> io::Result<()>
	where
		W: WriteBytesExt,
	{
		buf.write_u16::<BE>(self as u16)
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: Magic,
	pub size: u32,
	reserved6: u16, // safe to be 0
	reserved8: u16, // safe to be 0
	pub pixel_offset: u32,
}

impl Header {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Header, BitmapImportError>
	where
		R: ReadBytesExt,
	{
		Ok(Header {
			magic: Magic::read(buf)?,
			size: buf.read_u32::<LE>()?,
			reserved6: buf.read_u16::<LE>()?,
			reserved8: buf.read_u16::<LE>()?,
			pixel_offset: buf.read_u32::<LE>()?,
		})
	}

	#[cfg(feature = "export")]
	fn write<W>(&self, buf: &mut W) -> io::Result<()>
	where
		W: WriteBytesExt,
	{
		self.magic.write(buf)?;
		buf.write_u32::<LE>(self.size)?;
		buf.write_u16::<LE>(self.reserved6)?;
		buf.write_u16::<LE>(self.reserved8)?;
		buf.write_u32::<LE>(self.pixel_offset)
	}
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

impl Compression {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Compression, BitmapImportError>
	where
		R: ReadBytesExt,
	{
		let cmp = buf.read_u32::<LE>()?;
		match cmp {
			0 => Ok(Compression::RGB),
			1 => Ok(Compression::RLE8),
			2 => Ok(Compression::RLE4),
			3 => Ok(Compression::Huffman1D),
			4 => Ok(Compression::JPEG),
			5 => Ok(Compression::PNG),
			6 => Ok(Compression::AlphaBitfields),
			7 => Ok(Compression::CMYK),
			8 => Ok(Compression::CMYKRLE8),
			9 => Ok(Compression::CMYKRLE4),
			_ => Err(BitmapImportError::Compression(cmp)),
		}
	}

	#[cfg(feature = "export")]
	fn write<W>(self, buf: &mut W) -> io::Result<()>
	where
		W: WriteBytesExt,
	{
		buf.write_u32::<LE>(self as u32)
	}
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
	pub num_colors_used: u32, // defaults to 0 for power of 2
	pub num_important_colors: u32, // ignored, should be 0
}

impl InfoHeader {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<InfoHeader, BitmapImportError>
	where
		R: ReadBytesExt,
	{
		let header_size = buf.read_u32::<LE>()?;
		if header_size != HEADER_SIZE {
			return Err(BitmapImportError::HeaderSize(header_size));
		}

		Ok(InfoHeader {
			header_size: header_size,
			width: buf.read_i32::<LE>()?,
			height: buf.read_i32::<LE>()?,
			num_color_panes: buf.read_u16::<LE>()?,
			bpp: buf.read_u16::<LE>()?,
			compression: Compression::read(buf)?,
			img_size: buf.read_u32::<LE>()?,
			h_res: buf.read_i32::<LE>()?,
			v_res: buf.read_i32::<LE>()?,
			num_colors_used: buf.read_u32::<LE>()?,
			num_important_colors: buf.read_u32::<LE>()?,
		})
	}

	#[cfg(feature = "export")]
	fn write<W>(&self, buf: &mut W) -> io::Result<()>
	where
		W: WriteBytesExt,
	{
		buf.write_u32::<LE>(self.header_size)?;
		buf.write_i32::<LE>(self.width)?;
		buf.write_i32::<LE>(self.height)?;
		buf.write_u16::<LE>(self.num_color_panes)?;
		buf.write_u16::<LE>(self.bpp)?;
		self.compression.write(buf)?;
		buf.write_u32::<LE>(self.img_size)?;
		buf.write_i32::<LE>(self.h_res)?;
		buf.write_i32::<LE>(self.v_res)?;
		buf.write_u32::<LE>(self.num_colors_used)?;
		buf.write_u32::<LE>(self.num_important_colors)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bitmap {
	pub header: Header,
	pub info_header: InfoHeader,
	pub pixels: Vec<u32>,
}

impl Bitmap {
	#[cfg(feature = "import")]
	pub fn read<R>(buf: &mut R) -> Result<Bitmap, BitmapImportError>
	where
		R: ReadBytesExt,
	{
		let header = Header::read(buf)?;
		let info_h = InfoHeader::read(buf)?;

		Ok(Bitmap {
			header: header,
			info_header: info_h,
			pixels: vec![],
		})
	}

	/// Creates a new [`Bitmap`] object with a bit depth of 24
	#[cfg(feature = "export")]
	pub fn new_24bpp(tex: &Texture) -> Bitmap {
		let img_size = (((tex.width * 3) + 1) * tex.height) as u32;

		Bitmap {
			header: Header {
				magic: Magic::Windows,
				size: img_size + 54,
				reserved6: 0,
				reserved8: 0,
				pixel_offset: 54,
			},
			info_header: InfoHeader {
				header_size: HEADER_SIZE,
				width: tex.width as i32,
				height: tex.height as i32,
				num_color_panes: 1,
				bpp: 24,
				compression: Compression::RGB,
				img_size: img_size,

				// not sure how to handle these yet
				h_res: 0,
				v_res: 0,

				num_colors_used: 0,
				num_important_colors: 0,
			},
			pixels: tex.pixels().iter().map(|c| c.to_rgb888()).collect(),
		}
	}

	#[cfg(feature = "export")]
	fn write<W>(&self, buf: &mut W) -> io::Result<()>
	where
		W: WriteBytesExt,
	{
		self.header.write(buf)?;
		self.info_header.write(buf)?;

		for y in 0..(self.info_header.height as usize) {
			for x in 0..(self.info_header.width as usize) {
				buf.write_u24::<LE>(self.pixels[(y * (self.info_header.width as usize)) + x])?;
			}
			buf.write_u8(0)?;
		}

		Ok(())
	}
}

#[cfg(feature = "import")]
#[derive(Error, Debug)]
pub enum BitmapImportError {
	#[error("Unknown/unsupported compression method: {0}")]
	Compression(u32),
	#[error("Header size mismatch: expected 40, got {0}")]
	HeaderSize(u32),
	#[error("I/O error")]
	IO {
		#[from]
		source: io::Error,
	},
	#[error("Not a BMP file: {0:X}")]
	Magic(u16),
}

#[test]
fn test_read_bmp() {
	use std::fs::read;

	let data = read("test_data/Screenshot_20211119_104703.bmp").unwrap();
	println!("{:#?}", Bitmap::read(&mut data.as_slice()).unwrap());
}

#[cfg(feature = "export")]
#[test]
fn test_write_bmp() {
	use rgk_textures_playstation::read_tim;
	use std::fs::File;

	let tim = read_tim("test_data/3DFX.TIM").unwrap();
	let bmp = Bitmap::new_24bpp(&tim);
	println!("{:#?}", &bmp);
	let mut f = File::create("test_data/3DFX.bmp").unwrap();
	bmp.write(&mut f).unwrap();
}
