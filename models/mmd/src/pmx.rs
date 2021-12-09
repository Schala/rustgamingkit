use bitflags::bitflags;

use byteorder::{
	BE,
	LE,
	ReadBytesExt
};

use rgk_core::tag4;

use import::{
	index,
	MMD2ImportError,
	text
};

const MAGIC: u32 = tag4!(b"PMX ");

bitflags! {
	pub struct MaterialFlags: u8 {
		const NO_CULL = 1;
		const GROUND_SHADOW = 2;
		const DRAW_SHADOW = 4;
		const RECEIVE_SHADOW = 8;
		const HAS_EDGE = 16;
		const VERTEX_COLOR = 32;
		const POINT_DRAWING = 64;
		const LINE_DRAWING = 128;
	}

	pub struct BoneFlags: u16 {
		const INDEXED_TAIL_POS = 1;
		const ROTATABLE = 2;
		const TRANSLATABLE = 4;
		const VISIBLE = 8;
		const ENABLED = 16;
		const IK = 32;
		const INHERIT_ROTATION = 64;
		const INHERIT_TRANSLATION = 128;
		const FIXED_AXIS = 256;
		const LOCAL_COORD = 512;
		const PHYSICS_AFTER_DEFORM = 1024;
		const EXTERNAL_PARENT_DEFORM = 2048;
	}

	pub struct SoftBodyFlags: u8 {
		const B_LINK = 1;
		const CLUSTER_CREATION = 2;
		const LINK_CROSSING = 4;
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum AerodynamicsModel {
	VPoint = 0,
	V2Sided,
	V1Sided,
	F2Sided,
	F1Sided,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum BlendMode {
	Disabled = 0,
	Multiply,
	Additive,
	AdditionalVec4,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Encoding {
	UTF16LE = 0,
	UTF8,
	Unknown = 255,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Frame {
	Bone(i32),
	Morph(i32),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Globals {
	pub encoding: Encoding,
	pub additional_vec4_count: u8,
	pub vertex_index_size: u8,
	pub texture_index_size: u8,
	pub material_index_size: u8,
	pub bone_index_size: u8,
	pub morph_index_size: u8,
	pub rb_index_size: u8,
}

impl Globals {
	#[cfg(feature = "import")]
	fn read<R>(encoding: Encoding, buf: &mut R) -> Result<Globals, MMD2ImportError>
	where
		R: ReadBytesExt,
	{
		Ok(Globals {
			encoding: encoding,
			additional_vec4_count: buf.read_u8()?,
			vertex_index_size: buf.read_u8()?,
			texture_index_size: buf.read_u8()?,
			material_index_size: buf.read_u8()?,
			bone_index_size: buf.read_u8()?,
			morph_index_size: buf.read_u8()?,
			rb_index_size: buf.read_u8()?,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub version: f32,
	pub globals_count: u8,
	pub globals: Globals,
	pub name_local: String,
	pub name_universal: String,
	pub comment_local: String,
	pub comment_universal: String,
}

impl Header {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Header, MMD2ImportError>
	where
		R: ReadBytesExt,
	{
		let magic = buf.read_u32::<BE>()?;
		if magic != MAGIC {
			return Err(MMD2ImportError::Magic(magic));
		}

		let version = buf.read_f32::<LE>()?;

		let gcount = buf.read_u8()?;
		if gcount != 8 {
			// Without knowing how many globals there are, we risk parsing the wrong data
			return Err(MMD2ImportError::GlobalsCount(gcount));
		}

		let enc = match buf.read_u8()? {
			0 => Encoding::UTF16LE,
			1 => Encoding::UTF8,
			_ => Encoding::Unknown,
		};

		Ok(Header {
			magic: magic,
			version: version,
			globals_count: gcount,
			globals: Globals::read(enc, buf)?,
			name_local: text(enc, buf)?,
			name_universal: text(enc, buf)?,
			comment_local: text(enc, buf)?,
			comment_universal: text(enc, buf)?,
		})
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum JointType {
	Spring6DoF = 0,
	SixDoF,
	P2P,
	ConeTwist,
	Slider,
	Hinge,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum PhysicsMode {
	FollowBone = 0,
	Physics,
	PhysicsAndBone,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum RigidBodyShapeType {
	Sphere = 0,
	Box,
	Capsule,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum SoftBodyShapeType {
	TriMesh = 0,
	Rope,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ToonRef {
	Texture = 0,
	Internal,
}

#[cfg(feature = "import")]
pub mod import {
	use byteorder::{
		LE,
		ReadBytesExt
	};

	use encoding_rs::{
		UTF_16LE,
		UTF_8
	};

	use std::io;
	use thiserror::Error;

	use super::*;

	#[derive(Error, Debug)]
	pub enum MMD2ImportError {
		#[error("Bone index out of bounds: {0}")]
		BoneIndex(i32),
		#[error("Invalid/unsupported encoding: {0}")]
		Encoding(u8),
		#[error("Only a globals count of 8 is supported at this time. Found {0}")]
		GlobalsCount(u8),
		#[error("Not a valid index size (must be 1, 2, or 4): {0}")]
		IndexSize(u8),
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Not a PMX file: {0:X}")]
		Magic(u32),
		#[error("Material index out of bounds: {0}")]
		MaterialIndex(i32),
		#[error("Morph index out of bounds: {0}")]
		MorphIndex(i32),
		#[error("Rigidbody index out of bounds: {0}")]
		RigidbodyIndex(i32),
		#[error("Texture index out of bounds: {0}")]
		TextureIndex(i32),
		#[error("Vertex index out of bounds: {0}")]
		VertexIndex(i32),
	}

	/// Parses an index. This will always be a 32-bit unsigned integer result.
	pub fn index<R>(count: u8, buf: &mut R) -> Result<u32, MMD2ImportError>
	where
		R: ReadBytesExt,
	{
		match count {
			1 => Ok(buf.read_u8()? as u32),
			2 => Ok(buf.read_u16::<LE>()? as u32),
			4 => Ok(buf.read_u32::<LE>()?),
			_ => Err(MMD2ImportError::IndexSize(count)),
		}
	}

	/// Parses and decodes text, pending on the given encoding
	pub fn text<R>(encoding: Encoding, buf: &mut R) -> Result<String, MMD2ImportError>
	where
		R: ReadBytesExt,
	{
		let length = buf.read_u32::<LE>()? as usize;
		let mut raw = vec![];
		let mut chr = [0; 1];

		for _ in 0..length {
			buf.read_exact(&mut chr)?;
			raw.push(chr[0]);
		}

		match encoding {
			Encoding::UTF16LE => {
				let mut dec = UTF_16LE.new_decoder_without_bom_handling();
				let mut output = String::with_capacity(length);
				let _ = dec.decode_to_string(raw.as_slice(), &mut output, true);
				output.shrink_to_fit();
				Ok(output)
			},
			Encoding::UTF8 => {
				let mut dec = UTF_8.new_decoder_without_bom_handling();
				let mut output = String::with_capacity(length);
				let _ = dec.decode_to_string(raw.as_slice(), &mut output, true);
				Ok(output)
			},
			_ => Err(MMD2ImportError::Encoding(encoding as u8)),
		}
	}

	#[cfg(test)]
	mod tests {
		use std::fs::read;

		#[test]
		fn test_header() {
			let data = read("test_data/Mona.Pmx").unwrap();
			println!("{:#?}", super::super::Header::read(&mut data.as_slice()).unwrap());
		}
	}
}
