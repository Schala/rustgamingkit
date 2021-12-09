use byteorder::{
	BE,
	ByteOrder,
	LE,
	NativeEndian,
	ReadBytesExt
};

use std::io::{
	Seek,
	SeekFrom,
};

use ultraviolet::vec::Vec3;

use rgk_core::{
	io_ext::ReadBinExt,
	tag4
};

use import::NepImportError;

pub const MAGIC: u32 = tag4!(b"ISM2");

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum Precision {
	Half = 5,
	Single = 7,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum SectionType {
	Armature = 3,
	VertexMetaHeaders = 10,
	ObjectMesh,
	BoneMatrixTranslation = 20,
	BoneScale,
	Strings = 33,
	Textures = 46,
	BoneMatrices = 50,
	Animations = 52,
	Polygons = 69,
	PolygonBlock,
	SurfaceOffsets = 76,
	VertexBlocks = 89,
	BoneTransforms = 91,
	BoneParenting,
	BoneMatrixX,
	BoneMatrixY,
	BoneMatrixZ,
	Materials = 97,
	FXShaderNodes,
	JointOrientX = 103,
	JointOrientY,
	JointOrientZ,
	BoundingBox = 110,
	BoneCollision = 112,
	BoneCollisionRadius,
	BonePhysics,
	BonePhysicsRadius,
	BonePhysicsCost,
	BonePhysicsMass,
	BonePhysicsExpand,
	BonePhysicsShapeMemory,
	Unknown = u32::MAX,
}

impl SectionType {
	#[cfg(feature = "import")]
	fn read<B, R>(buf: &mut R) -> Result<SectionType, NepImportError>
	where
		B: ByteOrder,
		R: ReadBytesExt,
	{
		match buf.read_u32::<B>()? {
			3 => Ok(SectionType::Armature),
			10 => Ok(SectionType::VertexMetaHeaders),
			11 => Ok(SectionType::ObjectMesh),
			20 => Ok(SectionType::BoneMatrixTranslation),
			21 => Ok(SectionType::BoneScale),
			33 => Ok(SectionType::Strings),
			46 => Ok(SectionType::Textures),
			50 => Ok(SectionType::BoneMatrices),
			52 => Ok(SectionType::Animations),
			69 => Ok(SectionType::Polygons),
			70 => Ok(SectionType::PolygonBlock),
			76 => Ok(SectionType::SurfaceOffsets),
			89 => Ok(SectionType::VertexBlocks),
			91 => Ok(SectionType::BoneTransforms),
			92 => Ok(SectionType::BoneParenting),
			93 => Ok(SectionType::BoneMatrixX),
			94 => Ok(SectionType::BoneMatrixY),
			95 => Ok(SectionType::BoneMatrixZ),
			97 => Ok(SectionType::Materials),
			98 => Ok(SectionType::FXShaderNodes),
			103 => Ok(SectionType::JointOrientX),
			104 => Ok(SectionType::JointOrientY),
			105 => Ok(SectionType::JointOrientZ),
			110 => Ok(SectionType::BoundingBox),
			112 => Ok(SectionType::BoneCollision),
			113 => Ok(SectionType::BoneCollisionRadius),
			114 => Ok(SectionType::BonePhysics),
			115 => Ok(SectionType::BonePhysicsRadius),
			116 => Ok(SectionType::BonePhysicsCost),
			117 => Ok(SectionType::BonePhysicsMass),
			118 => Ok(SectionType::BonePhysicsExpand),
			119 => Ok(SectionType::BonePhysicsShapeMemory),
			_ => Ok(SectionType::Unknown),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SectionHeader {
	pub id: SectionType,
	pub offset: u32,
}

impl SectionHeader {
	#[cfg(feature = "import")]
	fn read<B, R>(buf: &mut R) -> Result<SectionHeader, NepImportError>
	where
		B: ByteOrder,
		R: ReadBytesExt,
	{
		Ok(SectionHeader {
			id: SectionType::read::<B, R>(buf)?,
			offset: buf.read_u32::<B>()?,
		})
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StringHeader {
	pub id: SectionType,
	pub header_size: u32,
	pub num_strings: u32,
}

impl StringHeader {
	#[cfg(feature = "import")]
	fn read<B, R>(buf: &mut R) -> Result<StringHeader, NepImportError>
	where
		B: ByteOrder,
		R: ReadBytesExt,
	{
		Ok(StringHeader {
			id: SectionType::read::<B, R>(buf)?,
			header_size: buf.read_u32::<B>()?,
			num_strings: buf.read_u32::<B>()?,
		})
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum SectionData {
	Strings(StringHeader, Vec<u32>, Vec<String>),
	Unknown,
}

impl SectionData {
	#[cfg(feature = "import")]
	fn read<'a, 'b, B, R>(header: &'a SectionHeader, buf: &'b mut R) -> Result<SectionData, NepImportError>
	where
		B: ByteOrder,
		R: ReadBytesExt + ReadBinExt + Seek,
	{
		match header.id {
			SectionType::Strings => {
				let str_h = StringHeader::read::<B, R>(buf)?;

				let mut offsets = vec![];
				for _ in 0..(str_h.num_strings as usize) {
					offsets.push(buf.read_u32::<B>()?);
				}

				let mut strings = vec![];
				for i in 0..(str_h.num_strings as usize) {
					buf.seek(SeekFrom::Start(offsets[i] as u64))?;
					strings.push(buf.read_cstr()?);
				}

				Ok(SectionData::Strings(str_h, offsets, strings))
			},
			_ => Ok(SectionData::Unknown),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
	pub header: SectionHeader,
	pub data: SectionData,
}

impl Section {
	#[cfg(feature = "import")]
	fn from_input(header: SectionHeader, data: SectionData) -> Section {
		Section {
			header: header,
			data: data,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum VertexType {
	PosNormUNorm2VColor = 1,
	BonesWeights = 3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	pub magic: u32,
	pub version: u32,
	pub unknown_08: u32,
	pub unknown_0c: u32,
	pub size: u32,
	pub num_sections: u32,
	pub unknown_18: u32,
	pub unknown_1c: u32,
}

impl Header {
	#[cfg(feature = "import")]
	fn read<B, R>(buf: &mut R) -> Result<Header, NepImportError>
	where
		B: ByteOrder,
		R: ReadBytesExt,
	{
		let magic = buf.read_u32::<BE>()?;
		if magic != MAGIC {
			return Err(NepImportError::Magic(magic));
		}

		Ok(Header {
			magic: magic,
			version: buf.read_u32::<B>()?,
			unknown_08: buf.read_u32::<B>()?,
			unknown_0c: buf.read_u32::<B>()?,
			size: buf.read_u32::<B>()?,
			num_sections: buf.read_u32::<B>()?,
			unknown_18: buf.read_u32::<B>()?,
			unknown_1c: buf.read_u32::<B>()?,
		})
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct NepModel {
	pub header: Header,
	pub sections: Vec<Section>,
}

impl NepModel {
	#[cfg(feature = "import")]
	pub fn read<R>(buf: &mut R) -> Result<NepModel, NepImportError>
	where
		R: ReadBytesExt + ReadBinExt + Seek,
	{
		// endian check
		buf.seek(SeekFrom::Start(24))?;
		let check = buf.read_i32::<NativeEndian>()?;
		buf.seek(SeekFrom::Start(0))?;

		let header: Header;
		let sections: Vec<Section>;

		if check >= 0 && check < 65536 {
			// little endian

			header = Header::read::<LE, R>(buf)?;

			// sections
			let mut section_headers = vec![];
			for _ in 0..(header.num_sections as usize) {
				section_headers.push(SectionHeader::read::<LE, R>(buf)?);
			}
			let mut section_data = vec![];
			for i in 0..(header.num_sections as usize) {
				buf.seek(SeekFrom::Start(section_headers[i].offset as u64))?;
				section_data.push(SectionData::read::<LE, R>(&section_headers[i], buf)?);
			}
			sections = section_headers.drain(..).zip(section_data.drain(..))
				.map(|(h, d)| Section::from_input(h, d)).collect();
		} else {
			// big endian

			header = Header::read::<BE, R>(buf)?;
			println!("{:#?}", header);

			// sections
			let mut section_headers = vec![];
			for _ in 0..(header.num_sections as usize) {
				section_headers.push(SectionHeader::read::<BE, R>(buf)?);
			}
			let mut section_data = vec![];
			for i in 0..(header.num_sections as usize) {
				buf.seek(SeekFrom::Start(section_headers[i].offset as u64))?;
				section_data.push(SectionData::read::<BE, R>(&section_headers[i], buf)?);
			}
			sections = section_headers.drain(..).zip(section_data.drain(..))
				.map(|(h, d)| Section::from_input(h, d)).collect();
		}

		Ok(NepModel {
			header: header,
			sections: sections,
		})
	}
}

#[cfg(feature = "import")]
pub mod import {
	use std::io;
	use thiserror::Error;

	#[derive(Error, Debug)]
	pub enum NepImportError {
		#[error("Bone index out of bounds: {0}")]
		BoneIndex(u32),
		#[error("I/O error")]
		IO {
			#[from]
			source: io::Error,
		},
		#[error("Not an ISM2 model file: {0:X}")]
		Magic(u32),
		/*#[error("Unknown/unsupported section type: {0}")]
		Section(u32),
		#[error("String table index out of bounds: {0}")]
		StringIndex(u32),*/
	}

	fn u16_to_f32(input: u16) -> f32 {
		let sign = (input & 0x8000) as u32;
		let exp = (((input & 0x7C00) >> 10) as u32) - 16;
		let frac = (input & 0x3FF) as u32;

		let sign = if sign != 0 { 1 } else { 0 };
		let exp = exp + 127;
		let res = ((frac << 13) | (exp << 23)) | (sign << 31);

		// fix rounding down to 0 properly for rigging
		if res == 931135488 {
			return 0.0;
		}

		f32::from_bits(res)
	}

	#[cfg(test)]
	mod tests {
		use std::{
			fs::read,
			io::Cursor
		};

		#[test]
		fn test_ism2() {
			let mut data = Cursor::new(read("test_data/model.ism2").unwrap());
			println!("{:#?}", super::super::NepModel::read(&mut data).unwrap());
		}
	}
}
