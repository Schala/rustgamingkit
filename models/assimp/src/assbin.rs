use bitflags::bitflags;

pub static MAGIC: &[u8] = b"ASSIMP.binary-dump.";
const RESERVE_PADDING: [u8; 64] = [0xCD; 64];

bitflags! {
	pub struct SceneFlag: u32 {
		const INCOMPLETE = 1,
		const VALIDATED = 2;
		const VALIDATION_WARNING = 4;
		const NON_VERBOSE_FORMAT = 8;
		const TERRAIN = 16;
		const ALLOW_SHARED = 32;
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum FileType {
	Normal = 0,
	Shortened,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum ChunkID {
	Camera = 0x1234,
	Light,
	Texture,
	Mesh,
	NodeAnimation,
	Scene,
	Bone,
	Animation,
	Node,
	Material,
	MaterialProperty,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum Compression {
	Uncompressed = 0,
	Deflate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: [u8; 44],
	pub major_ver: u32,
	pub minor_ver: u32,
	pub svn_rev: u32,
	pub compile_flags: u32,
	pub kind: FileType,
	pub compression: Compression,
	pub source_file: [u8; 256],
	pub cmd_line_params: [u8; 128],
	reserved: [u8; 64],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneHeader {
	pub flags: SceneFlag,
	pub num_anims: u32,
	pub num_cams: u32,
	pub num_lights: u32,
	pub num_mats: u32,
	pub num_meshes: u32,
	pub num_texs: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
	pub id: ChunkID,
	pub size: u32,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssimpBinaryDump {
	pub header: Header,
	pub chunks: Vec<Chunk>,
}
