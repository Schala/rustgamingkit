use byteorder::{
	LE,
	ReadBytesExt
};

use std::{
	fmt::{
		Display,
		Formatter,
		self
	},
	io
};

use thiserror::Error;
use ultraviolet::vec::Vec4;

use import::RipImportError;

pub const MAGIC: u32 = 0xDEADC0DE;
pub const VERSION: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttrFormat {
	Float,
	Unsigned,
	Signed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttrData {
	Vertex(Vec4),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
	pub semantic: String,
	pub semantic_index: u32,
	pub offset: u32,
	pub size: u32,
	pub num_items: u32,
	pub format: Vec<AttrFormat>,
	pub data: Vec<AttrData>,
}

impl Attribute {
	/// Parse the data buffer into a [`Vec4`]
	fn parse_vertex<R>(&mut self, buf: &mut R) -> Result<(), RipImportError>
	where
		R: ReadBytesExt,
	{
		let mut verts = vec![];
		for _ in 0..(self.num_items as usize) {
			verts.push(buf.read_f32::<LE>()?);
		}

		match self.num_items {
			3 => self.data.push(AttrData::Vertex(Vec4::new(verts[0], verts[1], verts[2], 1.0))),
			4 => self.data.push(AttrData::Vertex(Vec4::new(verts[0], verts[1], verts[2], verts[3]))),
			_ => return Err(RipImportError::VertexSize(self.num_items)),
		}

		Ok(())
	}

	/// Return the attribute format represented as a string
	pub fn format_str(&self) -> String {
		let mut s = String::new();

		for i in self.format.iter() {
			match *i {
				AttrFormat::Float => s.push('f'),
				AttrFormat::Unsigned => s.push('I'),
				AttrFormat::Signed => s.push('i'),
			}
		}

		s
	}
}

impl Display for Attribute {
	/// Display the attribute's hashtag
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "[{}:{}:{}:{}:{}]", self.semantic, self.semantic_index, self.offset,
			self.size, self.format_str())
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub version: u32,
	pub num_faces: u32,
	pub num_verts: u32,
	pub block_size: u32,
	pub num_texs: u32,
	pub num_shaders: u32,
	pub num_attrs: u32
}

pub type Face = [usize; 3];

#[derive(Clone, Debug, PartialEq)]
pub struct RipModel {
	pub header: Header,
	pub hash: Vec<u8>,
	pub attributes: Vec<Attribute>,
	pub textures: Vec<String>,
	pub shaders: Vec<String>,
	pub faces: Vec<Face>,
}

impl RipModel {
	/// Returns a hexadecimal string representation of the hash
	pub fn str_hash(&self) -> String {
		let mut s = String::new();

		for b in self.hash.iter() {
			s.push_str(format!("{:x}", *b).as_str());
		}
		s
	}

	/// If present, returns the attribute with the specified semantic
	pub fn get_attr(&self, semantic: &str, occurence: usize) -> Option<&Attribute> {
		let mut nth = 0usize;
		for attr in self.attributes.iter() {
			nth += 1;
			if (nth == occurence) && (&attr.semantic[..] == semantic) {
				return Some(attr);
			}
		}

		None
	}
}

#[cfg(feature = "import")]
pub mod import {
	use byteorder::{
		LE,
		ReadBytesExt
	};

	use sha1::{
		Digest,
		Sha1
	};

	use meshio_core::io_ext::ReadBinExt;
	use super::*;

	#[derive(Error, Debug)]
	pub enum RipImportError {
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Not a Ninja Ripper model file: {0}")]
		Magic(u32),
		#[error("Unknown/unsupported format version: {0}")]
		Version(u32),
		#[error("Unknown/unsupported vertex size: {0}")]
		VertexSize(u32),
	}

	fn header<R>(buf: &mut R) -> Result<Header, RipImportError>
	where
		R: ReadBytesExt,
	{
		let magic = buf.read_u32::<LE>()?;
		if magic != MAGIC {
			return Err(RipImportError::Magic(magic));
		}

		let version = buf.read_u32::<LE>()?;
		if version != VERSION {
			return Err(RipImportError::Version(version));
		}

		Ok(Header {
			magic: magic,
			version: version,
			num_faces: buf.read_u32::<LE>()?,
			num_verts: buf.read_u32::<LE>()?,
			block_size: buf.read_u32::<LE>()?,
			num_texs: buf.read_u32::<LE>()?,
			num_shaders: buf.read_u32::<LE>()?,
			num_attrs: buf.read_u32::<LE>()?,
		})
	}

	fn attribute<R>(buf: &mut R) -> Result<Attribute, RipImportError>
	where
		R: ReadBytesExt + ReadBinExt,
	{
		let sem = buf.read_cstr()?;
		let sem_idx = buf.read_u32::<LE>()?;
		let offs = buf.read_u32::<LE>()?;
		let size = buf.read_u32::<LE>()?;
		let nitems = buf.read_u32::<LE>()?;
		let mut fmt = vec![];

		for _i in 0..(nitems as usize) {
			fmt.push(match buf.read_u32::<LE>()? {
				0 => AttrFormat::Float,
				1 => AttrFormat::Unsigned,
				_ => AttrFormat::Signed,
			});
		}

		Ok(Attribute {
			semantic: sem,
			semantic_index: sem_idx,
			offset: offs,
			size: size,
			num_items: nitems,
			format: fmt,
			data: vec![],
		})
	}

	pub fn rip<R>(buf: &mut R) -> Result<Model, RipImportError>
	where
		R: ReadBytesExt + ReadBinExt,
	{
		let hdr = header(buf)?;
		let mut hasher = Sha1::new();

		// attributes
		let mut attrs = vec![];
		for _ in 0..(hdr.num_attrs as usize) {
			let attr = attribute(buf)?;
			hasher.update(attr.to_string().as_bytes());
			attrs.push(attr);
		}

		// textures
		let mut texs = vec![];
		for _ in 0..(hdr.num_texs as usize) {
			texs.push(buf.read_cstr()?);
		}

		// shaders
		let mut shaders = vec![];
		for _ in 0..(hdr.num_shaders as usize) {
			shaders.push(buf.read_cstr()?);
		}

		// faces
		let mut face_indices = vec![];
		for _ in 0..(hdr.num_faces as usize) {
			let mut raw_face = [0; 12];
			buf.read_exact(&mut raw_face)?;

			let facebuf = &mut raw_face.as_slice();
			let faces = [facebuf.read_u32::<LE>()? as usize, facebuf.read_u32::<LE>()? as usize,
				facebuf.read_u32::<LE>()? as usize];

			// Omit degenerate triangles - they are sometimes used to merge strips
			if faces[0] != faces[1] && faces[1] != faces[2] && faces[0] != faces[2] {
				hasher.update(facebuf);
				face_indices.push(faces);
			}
		}
		hasher.update(&b"|"[..]);

		// vertices
		for _ in 0..(hdr.num_verts as usize) {
			let mut raw_vert = vec![0; hdr.block_size as usize];
			buf.read_exact(&mut raw_vert)?;
			hasher.update(&raw_vert[..]);

			for attr in attrs.iter_mut() {
				let _ = attr.parse_vertex(&mut raw_vert.as_slice())?;
			}
		}

		Ok(Model {
			header: hdr,
			hash: hasher.finalize().to_vec(),
			attributes: attrs,
			textures: texs,
			shaders: shaders,
			faces: face_indices,
		})
	}

	#[cfg(test)]
	mod tests {
		use std::fs::read;

		#[test]
		fn test_rip() {
			let data = read("test_data/Mesh_0015.rip").unwrap();
			println!("{:#?}", super::rip(&mut data.as_slice()).unwrap());
		}
	}
}
