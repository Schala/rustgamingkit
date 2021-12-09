use std::collections::HashMap;

/// This follows the [`MAGIC`] and is stored as a 2 byte little endian value.
pub const HEADER_SIZE: u16 = 26;

pub static MAGIC: &[u8] = b"Kaydara FBX Binary  \x00";
pub const VERSION: u32 = 7300;

pub struct Array {
	pub encoding: Encoding,
	pub contents: PropertyData,
}

/// Encoding for array property contents
#[repr(u32)]
pub struct Encoding {
	None = 0,
	/// Deflate/zlib encoding
	Deflate,
}

/// FBX file header
pub struct Header {
	pub magic: [u8; 20],
	pub version: u32,
}

/// Named node record
pub struct Node {
	/// Distance between start of the file to end of the node record
	/// This is useful for skipping over unknown or non-essential records.
	pub end_offset: u32,
	pub num_properties: u32,
	pub property_list_size: u32,
	/// This is usually blank for top-level nodes. It's parsed as a Pascal string.
	pub name: String,
}

/// Property payload
pub enum PropertyData {
	/// Stored with a 4-byte length prefix
	Binary(Vec<u8>),
	/// Encoded as the LSB of a 1-byte value
	Boolean(bool),
	Float32(f32),
	Float64(f64),
	Int16(i16),
	Int32(i32),
	Int64(i64),
	/// Stored with a 4-byte length prefix. This is not null terminated and may
	/// contain null bytes.
	Text(String),
}

pub type PropertyList = HashMap<TypeCode, PropertyData>;

#[repr(u8)]
pub enum TypeCode {
	Boolean = b'C',
	Float64 = b'D',
	Float32 = b'F',
	Int32 = b'I',
	Int64 = b'L',
	Binary = b'R',
	Text = b'S',
	Int16 = b'Y',
	BooleanArray = b'b',
	Float64Array = b'd',
	Float32Array = b'f',
	Int32Array = b'i',
	Int64Array = b'l',
}

#[cfg(feature = "export")]
pub mod export {
	const SENTINEL: [u8; 13] = [0; 13];
}

#[cfg(feature = "import")]
pub mod import {
	use crate::import::FBXImportError;
	use super::*;
}
