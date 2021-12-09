#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetaRec {
	pub uv_offset: u32,
	pub vertex_offset: u32,
	pub vddm_offset: u32,
}

/// Similar to the CPT header, but with a checksum and 4 mystery bytes
#[derive(Clone, Debug, PartialEq)]
pub struct MetaInfoHeader {
	pub num_units: u32,
	pub offsets: Vec<u32>,
	pub checksum: u32,
	pub unknown: u32,
}

/// Contains info on UV map, vertices, vestigial data, and skeleton
pub struct MetaInfo {
	pub header: MetaInfoHeader,
}

#[cfg(feature = "import")]
pub mod import {
	use std::io;
	use thiserror::Error;

	#[derive(Error, Debug)]
	pub enum CCImportError {
		#[error("I/O error")]
		Io {
			#[from]
			source: io::Error,
		},
		#[error("Offset out of bounds: {0}")]
		Offset(u32),
	}
}
