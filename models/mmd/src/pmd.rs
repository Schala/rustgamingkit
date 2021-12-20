use bitflags::bitflags;

pub const MAGIC: [u8; 3] = ['P', 'm', 'd'];
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
