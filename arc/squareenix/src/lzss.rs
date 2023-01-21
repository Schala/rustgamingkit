use bitvec::prelude::*;

use byteorder::{
	ReadBytesExt,
	LE
};

use std::io;
use thiserror::Error;

use rgk_core::{
	bit_ext::vec_pop_n,
	rtag4
};

pub const MAGIC: u32 = rtag4!(b"sszl");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub dcmp_size: u32,
	pub unknown_8: u32,
}

impl Header {
	#[cfg(feature = "import")]
	pub fn read<R>(buf: &mut R) -> Result<Header, LZSSImportError>
	where
		R: ReadBytesExt,
	{
		let magic = buf.read_u32::<LE>()?;
		if magic != MAGIC {
			return Err(LZSSImportError::Magic(magic));
		}

		Ok(Header {
			magic: magic,
			dcmp_size: buf.read_u32::<LE>()?,
			unknown_8: buf.read_u32::<LE>()?,
		})
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct LZSSArchive {
	pub header: Header,
	pub data: Vec<u8>,
}

impl LZSSArchive {
	#[cfg(feature = "import")]
	pub fn read<R>(padding: usize, buf: &mut R) -> Result<LZSSArchive, LZSSImportError>
	where
		R: ReadBytesExt,
	{
		let header = Header::read(buf)?;
		let mut buffer = [0; 512];
		let mut ringbuffer = [0; 4096];
		let mut rb_index = 0;
		let mut slopover_bits = BitVec::new();
		let mut data = vec![];

		while let Ok(()) = buf.read_exact(&mut buffer) {
			let mut vob = buffer.view_bits_mut::<Lsb0>().to_bitvec();
			vob.append(&mut slopover_bits);
			println!("{}", vob.len());

			while let Some(is_value) = vob.pop() {
				if is_value {
					// copy the byte into both ring buffer and output data
					let value = vec_pop_n(&mut vob, 8) as u8;
					data.push(value);
					rb_index = (rb_index + 1) % 4096;
					ringbuffer[rb_index] = value;
				} else {
					// grab the offset and how many bytes to copy from ring buffer
					let offset = vec_pop_n(&mut vob, 12);
					let size = vec_pop_n(&mut vob, 4) + padding;

					for i in (0..size).rev() {
						let value = ringbuffer[offset + i];
						data.push(value);
						rb_index = (rb_index + 1) % 4096;
						ringbuffer[rb_index] = value;
					}
				}
			}

			slopover_bits = vob;
			println!("{}", vob.len());
		}

		Ok(LZSSArchive {
			header: header,
			data: data,
		})
	}
}

#[cfg(feature = "import")]
#[derive(Error, Debug)]
pub enum LZSSImportError {
	#[error("I/O error")]
	IO {
		#[from]
		source: io::Error,
	},
	#[error("Not an LZSS file: {0:X}")]
	Magic(u32),
}

#[test]
fn test_lzss() {
	use std::{
		fs::{
			File,
			read
		},
		io::{
			Cursor,
			Write
		}
	};

	let mut data = Cursor::new(read("test_data/2540.enemy_drop_bonus_list.lzss").unwrap());
	let arc = LZSSArchive::read(2, &mut data).unwrap();
	let mut outf = File::create("test_data/2540.scp").unwrap();
	outf.write_all(arc.data.as_slice()).unwrap();
}
