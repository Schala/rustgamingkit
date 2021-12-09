use bitflags::bitflags;

use meshio_core::tag4;

pub const MAGICS: [u32; 2] = [tag4!(b"MD20"), tag4!(b"MD21")];
pub const MAX_CAMERA_HEIGHT: f32 = 0.16;

bitflags! {
	pub struct GlobalFlags: u32 {
		const TILT_X = 1;
		const TILT_Y = 1 << 1;
		const UNKNOWN_00000004 = 1 << 2;
		const USE_TEXTURE_COMBINATOR_COMBOS = 1 << 3;
		const UNKNOWN_00000010 = 1 << 4;
		const LOAD_PHYS_DATA = 1 << 5;
		const UNKNOWN_00000040 = 1 << 6;
		const UNKNOWN_00000080 = 1 << 7; // demon hunter tattoos stop glowing
		const UNKNOWN_00000100 = 1 << 8; // camera stuff
		const UNKNOWN_00000200 = 1 << 9; // new particle system stuff
		const UNKNOWN_00000400 = 1 << 10;
		const TEXTURE_XFORMS_USE_BONE_SEQS = 1 << 11;
		const UNKNOWN_00001000 = 1 << 12;
		const UNKNOWN_00002000 = 1 << 13; // in various Legion models
		const UNKNOWN_00004000 = 1 << 14;
		const UNKNOWN_00008000 = 1 << 15; // in Legion UI_MainMenu
		const UNKNOWN_00010000 = 1 << 16;
		const UNKNOWN_00020000 = 1 << 17;
		const UNKNOWN_00040000 = 1 << 18;
		const UNKNOWN_00080000 = 1 << 19;
		const UNKNOWN_00100000 = 1 << 20;
		const UNKNOWN_00200000 = 1 << 21; // chunked .anim files, reorder seq+bone blocks
	}
}

pub struct Array {
	pub size: u32,
	pub offset: u32, // relative to file start
}

pub enum SkinProfileInfo {
	Data(Array),
	Count(u32),
}

pub struct AABox {
	pub min: f32,
	pub max: f32,
}

pub struct Header {
	pub magic: u32,
	pub version: u32,
	pub name: Array,
	pub flags: GlobalFlags,
	pub loops: Array,
	pub sequences: Array,
	pub seq_index_by_hash_id: Array,
	pub playable_anim_lookup: Option<Array>,
	pub bones: Array,
	pub bone_indices_by_id: Array,
	pub vertices: Array,
	pub skin_profiles: SkinProfileInfo,
	pub colors: Array,
	pub textures: Array,
	pub tex_weights: Array,
	pub tex_flipbooks: Option<Array>,
	pub tex_transforms: Array,
	pub tex_indices_by_id: Array,
	pub materials: Array,
	pub bone_combos: Array,
	pub tex_combos: Array,
	pub tex_transform_bone_map: Array,
	pub tex_weight_combos: Array,
	pub tex_transform_combos: Array,
	pub bounding_box: AABox,
	pub bounding_sphere_radius: f32,
	pub collision_box: AABox,
	pub collision_sphere_radius: f32,
	pub collision_indices: Array,
	pub collision_positions: Array,
	pub collision_face_normals: Array,
	pub attachments: Array,
	pub attachment_indices_by_id: Array,
	pub events: Array,
	pub lights: Array,
	pub cameras: Array,
	pub cam_indices_by_id: Array,
	pub ribbon_emitters: Array,
	pub particle_emitters: Array,
	pub tex_combinator_combos: Option<Array>,
}

pub struct Bounds {
	pub extent: AABox,
	pub radius: f32,
}

#[cfg(feature = "import")]
pub mod import {
	use std::io;

	use thiserror::Error;

	#[derive(Error, Debug)]
	pub enum M2ImportError {
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Not a World of Warcraft model file: {0:X}")]
		Magic(u32),
	}
}
