use bitflags::bitflags;

use rgk_core::tag3;

pub const MAGIC: u32 = tag3!(b"Pmd");
pub static VERSION: f32 = 1.0;

#[repr(u8)]
pub enum BoneType {
	IKFollow = 4,
	CoRotate = 9,
}

#[repr(u8)]
pub enum FaceMorphType {
	Other = 0,
	Eyebrow,
	Eye,
	Lip
}
