use byteorder::{
	BE,
	LE,
	ReadBytesExt
};

use ultraviolet::vec::{
	Vec2,
	Vec3
};

use rgk_core::{
	io_ext::RedBinExt,
	tag4
};

use import::{
	QuakeImportError,
	to_vec2
};

pub const MAGIC: u32 = tag4!(b"IDPO");
pub const VERSION: u32 = 6;
pub const MAX_TRIANGLES: u32 = 2048;
pub const MAX_VERTICES: u32 = 1024;
pub const MAX_TEX_COORDS: u32 = 1024;
pub const MAX_FRAMES: u32 = 256;
pub const MAX_PRECALC_NORMALS: u32 = 162;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum SyncType {
	Synchronize = 0,
	Random,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub version: u32,
	pub scale: Vec3,
	pub translate: Vec3,
	pub bounding_radius: f32,
	pub eye_pos: Vec3,
	pub num_skins: u32,
	pub skin_width: u32,
	pub skin_height: u32,
	pub num_verts: u32,
	pub num_tris: u32,
	pub num_frames: u32,
	pub sync_type: SyncType,
	pub flags: u32, // ?
	pub size: f32,
}

impl Header {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<Header, QuakeImportError>
	where
		R: ReadBytesExt + ReadBinExt,
	{
		let magic = buf.read_u32::<BE>()?;
		if magic != MAGIC {
			return Err(QuakeImportError::Magic(magic));
		}

		let version = buf.read_u32::<LE>()?;
		if version != VERSION {
			return Err(QuakeImportError::Version(version));
		}

		let scale = buf.read_vec3_le()?;
		let trans = buf.read_vec3_le()?;
		let radius = buf.read_f32::<LE>()?;
		let eye_pos = buf.read_vec3_le()?;
		let nskins = buf.read_u32::<LE>()?;
		let skinw = buf.read_u32::<LE>()?;
		let skinh = buf.read_u32::<LE>()?;

		let nverts = buf.read_u32::<LE>()?;
		if nverts > MAX_VERTICES {
			return Err(QuakeImportError::MaxVertices(nverts));
		}

		let ntris = buf.read_u32::<LE>()?;
		if ntris > MAX_TRIANGLES {
			return Err(QuakeImportError::MaxTriangles(ntris));
		}

		let nframes = buf.read_u32::<LE>()?;
		if nframes > MAX_FRAMES {
			return Err(QuakeImportError::MaxFrames(nframes));
		}

		Ok(Header {
			magic: magic,
			version: version,
			scale: scale,
			translate: trans,
			bounding_radius: radius,
			eye_pos: eye_pos,
			num_skins: nskins,
			skin_width: skinw,
			skin_height: skinh,
			num_verts: nverts,
			num_tris: ntris,
			num_frames: nframes,
			sync_type: match buf.read_i32::<LE>()? {
				0 => SyncType::Synchronize,
				_ => SyncType::Random,
			},
			flags: buf.read_u32::<LE>()?,
			size: buf.read_f32::<LE>()?,
		})
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Skin {
	Single(Vec<u8>),
	Group {
		time: Vec<f32>,
		data: Vec<Vec<u8>>,
	},
}

impl Skin {
	#[cfg(feature = "import")]
	fn read<R>(width: usize, height: usize, buf: &mut R) -> Result<Skin, QuakeImportError>
	where
		R: ReadBytesExt,
	{
		let _is_group = buf.read_u32::<LE>()?;

		let mut data = vec![0; width * height];
		buf.read_exact(&mut data)?;

		Ok(Skin::Single(data))
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum TexCoordOnSeam {
	No = 0,
	Yes,
}

impl TexCoordOnSeam {
	#[cfg(feature = "import")]
	fn read<R>(buf: &mut R) -> Result<TexCoordOnSeam, QuakeImportError>
	where
		R: ReadBytesExt,
	{
		match buf.read_i32::<LE>()? {
			0 => Ok(TexCoordOnSeam::No),
			_ => Ok(TexCoordOnSeam::Yes),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TexCoord {
	pub on_seam: TexCoordOnSeam,
	pub coords: Vec2,
}

impl TexCoord {
	#[cfg(feature = "import")]
	fn read<R>(width: usize, height: usize, buf: &mut R) -> Result<TexCoord, QuakeImportError>
	where
		R: ReadBytesExt,
	{
		Ok(TexCoord {
			on_seam: TexCoordOnSeam::read(buf)?,
			coords: to_vec2(buf.read_i32::<LE>()?, buf.read_i32::<LE>()?,
				width as u32, height as u32),
		})
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum FaceOrient {
	Back = 0,
	Front,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
	pub orient: FaceOrient,
	pub indices: [u32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
	pub v: [u8; 3],
	pub normal_index: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleFrame {
	pub bb_min: Vertex,
	pub bb_max: Vertex,
	pub name: [u8; 16],
	pub verts: Vec<Vertex>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Frame {
	Single(SimpleFrame),
	Group {
		min: Vertex,
		max: Vertex,
		time: Vec<f32>,
		frames: Vec<SimpleFrame>,
	},
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuakeModel {
	pub header: Header,
	pub skins: Vec<Skin>,
	pub uvs: Vec<TexCoord>,
	pub tris: Vec<Triangle>,
	pub frames: Vec<Frame>,
	pub tex_ids: Vec<u32>,
}

impl Model {
	#[cfg(feature = "import")]
	pub fn read<R>(buf: &mut R) -> Result<Model, QuakeImportError>
	where
		R: ReadBytesExt + ReadBinExt,
	{
		let header = Header::read(buf)?;

		let mut skins = vec![];
		for _ in 0..(header.num_skins as usize) {
			skins.push(skin(header.skin_width as usize, header.skin_height as usize, buf)?);
		}

		let mut uvs = vec![];
		for _ in 0..(h.num_verts as usize) {
			uvs.push(texcoord(header.skin_width as usize, header.skin_height as usize, buf)?);
		}

		Ok(Model {
			header: header,
			skins: skins,
			uvs: uvs,
			tris: vec![],
			frames: vec![],
			tex_ids: vec![],
		})
	}
}

#[cfg(feature = "import")]
pub mod import {
	use byteorder::{
		BE,
		LE,
		ReadBytesExt
	};

	use std::io;
	use thiserror::Error;

	use rgk_core::io_ext::ReadBinExt;

	use super::*;

	#[derive(Debug, Error)]
	pub enum QuakeImportError {
		#[error("Unknown/unsupported grouping type: {0}")]
		GroupType(i32),
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Not an MDL file: {0}")]
		Magic(u32),
		#[error("Max number of allowed frames exceeded: {0}/256")]
		MaxFrames(u32),
		#[error("Max number of allowed precalculated normals exceeded: {0}/162")]
		MaxPrecalcNormals(u32),
		#[error("Max number of allowed texture coordinates exceeded: {0}/1024")]
		MaxTexCoords(u32),
		#[error("Max number of allowed triangles exceeded: {0}/2048")]
		MaxTriangles(u32),
		#[error("Max number of allowed vertices exceeded: {0}/1024")]
		MaxVertices(u32),
		#[error("Unknown/unsupported version: {0}")]
		Version(u32),
	}

	/// Obtains actual float coordinates, given width and height
	fn to_vec2(s: i32, t: i32, w: u32, h: u32) -> Vec2 {
		Vec2 {
			x: ((s as f32) + 0.5) / (w as f32),
			y: ((t as f32) + 0.5) / (h as f32),
		}
	}

	#[cfg(test)]
	mod tests {
		use std::fs::read;

		#[test]
		fn test_header() {
			let data = read("test_data/gijoe.mdl").unwrap();
			println!("{:#?}", super::QuakeModel::read(&mut data.as_slice()).unwrap());
		}
	}
}
