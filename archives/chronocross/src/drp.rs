use rgk_core::tag4;

pub const MAGIC: u32 = tag4!(b"drp\x00");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	reserved_4: u32,
	pub num_res: u16,
	reserved_a: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ResType {
	NestedDRP = 1,
	GenericMesh,
	SpriteInfo,
	Texture,
	MusicInstrument,
	ModelPack = 11,
	BattleFieldMesh = 18,
	LightInfo = 21,
	MusicSequence,
	Animation = 25,
	Compressed = 37,
	Unknown = 255,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResHeader {
	reserved: u32,
	pub name: [u8; 4],
	pub kind: ResType,
	pub size: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
	pub header: ResHeader,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynResPack {
	pub header: Header,
	pub resources: Vec<Resource>,
}
