use json::JsonValue;

use rgk_core::tag4;

pub const MAGIC: u32 = tag4!(b"fspy");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub version: u32,
	pub state_len: u32,
	pub image_len: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FSpyProject {
	pub header: Header,
	pub state: JsonValue,
	pub image: Vec<u8>,
}

#[cfg(feature = "import")]
pub mod import {
	use json::{
		from,
		JsonValue,
		parse,
		self
	};

	use std::io;
	use thiserror::Error;

	#[derive(Debug, Error)]
	pub enum FSpyImportError {
		#[error("I/O error")]
		Io {
			#[from]
			source: io::Error,
		},
		#[error("JSON parsing error")]
		Json(json::Error),
	}
}
