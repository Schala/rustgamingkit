use crate::mesh::{
	FaceType,
	UV,
	UVFooter
};

pub const MAGIC: u32 = 0x80000001;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub num_vertices: u16,
	pub num_faces: u16,
	pub vertex_offset: u32,
	pub uv_offset: u32,
	pub uv_footer_offset: u32,
	pub
}
