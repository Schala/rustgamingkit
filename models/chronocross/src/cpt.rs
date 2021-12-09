use byteorder::{
	LE,
	ReadBytesExt
};

use thiserror::Error;

/// Recurring file header in Chrono Cross data files, including 3D models
#[derive(Clone, Debug, PartialEq)]
pub struct CPTHeader {
	pub num_objs: u32,
	pub offsets: Vec<u32>,
}

impl CPTHeader {
	#[cfg(feature = "import")]
	pub fn read<R, E>(buf: &mut R) -> Result<CPTHeader, E>
	where
		R: ReadBytesExt,
		E: Error,
	{
		let nobjs = buf.read_u32::<LE>()?;
		let mut offsets = vec![];
		for _ in 0..(nobjs as usize) {
			offsets.push(buf.read_u32::<LE>()?);
		}

		Ok(input, CPTHeader {
			num_objs: nobjs,
			offsets: offsets,
		})
	}
}
