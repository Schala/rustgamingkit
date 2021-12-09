use byteorder::{
	LE,
	ReadBytesExt,
	WriteBytesExt
};

use std::io::Result;

/// Recurring file header in Chrono Cross data files
#[derive(Clone, Debug, PartialEq)]
pub struct Header {
	pub num_objs: u32,
	pub offsets: Vec<u32>,
}

impl Header {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Header>
	where
		R: ReadBytesExt,
	{
		let nobjs = buf.read_u32::<LE>()?;
		let mut offsets = vec![];
		for _ in 0..(nobjs as usize) {
			offsets.push(buf.read_u32::<LE>()?);
		}

		Ok(Header {
			num_objs: nobjs,
			offsets: offsets,
		})
	}

	#[cfg(feature = "export")]
	fn write<W>(&self, buf: &mut W) -> Result<()>
	where
		W: WriteBytesExt,
	{
		buf.write_u32::<LE>(self.num_objs)?;

		for offset in self.offsets.iter() {
			buf.write_u32::<LE>(*offset)?;
		}

		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct CPTArchive {
	pub header: Header,
	pub objects: Vec<Vec<u8>>,
	pub has_eof_ptr: bool,
}

impl CPTArchive {
	#[cfg(feature = "export")]
	pub fn new(objs: Vec<Vec<u8>>, has_eof_ptr: bool) -> CPTArchive {
		let mut offset = (objs.len() * 4 + 4) as u32;
		let mut offsets = vec![];

		if has_eof_ptr {
			offset += 4;
		}

		offsets.push(offset);

		for i in 0..(objs.len() - 1) {
			offset += objs[i].len() as u32;
			offsets.push(offset);
		}

		if has_eof_ptr {
			offset += objs[objs.len() - 1].len() as u32;
			offsets.push(offset);
		}

		CPTArchive {
			header: Header {
				num_objs: objs.len() as u32,
				offsets: offsets,
			},
			objects: objs,
			has_eof_ptr: has_eof_ptr,
		}
	}

	#[cfg(feature = "export")]
	pub fn write<W>(&self, buf: &mut W) -> Result<()>
	where
		W: WriteBytesExt,
	{
		self.header.write(buf)?;

		for obj in self.objects.iter() {
			buf.write_all(obj.as_slice())?;
		}

		Ok(())
	}

	#[cfg(feature = "import")]
	pub fn read<R>(has_eof_ptr: bool, buf: &mut R) -> Result<CPTArchive>
	where
		R: ReadBytesExt,
	{
		let header = Header::read(buf)?;

		let length = if has_eof_ptr {
			(header.num_objs - 1) as usize
		} else {
			header.num_objs as usize
		};

		let mut objs = vec![];
		for i in 0..length {
			if i < (length - 1) {
				let size = (header.offsets[i + 1] - header.offsets[i]) as usize;
				let mut data = vec![0; size];
				buf.read_exact(data.as_mut_slice())?;

				objs.push(data);
			} else {
				let mut data = vec![];
				buf.read_to_end(&mut data)?;

				objs.push(data);
			}
		}

		Ok(CPTArchive {
			header: header,
			objects: objs,
			has_eof_ptr: has_eof_ptr,
		})
	}
}

#[cfg(test)]
mod tests {
	#[cfg(feature = "export")]
	#[test]
	fn test_write() {
		use std::fs::{
			File,
			read
		};

		let data = vec![read("test_data/Screenshot_20211206_222120.png").unwrap(),
			read("test_data/Screenshot_20211206_230126.png").unwrap(),
			read("test_data/Screenshot_20211206_232554.png").unwrap()];

		let cpt = super::CPTArchive::new(data, true);
		let mut f = File::create("test_data/test.cpt").unwrap();
		println!("{:?}", cpt.header);
		cpt.write(&mut f).unwrap();
	}
}
