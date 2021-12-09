#[cfg(feature = "import")]
pub mod import {
	use nom::error::{
		ErrorKind,
		ParseError
	};

	use std::io;
	use thiserror::Error;

	#[derive(Error, Debug, PartialEq)]
	pub enum FBXImportError<'a> {
		#[error("Unknown/unsupported encoding ID: {1}")]
		Encoding(u32),
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Parser error")]
		Parse(&'a str, ErrorKind),
	}

	impl<'a> ParseError<&'a str> for FBXImportError<'a> {
		fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
			FBXImportError::Parse(input, kind)
		}

		fn append(_: &'a str, _: ErrorKind, other: Self) -> Self {
			other
		}
	}
}
