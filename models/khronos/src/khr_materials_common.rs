use ultraviolet::vec::Vec3;

pub struct Light {
	pub kind: (String, Vec3),
	pub name: String,
}

pub enum MatExtTech {
	Phong,
}

pub struct MaterialExts {
	pub double_sided: bool,
	pub technique: MatExtTech,
	pub transparent: bool,
}
