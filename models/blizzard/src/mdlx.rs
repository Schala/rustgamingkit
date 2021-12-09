use bitflags::bitflags;

use ultraviolet::vec::{
    Vec3,
    Vec4
};

use crate::mdx::TrackTag;

bitflags! {
    pub struct NodeFlags: u32 {
        const HELPER = 0;
        const DONT_INHERIT_TRANSLATION = 1;
        const DONT_INHERIT_ROTATION = 1 << 1;
        const DONT_INHERIT_SCALE = 1 << 2;
        const BILLBOARDED = 1 << 3;
        const BILLBOARD_LOCK_X = 1 << 4;
        const BILLBOARD_LOCK_Y = 1 << 5;
        const BILLBOARD_LOCK_Z = 1 << 6;
        const CAMERA_ANCHORED = 1 << 7;
        const BONE = 1 << 8;
        const LIGHT = 1 << 9;
        const EVENT_OBJECT = 1 << 10;
        const ATTACHMENT = 1 << 11;
        const PARTICLE_EMITTER = 1 << 12;
        const COLLISION_SHAPE = 1 << 13;
        const RIBBON_EMITTER = 1 << 14;
        const MDL_OR_UNSHADED = 1 << 15;
        const TGA_OR_SORT_FAR_Z = 1 << 16;
        const LINE_EMITTER = 1 << 17;
        const UNFOGGED = 1 << 18;
        const MODEL_SPACE = 1 << 19;
        const XY_QUAD = 1 << 20;
    }

    pub struct Shading: u32 {
        const UNSHADED = 1;
        const SPHERE_ENVIRONMENT_MAP = 1 << 1;
        const UNKNOWN_4 = 1 << 2;
        const UNKNOWN_8 = 1 << 3;
        const TWO_SIDED = 1 << 4;
        const UNFOGGED = 1 << 5;
        const NO_DEPTH_TEST = 1 << 6;
        const NO_DEPTH_SET = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackValue {
    AttenuationStart(f32),
    AttenuationStartEnd(f32),
    Alpha(f32),
    CameraRotation(f32),
    Color(Vec3),
    EmissiveGain(f32),
    FresnelTeamColor(f32),
    Gravity(f32),
    Height(f32),
    Intensity(f32),
    Latitude(f32),
    Length(f32),
    Lifespan(f32),
    Longitude(f32),
    Rotation(Vec4),
    Scale(Vec3),
    Speed(f32),
    Texture(u32),
    Translation(Vec3),
    Variation(f32),
    Visibility(f32),
    Width(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    pub frame: i32,
    pub value: TrackValue,
    pub in_tan: Option<TrackValue>,
    pub out_tan: Option<TrackValue>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum TrackInterpolation {
    None = 0,
    Linear,
    Hermite,
    Bezier,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackChunk {
    pub tag: TrackTag,
    pub interp: TrackInterpolation,
    pub global_seq_id: u32,
    pub tracks: Vec<Track>,
}

/// Object bounds
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    pub bounds_radius: f32,
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for Extent {
	fn default() -> Self {
		Self {
			bounds_radius: 0.0,
			min: Vec3::zero(),
			max: Vec3::zero(),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub size: u32,
    pub name: String,
    pub obj_id: u32,
    pub parent_id: u32,
    pub flags: NodeFlags,
    pub kgtr: TrackChunk,
    pub kgrt: TrackChunk,
    pub kgsc: TrackChunk,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum SeqFlags {
    Looping = 0,
    NonLooping,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Seq {
    pub name: String,
    pub interval: [u32; 2],
    pub speed: f32,
    pub flags: SeqFlags,
    pub rarity: f32,
    pub sync_point: u32,
    pub extent: Extent,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    pub replaceable_id: u32,
    pub file_name: String,
    pub flags: u32, // ?
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundTrack {
    pub file_name: String,
    pub volume: f32,
    pub pitch: f32,
    pub flags: u32, // ?
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Filter {
    None = 0,
    Transparent,
    Blend,
    Additive,
    AddAlpha,
    Modulate,
    Modulate2X,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerV800 {
    pub emissive_gain: f32,
    pub fresnel_color: Vec3,
    pub fresnel_opacity: f32,
    pub fresnel_team_color: f32,
    pub kmte: TrackChunk,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerV900 {
    pub kfc3: TrackChunk,
    pub kfca: TrackChunk,
    pub kftc: TrackChunk,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub size: u32,
    pub filter: Filter,
    pub shading: Shading,
    pub tex: u32,
    pub tex_anim: u32,
    pub coord: u32,
    pub alpha: f32,
    pub v800: Option<LayerV800>,
    pub kmtf: TrackChunk,
    pub kmta: TrackChunk,
    pub v900: Option<LayerV900>,
}

impl Default for Layer {
	fn default() -> Self {
		Self {
			size: 0,
			filter: Filter::None,
			shading: Shading::UNSHADED,
			tex: 0,
			tex_anim: 0,
			coord: 0,
			alpha: 1.0,
			v800: None,

		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub size: u32,
    pub priority_plane: u32,
    pub flags: u32, // ?
    pub v800_shader: Option<String>,
    pub layers: Vec<Layer>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TexAnim {
    pub size: u32,
    pub ktat: TrackChunk,
    pub ktar: TrackChunk,
    pub ktas: TrackChunk,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChunkData {
    Version(u32),
    ModelInfo {
        name: String,
        anim_file_name: String,
        extent: Extent,
        blend_time: u32,
    },
    Pivots(Vec<Vec3>),
}

#[cfg(feature = "import")]
pub mod import {
	use nom::error::{
		ErrorKind,
		ParseError
	};

	use std::str::Utf8Error;
	use thiserror::Error;

	#[derive(Error, Debug, PartialEq)]
	pub enum WC3ImportError<'a> {
		#[error("Unknown/unsupported filter type")]
		Filter(u32),
		#[error("Invalid global sequence ID")]
		GlobalSeqId(u32),
		#[error("Invalid object ID")]
		ID(u32),
		#[error("Unknown/unsupported linear interpolation type")]
		Interpolation(u32),
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Unknown/unsupported loop flag")]
		Looping(u32),
		#[error("Not a Warcraft 3 model file")]
		Magic(u32),
		#[error("Unknown/unsupported node flags")]
		NodeFlags(u32),
		#[error("Parser error")]
		Parse(&'a str, ErrorKind),
		#[error("Unknown/unsupported shading flags")]
		ShadingFlags(u32),
		#[error("Unknown/unsupported chunk tag")]
		Tag(u32),
		#[error("Unknown/unsupported track tag")]
		TrackTag(u32),
		#[error("String is not valid UTF-8")]
		UTF8(Utf8Error, Vec<u8>),
	}

	impl<'a> ParseError<&'a str> for WC3ImportError<'a> {
		fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
			WC3ImportError::Parse(input, kind)
		}

		fn append(_: &'a str, _: ErrorKind, other: Self) -> Self {
			other
		}
	}
}
