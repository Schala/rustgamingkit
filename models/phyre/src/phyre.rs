use meshio_core::{
	rtag4,
	tag4
};

pub const MAGIC_BE: u32 = tag4!("PHYR");
pub const MAGIC_LE: u32 = rtag4!("PHYR");

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum PhyrePlatform {
	DirectX11 = tag4!("DX11"),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhyreClusterHeader {
	pub magic: u32,
	pub size: u32,
	pub packed_namespace_size: u32,
	pub platform: PhyrePlatform,
	pub instance_count: u32,
	pub array_fixup_size: u32,
	pub array_fixup_count: u32,
}
