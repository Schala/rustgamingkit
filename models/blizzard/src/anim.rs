use bitflags::bitflags;

use crate::m2::Bounds;

bitflags! {
	pub struct SeqFlags: u32 {
		const INIT = 1;
		const UNKNOWN_00000002 = 1 << 1;
		const UNKNOWN_00000004 = 1 << 2;
		const UNKNOWN_00000008 = 1 << 3;
		const LOW_PRIORITY = 1 << 4;
		const PRIMARY_BONE_SEQ = 1 << 5;
		const HAS_NEXT_IS_ALIAS = 1 << 6;
		const BLENDED = 1 << 7;
		const STORED_IN_MODEL = 1 << 8;
		const BLEND_TIME_OP = 1 << 9;
		const UNKNOWN_00000400 = 1 << 10;
		const UNKNOWN_00000800 = 1 << 11; // Legion 24500 models
	}
}

pub enum Timestamp {
	StartEnd(u32, u32),
	Duration(u32),
}

pub struct Range {
	pub min: u32,
	pub max: u32,
}

pub enum BlendTime {
	Old(u32),
	InOut(u16, u16),
}

pub struct Sequence {
	pub id: u16,
	pub variation_index: u16,
	pub timestamp: Timestamp,
	pub speed: f32,
	pub flags: SeqFlags,
	pub frequency: i16,
	padding: u16,
	pub replay: Range, // both 0 = no repeat
	pub blend_time: BlendTime,
	pub bounds: Bounds,
	pub variation_next: i16,
	pub alias_next: u16,
}
