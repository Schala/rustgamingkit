use bitflags::bitflags;

use ultraviolet::{
	mat::Mat4,
	vec::{
		Vec2,
		Vec3
	}
};

use meshio_core::rtag4;

const MAGICS: [u32; 2] = [rtag4!(b"MD33"), rtag4!(b"MD34")];

bitflags! {
	pub struct BoneFlags: u32 {
		const NONE = 0;
		const INHERIT_TRANSLATION = 1;
		const INHERIT_SCALE = 1 << 1;
		const INHERIT_ROTATION = 1 << 2;
		const BILLBOARD_1 = 1 << 4;
		const BILLBOARD_2 = 1 << 6;
		const PROJECTION_2D = 1 << 8;
		const ANIMATED = 1 << 9;
		const INVERSE_KINEMATICS = 1 << 10;
		const SKINNED = 1 << 11;
		const REAL = 1 << 13;
	}

	pub struct ModelFlags: u32 {
		const HAS_MESH = 1 << 20;
	}

	pub struct VertexFlags: u32 {
		const HAS_VERTEX_COLORS = 1 << 9;
		const HAS_VERTICES = 1 << 17;
		const USE_UV_CHANNEL_1 = 1 << 18;
		const USE_UV_CHANNEL_2 = 1 << 19;
		const USE_UV_CHANNEL_3 = 1 << 20;
		const USE_UV_CHANNEL_5 = 1 << 30;
	}
}

pub struct Bone {
	pub unknown_00: i32, // keybone?
	pub name: Ref, // 4
	pub flags: BoneFlags, // c
	pub parent: i16, // 10
}

pub struct Bounds {
	pub min: Vec3,
	pub max: Vec3,
	pub radius: f32,
}

/// File header
pub struct Header {
	pub magic: u32,
	pub table_offset: u32,
	pub table_entries: u32,
	pub modl: Ref,
}

/// Section metadata
pub struct Index {
	pub tag: u32,
	pub offset: u32,
	pub count: u32,
	pub version: u32,
}

pub struct Material {
	pub name: Ref,
	pub unknown_08: [i32; 8],
	pub xy: Vec2, // always 1.0? (5.0, 3.5) in Immortal.m3, scaling?
	pub layers: [Ref; 13],
	pub unknown_98: [i32; 15],
}

/// The MODL block contains the actual info about the model
pub struct M3Model {
	pub name: Ref,
	pub flags: ModelFlags, // 8
	pub seqs: Ref, // c
	pub stc: Ref, // 14
	pub stg: Ref, // 1c
	pub unknown_024: Ref,
	pub sts: Ref, // 2c
	pub bone: Ref, // 34
	pub skinned_bones: u32, // 3c
	pub vflags: VertexFlags, // 44
	pub vertices: Ref, // 48
	pub divisions: Ref, // 50
	pub bone_lookup: Ref, // 58
	pub extents: Ref, // 60
	pub unknown_068: u32, // extents flags?
	pub unknown_06c: Ref,
	pub unknown_074: Ref,
	pub unknown_07c: Ref,
	pub unknown_084: [u32; 6],
	pub att: Ref, // a0
	pub attachment_point_addons: Ref, // a8
	pub lite: Ref, // b0
	pub shbx: Ref, // b8
	pub cam: Ref, // c0
	pub unknown_0c8: Ref,
	pub matm: Ref, // d0
	pub mat: Ref, // d8
	pub dis: Ref, // e0
	pub cmp: Ref, // e8
	pub ter: Ref, // f0
	pub vol: Ref, // f8
	pub unknown_100: Ref,
	pub crep: Ref, // 108
	pub von: Ref, // 110
	pub stbm: Ref, // 118
	pub refl_mats: Ref, // 120
	pub lflr: Ref, // 128
	pub madd: Ref, // 130
	pub par: Ref, // 138
	pub parc: Ref, // 140
	pub rib: Ref, // 148
	pub proj: Ref, // 150
	pub forces: Ref, // 158
	pub wrp: Ref, // 160
	pub unknown_168: Ref,
	pub phrb: Ref, // 170
	pub unknown_178: Ref,
	pub phyj: Ref, // 180
	pub phcl: Ref, // 188
	pub unknown_190: Ref,
	pub unknown_198: Ref,
	pub ikjt: Ref, // 1a0
	pub unknown_1a8: Ref,
	pub patu: Ref, // 1b0
	pub trgd: Ref, // 1b8
	pub iref: Ref, // 1c0
	pub tight_hit_test: SimpleShape, // 1c8
	pub ssgs: Ref, // 234
	pub atvl: Ref, // 24c
	pub atvl_addons: [Ref; 2], // 254
	pub bbsc: Ref, // 25c
	pub tmd: Ref, // 264
	pub unknown_26c: u32, // 26c
	pub unknown_270: Ref, // 270
}

/// A reference to a block stored elsewhere
pub struct Ref {
	pub entries: u32,
	pub index: u32,
	pub flags: Option<u32>, // added in MD34
}

#[repr(u32)]
pub enum RefTag {
	AttachmentPoint = rtag4!(b"ATT_"),
	AttachmentVolume = rtag4!(b"ATVL"),
	UnknownBat = rtag4!(b"BAT_"),
	BillboardBehavior = rtag4!(b"BBSC"),
	Bounds = rtag4!(b"BNDS"),
	Bone = rtag4!(b"BONE"),
	Camera = rtag4!(b"CAM_"),
	Chars = rtag4!(b"CHAR"),
	CompositeMat = rtag4!(b"CMP_"),
	CompositeMatSection = rtag4!(b"CMS_"),
	CreepMat = rtag4!(b"CREP"),
	DisplacementMat = rtag4!(b"DIS_"),
	MeshDiv = rtag4!(b"DIV_"),
	UnknownDmmn = rtag4!(b"DMMN"),
	UnknownDmse = rtag4!(b"DMSE"),
	Event = rtag4!(b"EVNT"),
	Flags = rtag4!(b"FLAG"),
	Force = rtag4!(b"FOR_"),
	Int32 = rtag4!(b"I32_"),
	IkChain = rtag4!(b"IKJT"),
	Matrix4x4 = rtag4!(b"IREF"),
	MatLayer = rtag4!(b"LAYR"),
	LensFlareMat = rtag4!(b"LFLR"),
	LensFlareMatSub = rtag4!(b"LFSB"),
	Light = rtag4!(b"LITE"),
	BufferMat = rtag4!(b"MADD"),
	StdMat = rtag4!(b"MAT_"),
	MaterialRef = rtag4!(b"MATM"),
	Model = rtag4!(b"MODL"),
	UnknownMsec = rtag4!(b"MSEC"),
	UnknownMt32 = rtag4!(b"MT32"),
	Particle = rtag4!(b"PAR_"),
	ParticleCopy = rtag4!(b"PARC"),
	TurretPart = rtag4!(b"PATU"),
	UnknownPhac = rtag4!(b"PHAC"), // cloth related
	ClothConstraint = rtag4!(b"PHCC"),
	ClothBehavior = rtag4!(b"PHCL"),
	Rigidbody = rtag4!(b"PHRB"),
	PhysicsShape = rtag4!(b"PHSH"),
	PhysicsJoint = rtag4!(b"PHYJ"),
	Projection = rtag4!(b"PROJ"),
	Quaternion = rtag4!(b"QUAT"),
	Float = rtag4!(b"REAL"),
	ReflectionMat = rtag4!(b"REF_"),
	MeshRegion = rtag4!(b"REGN"),
	Ribbon = rtag4!(b"RIB_"),
	UnknownSchr = rtag4!(b"SCHR"),
	AnimVec2 = rtag4!(b"SD2V"),
	AnimVec3 = rtag4!(b"SD3V"),
	AnimQuat = rtag4!(b"SD4Q"),
	AnimColor = rtag4!(b"SDCC"),
	AnimEvent = rtag4!(b"SDEV"),
	AnimFlags = rtag4!(b"SDFG"),
	AnimBounds = rtag4!(b"SDMB"),
	AnimF32 = rtag4!(b"SDR3"),
	AnimI16 = rtag4!(b"SDS6"),
	AnimU32 = rtag4!(b"SDU3"),
	AnimU16 = rtag4!(b"SDU6"),
	Sequence = rtag4!(b"SEQS"),
	ShadowBox = rtag4!(b"SHBX"),
	SplineRibbon = rtag4!(b"SRIB"),
	SimpleGeoShape = rtag4!(b"SSGS"),
	SplatTerrainBakeMat = rtag4!(b"STBM"),
	SeqTransformCollection = rtag4!(b"STC_"),
	SeqTransformGroup = rtag4!(b"STG_"),
	UnknownSts = rtag4!(b"STS_"),
	TerrainMat = rtag4!(b"TER_"),
	UnknownTmd = rtag4!(b"TMD_"),
	TurretBehavior = rtag4!(b"TRGD"),
	Uint8 = rtag4!(b"U8__"),
	Uint16 = rtag4!(b"U16_"),
	Uint32 = rtag4!(b"U32_"),
	Vector2 = rtag4!(b"VEC2"),
	Vector3 = rtag4!(b"VEC3"),
	Vector4 = rtag4!(b"VEC4"),
	VolumeMat = rtag4!(b"VOL_"),
	VolumeNoiseMat = rtag4!(b"VON_"),
	Warp = rtag4!(b"WRP_"),
}

pub struct SeqData {
	pub timeline: Ref,
	pub flags: u32,
	pub length: u32,
	pub data: Ref,
}

pub struct Sequence {
	pub unknown_00: i32,
	pub unknown_04: i32,
	pub name: Ref,
	pub unknown_10: i32,
	pub length: i32,
	pub unknown_18: i32,
	pub flags: u32,
	pub unknown_20: [u32; 5],
	pub extents: Bounds,
	pub unknown_50: i32,
	pub unknown_54: i32,
}

#[repr(u32)]
pub enum SimpleShapeType {
	Cuboid = 0,
	Sphere,
	Cylinder,
}

pub struct SimpleShape {
	pub kind: SimpleShapeType,
	pub bone_index: i16, // 4
	pub unknown_08: u16, // 8
	pub matrix: Mat4, // c
	pub unknown_4c: Ref, // 4c
	pub unknown_54: Ref, // 54
	pub size: Vec3, // 60
}

/// Tex
pub struct TexLayer {

}

#[cfg(feature = "export")]
pub mod export {
	use ultraviolet::vec::Vec3;

	/// Coordinates are converted with (i + 1) / 2 * 255
	fn vec3_to_bytes(v: Vec3) -> [u8; 3] {
		[
			((v.x + 1.0) * 255.0 / 2.0) as u8,
			((v.y + 1.0) * 255.0 / 2.0) as u8,
			((v.z + 1.0) * 255.0 / 2.0) as u8,
		]
	}

	/// Coordinates are converted with i * 2048
	fn vec3_to_words(v: Vec3) -> [u16; 3] {
		[
			((v.x * 2048.0) as u16,
			((v.y * 2048.0) as u16,
			((v.z * 2048.0) as u16,
		]
	}
}

#[cfg(feature = "import")]
pub mod import {
	use thiserror::Error;
	use ultraviolet::vec::Vec3;

	#[derive(Error, Debug)]
	pub enum M3ImportError {
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Not a Blizzard M3 model file: {0:X}")]
		Magic(u32),
	}

	/// Coordinates are converted with (i / 255 * 2) -1
	pub fn bytes_to_vec3(x: u8, y: u8, z: u8) -> Vec3 {
		Vec3 {
			x: ((x as f32) / 255.0 * 2.0) - 1.0,
			y: ((y as f32) / 255.0 * 2.0) - 1.0,
			z: ((z as f32) / 255.0 * 2.0) - 1.0,
		}
	}

	/// Coordinates are converted with i / 2048
	pub fn words_to_vec3(x: u16, y: u16, z: u16) -> Vec3 {
		Vec3 {
			x: (x as f32) / 2048.0,
			y: (y as f32) / 2048.0,
			z: (z as f32) / 2048.0,
		}
	}

	#[cfg(test)]
	mod tests {
	}
}
