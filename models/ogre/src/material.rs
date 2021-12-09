pub mod ast {
	use bitflags::bitflags;

	bitflags! {
		pub struct PassFlag: u32 {
			const LIGHTING = 1;
			const DEPTH_WRITE = 2;
			const FOG_OVERRIDE = 4;
		}
	}

	bitflags! {
		pub struct MaterialFlag: u32 {
			const RECEIVE_SHADOWS = 1;
		}
	}

	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum Diffuse {

		VertexColour,
	}

	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum Filtering {
		None,
	}

	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum FogOverride {
		None,
	}

	#[derive(Clone, Debug, PartialEq)]
	pub struct Material {
		pub name: String,
		pub pass: Pass,
	}

	#[derive(Clone, Debug, PartialEq)]
	pub struct Pass {
		pub flags: PassFlag,
		pub blend: Option<SceneBlend>,
		pub diffuse: Option<Diffuse>,
		pub fog_override: Option<FogOverride>,
		pub depth_bias: Option<(f32, f32)>,
		pub tex_unit: Option<TextureUnit>,
	}

	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum SceneBlend {
		Add,
	}

	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum TexAddressMode {
		Clamp,
	}

	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum TexDimension {
		One,
	}

	#[derive(Clone, Debug, PartialEq)]
	pub struct TextureUnit {
		pub texture: (String, Option<TexDimension>),
		pub addr_mode: Option<TexAddressMode>,
		pub filtering: Option<Filtering>,
	}
}

#[cfg(feature = "import")]
pub mod import {
	use nom::{
		branch::alt,
		bytes::complete::{
			tag,
			take_while
		},
		character::
			complete::{
				multispace1
			},
			is_newline,
			is_space
		},
		combinator::value,
		error::{
			ErrorKind,
			ParseError
		},
		IResult,
		multi::many0
		Parser,
		sequence::preceded
	};

	use meshio_core::parser::{
		c_comment,
		curly,
		ws
	};

	#[derive(Error, Debug, PartialEq)]
	pub enum MaterialImportError<'a> {
		#[error("Misc import error")]
		Misc(&'a str, ErrorKind),
	}

	impl<'a> ParseError<&'a str> for MaterialImportError<'a> {
		fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
			MaterialImportError::Misc(input, kind)
		}

		fn append(_: &'a str, _: ErrorKind, other: Self) -> Self {
			other
		}
	}

	fn etc<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, MaterialImportError<'a>>
	where
		F: Parser<&'a str, O, MaterialImportError<'a>>,
	{
		preceded(
			many0(alt((
				c_comment,
				value((), multispace1)
			))),
			inner
		)
	}

	fn identifier<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, MaterialImportError<'a>> {
		take_while(move |c| !is_newline(c) || !is_space(c))(input)
	}

	fn false_true<'a>(input: &'a str) -> IResult<&'a str, bool, MaterialImportError<'a>> {
		alt((
            value(false, etc(tag("false"))),
            value(true, etc(tag("true"))),
        ))(input)
	}

	fn off_on<'a>(input: &'a str) -> IResult<&'a str, bool, MaterialImportError<'a>> {
		alt((
            value(false, etc(tag("off"))),
            value(true, etc(tag("on"))),
        ))(input)
	}


}
