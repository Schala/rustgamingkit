pub mod ast {
	use ultraviolet::vec::Vec3;

	/// Material attribute's format
	#[derive(Debug, PartialEq)]
	pub enum MtlFormat {
		/// Red-Green-Blue color value, default, if no format is explicitly given
		Rgb(Vec3),
		Spectral(String),
		/// 3D vector coordinate
		Xyz(Vec3),
	}

	/// Material texture option
	#[derive(Debug, PartialEq)]
	pub enum MtlTexOp {
		BlendU(bool),
		BlendV(bool),
		Boost(f32),
		Mm(f32, f32),
		//Origin(
	}

	/// Material reflection type
	#[derive(Debug, PartialEq)]
	pub enum MtlReflection {
		CubeTop,
		CubeFront,
		CubeLeft,
		CubeBottom,
		CubeBack,
		CubeRight,
		Sphere,
	}
}

#[cfg(feature = "import")]
pub mod import {
	use nom::{
		branch::alt,
		bytes::complete::tag,
		character::complete::{
			char,
			u32
		},
		combinator::map,
		error::{
			ErrorKind,
			ParseError
		},
		IResult,
		number::complete::float,
		sequence::preceded
	};

	use thiserror::Error;
	use ultraviolet::vec::Vec3;

	use crate::{
		etc,
		identifier,
		uvw
	};

	use meshio_core::parser::vec3ws;
	use super::ast::*;

	#[derive(Error, Debug, PartialEq)]
	pub enum MtlImportError {
		#[error("Unknown import error")]
		Unknown,
	}

	impl<I> ParseError<I> for MtlImportError {
		fn from_error_kind(_: I, _: ErrorKind) -> Self {
			MtlImportError::Unknown
		}

		fn append(_: I, _: ErrorKind, other: Self) -> Self {
			other
		}
	}

	/// Parses a ['MtlFormat']
	fn fmt(input: &str) -> IResult<&str, MtlFormat> {
		alt((
			map(preceded(etc(tag("spectral")), identifier), |s| MtlFormat::Spectral(s.to_string())),
			map(preceded(etc(tag("xyz")), vec3ws), |v| MtlFormat::Xyz(v)),
			map(vec3ws, |v| MtlFormat::Rgb(v))
		))(input)
	}

	/// Parses a filename used for the material's bump map (`bump` attribute)
	fn bump<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
        preceded(etc(tag("bump")), etc(identifier))(input)
    }

    fn d(input: &str) -> IResult<&str, f32> {
        preceded(etc(char('d')), etc(float))(input)
    }

	/// Parses a filename used for the material's decal
	fn decal<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
        preceded(etc(tag("decal")), etc(identifier))(input)
    }

	/// Parses a filename used for the material's displacement map
	fn disp<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
        preceded(etc(tag("disp")), etc(identifier))(input)
    }

    fn illum(input: &str) -> IResult<&str, u32> {
        preceded(etc(tag("illum")), etc(u32))(input)
    }

    fn ka(input: &str) -> IResult<&str, Vec3> {
        preceded(etc(tag("Ka")), etc(vec3ws))(input)
    }

	fn kd(input: &str) -> IResult<&str, Vec3> {
        preceded(etc(tag("Kd")), etc(vec3ws))(input)
    }

	fn km(input: &str) -> IResult<&str, Vec3> {
        preceded(etc(tag("Km")), etc(vec3ws))(input)
    }

	fn ks(input: &str) -> IResult<&str, Vec3> {
        preceded(etc(tag("Ks")), etc(vec3ws))(input)
    }

	/// Parses a filename used for the material's bump map (`map_bump` attribute)
	fn map_bump<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
        preceded(etc(tag("map_bump")), etc(identifier))(input)
    }

	/// Parses a filename used for the material's diffuse map
	fn map_d<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
        preceded(etc(tag("map_d")), etc(identifier))(input)
    }

	fn map_kd<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
        preceded(etc(tag("map_Kd")), etc(identifier))(input)
    }

	/// Parses a material's name
	fn newmtl<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
        preceded(etc(tag("newmtl")), etc(identifier))(input)
    }

	fn ni(input: &str) -> IResult<&str, f32> {
        preceded(etc(tag("Ni")), etc(float))(input)
    }

	fn ns(input: &str) -> IResult<&str, f32> {
        preceded(etc(tag("Ns")), etc(float))(input)
    }

	fn tf(input: &str) -> IResult<&str, MtlFormat> {
        preceded(etc(tag("Tf")), etc(fmt))(input)
    }

	fn tr(input: &str) -> IResult<&str, f32> {
        preceded(etc(tag("Tr")), etc(float))(input)
    }

	#[cfg(test)]
	mod tests {
	}
}

