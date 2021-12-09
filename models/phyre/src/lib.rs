use bitflags::bitflags;

bitflags! {
	pub struct PhyreImportFlag: u32 {
		const INCLUDE_NORMALS = 1;
		const INVERT_UV_Y = 2;
	}
}
