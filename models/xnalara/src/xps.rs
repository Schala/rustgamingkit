use bitflags::bitflags;

use nom::{
	ErrorKind,
	ParseError
};

use std::io;
use thiserror::Error;

bitflags! {
	pub struct Flags: u8 {
		const BACK_FACE_CULLING = 1;
		const ALWAYS_FORCE_CULLING = 2;
		const CAST_SHADOWS = 4;
		const TANGENT_SPACE_RED = 8;
		const TANGENT_SPACE_GREEN = 16;
		const TANGENT_SPACE_BLUE = 32;
		const GLOSS = 64;
		const BONE_DIRECTIONS = 128;
	}
}

impl Default for Flags {
	fn default() -> Self {
		Self::CAST_SHADOWS | Self::TANGENT_SPACE_GREEN | Self::GLOSS
	}
}

#[cfg(feature = "import")]
pub mod import {
	#[derive(Error, Debug)]
	pub enum XPSImportError<'a> {
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Not an XNALara model file: {0:X}")]
		Magic(u32),
		#[error("Not an XNALara model file")]
		MagicStr(String),
		#[error("Parser error")]
		Parse(&'a str, ErrorKind),
	}

	impl<'a> ParseError<&'a str> for XPSImportError<'a> {
		fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
			XPSImportError::Parse(input, kind)
		}

		fn append(_: &'a str, _: ErrorKind, other: Self) -> Self {
			other
		}
	}
}
