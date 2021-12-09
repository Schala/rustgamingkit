use meshio_core::tag4;

pub const MAGIC: u32 = tag4!(b"glTF");

pub struct Header {
	pub magic: u32,
	pub version: u32,
	pub size: u32,
}

#[repr(u32)]
pub enum ChunkType {
	Binary = tag4!(b"BIN\x00"),
	Json = tag4!(b"JSON"),
	Unknown,
}

pub struct Chunk {
	pub size: u32,
	pub kind: ChunkType,
	pub data: Vec<u8>,
}

#[cfg(feature = "export")]
pub mod export {
	pub const VERSION: u32 = 2;
}
