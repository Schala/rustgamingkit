pub mod ast {
	use bitflags::bitflags;
	use std::collections::HashMap;
	use ultraviolet::vec::Vec3;

	#[cfg(feature = "import")]
	use super::import::ASEImportError;

	bitflags! {
		/// Mesh properties
		pub struct Properties: u32 {
			const MOTION_BLUR = 1;
			const CAST_SHADOW = 2;
			const RECV_SHADOW = 4;
		}
	}

	/// Bitmap filter type
	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum BitmapFilter {
		None,
		Pyramidal,
	}

	impl BitmapFilter {
		#[cfg(feature = "import")]
		fn parse<'a>(input: &'a str) -> nom::IResult<&'a str, BitmapFilter, ImportError<'a>> {
			use nom::{
				branch::alt,
				bytes::complete::tag,
				combinator::value,
				sequence::preceded
			};

			use meshio_core::nom_ext::ws;

			preceded(ws(tag("*BITMAP_FILTER")), alt((
				value(BitmapFilter::None, ws(tag("None"))),
				value(BitmapFilter::Pyramidal, ws(tag("Pyramidal")))
			)))(input)
		}
	}

	/// Falloff exponent
	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum Exponent {
		Filter,
	}

	impl Exponent {
		#[cfg(feature = "import")]
		fn parse<'a>(input: &'a str) -> nom::IResult<&'a str, Exponent, ImportError<'a>> {
			use nom::{
				branch::alt,
				bytes::complete::tag,
				combinator::value,
				sequence::preceded
			};

			use meshio_core::nom_ext::ws;

			preceded(ws(tag("*MATERIAL_XP_TYPE")), alt((
				value(Exponent::Filter, ws(tag("Filter")))
			)))(input)
		}
	}

	/// Material falloff type
	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum Falloff {
		In,
	}

	impl Falloff {
		#[cfg(feature = "import")]
		fn parse<'a>(input: &'a str) -> nom::IResult<&'a str, Falloff, ImportError<'a>> {
			use nom::{
				branch::alt,
				bytes::complete::tag,
				combinator::value,
				sequence::preceded
			};

			use meshio_core::nom_ext::ws;

			preceded(ws(tag("*MATERIAL_FALLOFF")), alt((
				value(Falloff::In, ws(tag("In")))
			)))(input)
		}
	}

	#[derive(Clone, Debug, PartialEq)]
	pub struct Map {
		pub name: String,
		pub class: String,
		pub sub_no: u32,
		pub amount: f32,
		pub bitmap: String,
		pub kind: MapType,
		pub u_offset: f32,
		pub v_offset: f32,
		pub u_tiling: f32,
		pub v_tiling: f32,
		pub angle: f32,
		pub blur: f32,
		pub blur_offset: f32,
		pub noise_amt: f32,
		pub noise_size: f32,
		pub noise_level: u32,
		pub noise_phase: f32,
		pub bitmap_filter: BitmapFilter,
	}

	impl Map {
		#[cfg(feature = "import")]
		fn parse<'a>(input: &'a str) -> nom::IResult<&'a str, Map, ImportError<'a>> {
			use nom::{
				branch::permutation,
				bytes::complete::tag,
				character::complete::{
					i32,
					u32
				},
				number::complete::float,
				sequence::preceded
			};

			use meshio_core::nom_ext::{
				curly,
				double_quoted,
				ws
			};

			let (input, data) = curly(permutation((
				preceded(ws(tag("*MAP_NAME")), ws(double_quoted)),
				preceded(ws(tag("*MAP_CLASS")), ws(double_quoted)),
				preceded(ws(tag("*MAP_SUBNO")), ws(u32)),
				preceded(ws(tag("*MAP_AMOUNT")), ws(float)),
				preceded(ws(tag("*BITMAP")), ws(double_quoted)),
				MapType::parse,
				preceded(ws(tag("*UVW_U_OFFSET")), ws(float)),
				preceded(ws(tag("*UVW_V_OFFSET")), ws(float)),
				preceded(ws(tag("*UVW_U_TILING")), ws(float)),
				preceded(ws(tag("*UVW_V_TILING")), ws(float)),
				preceded(ws(tag("*UVW_ANGLE")), ws(float)),
				preceded(ws(tag("*UVW_BLUR")), ws(float)),
				preceded(ws(tag("*UVW_BLUR_OFFSET")), ws(float)),
				preceded(ws(tag("*UVW_NOISE_AMT")), ws(float)),
				preceded(ws(tag("*UVW_NOISE_SIZE")), ws(float)),
				preceded(ws(tag("*UVW_NOISE_LEVEL")), ws(i32)),
				preceded(ws(tag("*UVW_NOISE_PHASE")), ws(float)),
				BitmapFilter::parse
			)))(input)?;

			Ok((input, Map {
				name: data.0.to_string(),
				class: data.1.to_string(),
				sub_no: data.2,
				amount: data.3,
				bitmap: data.4.to_string(),
				kind: data.5,
				u_offset: data.6,
				v_offset: data.7,
				u_tiling: data.8,
				v_tiling: data.9,
				angle: data.10,
				blur: data.11,
				blur_offset: data.12,
				noise_amt: data.13,
				noise_size: data.14,
				noise_level: data.15,
				noise_phase: data.16,
				bitmap_filter: data.17,
			}))
		}
	}

	#[derive(Clone, Copy, Debug, PartialEq, Eq)]
	pub enum MapKey {
		Diffuse,
	}

	pub type MapPair = (MapKey, Map);
	pub type Maps = HashMap<MapKey, Map>;

	/// Material map type
	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum MapType {
		Screen,
	}

	impl MapType {
		#[cfg(feature = "import")]
		fn parse<'a>(input: &'a str) -> nom::IResult<&'a str, MapType, ImportError<'a>> {
			use nom::{
				branch::alt,
				bytes::complete::tag,
				combinator::value,
				sequence::preceded
			};

			use meshio_core::nom_ext::ws;

			preceded(ws(tag("*MAP_TYPE")), alt((
				value(MapType::Screen, ws(tag("Screen")))
			)))(input)
		}
	}

	#[derive(Clone, Debug)]
	pub struct Material {
		pub name: String,
		pub class: String,
		pub ambient: Vec3,
		pub diffuse: Vec3,
		pub specular: Vec3,
		pub shine: f32,
		pub shine_strength: f32,
		pub transparency: f32,
		pub wire_size: f32,
		pub shading: Shading,
		pub xp_falloff: f32,
		pub self_illum: f32,
		pub falloff: Falloff,
		pub xp: Exponent,
		pub maps: Maps,
	}

	impl Material {
		#[cfg(feature = "import")]
		fn parse<'a>(input: &'a str) -> nom::IResult<&'a str, Material, ImportError<'a>> {
			use nom::{
				branch::{
					alt,
					permutation
				},
				bytes::complete::tag,
				character::complete::u32,
				combinator::opt,
				sequence::{
					preceded,
					tuple
				}
			};

			use std::mem::size_of;

			use meshio_core::nom_ext::{
				curly,
				double_quoted,
				vec3ws,
				ws
			};

			let (input, data) = preceded(tuple((ws(tag("*MATERIAL")), ws(u32))),
				curly(permutation((
					preceded(ws(tag("*MATERIAL_NAME")), ws(double_quoted)),
					preceded(ws(tag("*MATERIAL_CLASS")), ws(double_quoted)),
					preceded(ws(tag("*MATERIAL_AMBIENT")), ws(vec3ws)),
					preceded(ws(tag("*MATERIAL_DIFFUSE")), ws(vec3ws)),
					preceded(ws(tag("*MATERIAL_SPECULAR")), ws(vec3ws)),
					preceded(ws(tag("*MATERIAL_SHINE")), ws(float)),
					preceded(ws(tag("*MATERIAL_SHINESTRENGTH")), ws(float)),
					preceded(ws(tag("*MATERIAL_TRANSPARENCY")), ws(float)),
					preceded(ws(tag("*MATERIAL_WIRESIZE")), ws(float)),
					Shading::parse,
					preceded(ws(tag("*MATERIAL_XP_FALLOFF")), ws(float)),
					preceded(ws(tag("*MATERIAL_SELFILLUM")), ws(float)),
					Falloff::parse,
					Exponent::parse,
					permutation((
						opt(tuple((value(MapKey::Diffuse, ws(tag("*MAP_DIFFUSE"))), Map::parse)))
					)),
				)))
			)(input)?;

			let mut maps = Maps::new();
			for i in 0..(size_of::<Option<MapPair>>() / 1) {
				match data.14.i {
					Some(m) => maps.insert(m.0, m.1),
					_ => continue,
				};
			}

			Ok((input, Material {
				name: data.0.to_string(),
				class: data.1.to_string(),
				ambient: data.2,
				diffuse: data.3,
				specular: data.4,
				shine: data.5,
				shine_strength: data.6,
				transparency: data.7,
				wire_size: data.8,
				shading: data.9,
				xp_falloff: data.10,
				self_illum: data.11,
				falloff: data.12,
				xp: data.13,
				maps: maps,
			}))
		}
	}

	/// Meta info for a scene
	#[derive(Debug, PartialEq)]
	pub struct AseScene {
		pub filename: String,
		pub first_frame: i32,
		pub last_frame: i32,
		pub speed: i32,
		pub ticks: u32,
		pub bg: Vec3,
		pub ambient: Vec3,
	}

	impl Scene {
		#[cfg(feature = "import")]
		fn parse<'a>(input: &'a str) -> nom::IResult<&'a str, Scene, ASEImportError<'a>> {
			use nom::{
				branch::permutation,
				bytes::complete::tag,
				character::complete::{
					i32,
					u32
				},
				sequence::preceded
			};

			use meshio_core::nom_ext::{
				curly,
				double_quoted,
				vec3ws,
				ws
			};

			let (input, data) = preceded(ws(tag("*SCENE")),
				curly(permutation((
					preceded(ws(tag("*SCENE_FILENAME")), ws(double_quoted)),
					preceded(ws(tag("*SCENE_FIRSTFRAME")), ws(i32)),
					preceded(ws(tag("*SCENE_LASTFRAME")), ws(i32)),
					preceded(ws(tag("*SCENE_FRAMESPEED")), ws(i32)),
					preceded(ws(tag("*SCENE_TICKSPERFRAME")), ws(u32)),
					preceded(ws(tag("*SCENE_BACKGROUND_STATIC")), ws(vec3ws)),
					preceded(ws(tag("*SCENE_AMBIENT_STATIC")), ws(vec3ws))
				)))
			)(input)?;

			Ok((input, Scene {
				filename: data.0.to_string(),
				first_frame: data.1,
				last_frame: data.2,
				speed: data.3,
				ticks: data.4,
				bg: data.5,
				ambient: data.6,
			}))
		}
	}

	/// Material shading types
	#[derive(Clone, Copy, Debug, PartialEq)]
	pub enum Shading {
		Blinn,
	}

	impl Shading {
		#[cfg(feature = "import")]
		fn parse<'a>(input: &'a str) -> nom::IResult<&'a str, Shading, ImportError<'a>> {
			use nom::{
				branch::alt,
				bytes::complete::tag,
				combinator::value,
				sequence::preceded
			};

			use meshio_core::nom_ext::ws;

			preceded(ws(tag("*MATERIAL_SHADING")), alt((
				value(Shading::Blinn, ws(tag("Blinn")))
			)))(input)
		}
	}
}

#[cfg(feature = "export")]
pub mod export {
	const VERSION: u32 = 200;
	static VERSION_STR: &str = format!("*3DSMAX_ASCIIEXPORT {}\n", VERSION).to_str();
	static COMMENT: &str = "AsciiExport Version 2,00 - {}";
}

#[cfg(feature = "import")]
pub mod import {
	use nom::{
		bytes::complete::tag,
		character::complete::u32,
		error::{
			ErrorKind,
			ParseError
		},
		IResult,
		sequence::preceded
	};
	
	use thiserror::Error;
	
	use meshio_core::nom_ext::{
		curly,
		double_quoted,
		ws
	};

	use super::ast::*;

	#[derive(Error, Debug)]
	pub enum ASEImportError<'a> {
		#[error("Parser error")]
		Parse(&'a str, ErrorKind),
	}

	impl<'a> ParseError<&'a str> for ASEImportError<'a> {
		fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
			ASEImportError::Parse(input, kind)
		}

		fn append(_: &'a str, _: ErrorKind, other: Self) -> Self {
			other
		}
	}

	fn asciiexport<'a>(input: &'a str) -> IResult<&'a str, u32, ImportError<'a>> {
        preceded(ws(tag("*3DSMAX_ASCIIEXPORT")), ws(u32))(input)
	}

	fn comment<'a>(input: &'a str) -> IResult<&'a str, &'a str, ImportError<'a>> {
        preceded(ws(tag("*COMMENT")), ws(double_quoted))(input)
	}

	#[cfg(test)]
	mod tests {
		use ultraviolet::vec::Vec3;
		use super::*;
		use super::super::ast::*;

		#[test]
		fn test_asciiexport() {
			assert_eq!(asciiexport("*3DSMAX_ASCIIEXPORT 200\n"), Ok(("", 200)));
		}

		#[test]
		fn test_comment() {
			assert_eq!(comment("*COMMENT \"flumpadoo\"\n"), Ok(("", "flumpadoo")));
		}

		#[test]
		fn test_scene() {
			let input = r#"
				*SCENE {
					*SCENE_FILENAME "test.max"
					*SCENE_FIRSTFRAME 0
					*SCENE_LASTFRAME 100
					*SCENE_FRAMESPEED 30
					*SCENE_TICKSPERFRAME 160
					*SCENE_BACKGROUND_STATIC 0.4000	0.4000	0.4000
					*SCENE_AMBIENT_STATIC 0.0000	0.0000	0.0000
				}"#;
			let expected = Scene {
				filename: "test.max".to_string(),
				first_frame: 0,
				last_frame: 100,
				speed: 30,
				ticks: 160,
				bg: Vec3::new(0.4, 0.4, 0.4),
				ambient: Vec3::new(0.0, 0.0, 0.0),
			};

			assert_eq!(Scene::parse(input), Ok(("", expected)));
		}
	}
}
