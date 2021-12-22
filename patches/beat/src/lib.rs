use byteorder::{
	NativeEndian,
	ReadBytesExt,
	WriteBytesExt
};

use crc32fast::hash;

use std::io::{
	Seek,
	SeekFrom,
	self
};

#[cfg(feature = "import")]
use thiserror::Error;

use rgk_core::tag4;

type NE = NativeEndian;

pub const MAGIC: u32 = tag4!(b"BPS1");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub source_size: u64,
	pub target_size: u64,
	pub metadata_size: u64,
}

impl Header {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Header, BeatImportError>
	where
		R: ReadBytesExt,
	{
		let magic = buf::read_u32::<NE>()?;
		if magic != MAGIC {
			return Err(BeatImportError::Magic(magic));
		}

		Ok(Header {
			magic: magic,
			source_size: decode(buf)?,
			target_size: decode(buf)?,
			metadata_size: decode(buf)?,
		})
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Mode {
	SourceRead = 0,
	TargetRead,
	SourceCopy,
	TargetCopy,
}

impl Mode {
	#[cfg(feature = "import")]
	fn read(input: u64) -> Mode {
		match input & 3 {
			0 => Mode::SourceRead,
			1 => Mode::TargetRead,
			2 => Mode::SourceCopy,
			3 => Mode::TargetCopy,
			_ => unreachable!(),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Record {
	pub mode: Mode,
	pub length: u64,
	pub offset: Option<u64>,
}

impl Record {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Record, BeatImportError>
	where
		R: ReadBytesExt,
	{
		let length = decode(buf)?;
		let mode: Mode::read(length);

		Ok(Record {
			mode: mode,
			length: length,
			offset: if mode != Mode::SourceRead && mode != Mode::TargetRead {
				Some(decode())
			} else {
				None
			},
		})
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeatPatch {
	pub header: Header,
	pub manifest: Option<String>,
	pub records: Vec<Record>,
	pub source_hash: u32,
	pub target_hash: u32,
	pub patch_hash: u32,
}

impl BeatPatch {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<BeatPatch, BeatImportError>
	where
		R: ReadBytesExt + Seek,
	{
		let header = Header::read(buf)?;

		let pos = buf.stream_position()?;
		let size = buf.seek(SeekFrom::End(0))?;
		let _ = buf.seek(SeekFrom::Start(pos))?;
		let mut recs = vec![];

		while buf.stream_position()? < ((size - 12) as u64) {

		}
	}
}

#[cfg(feature = "import")]
#[derive(Debug, Error)]
pub enum BeatImportError {
	#[error("I/O error")]
	IO {
		#[from]
		source: io::Error,
	},
	#[error("Not a Beat patch: {X:0}")]
	Magic(u32),
}

#[cfg(feature = "import")]
fn decode<R>(buf: &mut R) -> Result<u64, BeatImportError>
where
	R: ReadBytesExt,
{
	let mut data = 0;
	let mut shift = 1;

	loop {
		let x = buf.read_u8()?;
		data += ((x & 127) as u64) * shift;
		if (x & 128) != 0 {
			break;
		}
		shift <<= 7;
		data += shift;
	}

	Ok(data)
}

#[cfg(feature = "export")]
fn encode<W>(data: u64, buf: &mut W) -> io::Result<()>
where
	W: WriteBytesExt,
{
	let mut data = data;

	loop {
		let x = (data & 127) as u8;
		data >>= 7;
		if data == 0 {
			buf.write_u8(128 | x)?;
			break;
		}

		buf.write_u8(x)?;
		data -= 1;
	}

	Ok(())
}
