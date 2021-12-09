pub const MAGIC: u32 = 0x80000001;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum FaceType {
	/// Tris, 1 vertex color, 3 vertex indices, and 2 bytes of padding (?)
	TriCol1Idx = 32,
	/// Tris, 1 vertex color, UV map, 3 vertex indices
	TriCol1Idx = 36,
	/// Quads, 1 vertex color, 4 vertex indices
	QuadCol1Idx = 40,
	/// Quads, 1 vertex color, Uv map, 4 vertex indices
	QuadCol1UvIdx = 44,
	/// Tris, 3 vertex colors, 3 vertex indices
	TriCol3Idx = 48,
	/// Tris, 3 vertex colors, UV map, 3 vertex indices
	TriCol3UvIdx = 52,
	/// Quads, 3 vertex colors, 4 vertex indices
	QuadCol3Idx = 56,
	/// Quads, 3 vertex colors, UV map, 4 vertex indices
	QuadCol3Idx = 60,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UV {
	pub kind: FaceType,
	pub num_faces: u16,
	pub uv_offset: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UVFooter {
	pub num_entries: u32,
	pub entries: Vec<UV>,
}
