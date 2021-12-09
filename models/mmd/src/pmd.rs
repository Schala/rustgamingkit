use bitflags::bitflags;

static MAGIC: &[u8] = b"Pmd";
static VERSION: f32 = 1.0;

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
