use byteorder::{
	LE,
	ReadBytesExt
};

use crate::xps::import::XPSImportError;
use import::text;

const MAGIC: u32 = 323232;
static MAGIC_STR: &str = "XNAaraL";
const ROUND_MULTIPLE: u32 = 4;
const STR_LEN_LIMIT: u8 = 128;

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub ver_major: u16,
	pub ver_minor: u16,
	pub magic_str: String,
	pub settings_len: u32,
	pub author_device_name: String,
	pub author: String,
	pub file: String,
}

impl Header {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Header, XPSImportError>
	where
		R: ReadBytesExt,
	{
		let magic = buf.read_u32::<LE>()?;
		if magic != MAGIC {
			return Err(XPSImportError::Magic(magic));
		}

		let vmajor = buf.read_u16::<LE>()?;
		let vminor = buf.read_u16::<LE>()?;

		let magic_str = text(buf)?;
		if magic_str != MAGIC_STR {
			return Err(XPSImportError::MagicStr(magic_str));
		}

		let settings_len = buf.read_u32::<LE>()?;
		let device = rev_str(text(buf)?.as_bytes());
		let user = rev_str(text(buf)?.as_bytes());
		let files = text(buf)?;

		let has_tangent = vminor <= 12 && vmajor <= 2;
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tangent {
	Old(Vec<u8>),
	New(u32, u32, Vec<Item>),
}

impl Tangent {
	#[cfg(feature = "import")]
	fn read<R>(is_tangent: bool, length: usize, buf: &mut R) -> Result<Tangent, XPSImportError>
	where
		R: ReadBytesExt,
	{
		if is_tangent {
			let mut settings = vec![0; length];
			buf.read_exact(&mut settings.as_slice())?;

			Ok(Tangent::Old(settings))
		} else {
			let hash = buf.read_u32::<LE>()?;
			let nitems = buf.read_u32::<LE>()?;

			let mut rpos = 8;
		}
	}
}

/// Reverses a string's contents
fn rev_str(s: &str) -> String {
	s.chars().rev().collect()
}

fn round_to_multiple(num: u32, multiple: u32) -> u32 {
	(num + multiple - 1) / multiple * multiple
}

#[cfg(feature = "export")]
pub mod export {
	const VER_MAJOR: u16 = 3;
	const VER_MINOR: u16 = 15;

	#[cfg(windows)]
	include!("windows.rs");

	#[cfg(unix)]
	include!("unix.rs");

	#[cfg(test)]
	mod tests {
		#[test]
		fn test_username() {
			let uname = super::get_user_name().unwrap();
			println!("{}", &uname);
			assert_ne!(uname.len(), 0);
		}

		#[test]
		fn test_device_name() {
			let dname = super::get_device_name().unwrap();
			println!("{}", &dname);
			assert_ne!(dname.len(), 0);
		}
	}
}

#[cfg(feature = "import")]
pub mod import {
	use byteorder::ReadBytesExt;

	use crate::xps::import::XPSImportError;
	use super::*;

	/// Parses a length-prefixed string
	pub fn text<R>(buf: &mut R) -> Result<String, XPSImportError>
	where
		R: ReadBytesExt,
	{
		let mut len_byte2 = 0;
		let len_byte1 = buf.read_u8()?;

		if len_byte1 >= STR_LEN_LIMIT {
			len_byte2 = buf.read_u8()?;
		}

		let length = ((len_byte1 % STR_LEN_LIMIT) + (len_byte2 * STR_LEN_LIMIT)) as usize;
		let mut s = String::new();
		let mut chr = [0; 1];
		for _ in 0..length {
			self.read_exact(&mut chr)?;
			s.push(chr[0] as char);
		}
	}

	#[cfg(test)]
	mod tests {
		#[test]
		fn test_string() {
			assert_eq!(super::text(&b"\x04heya"[..]), Ok((&b""[..], "heya")));
		}
	}
}
