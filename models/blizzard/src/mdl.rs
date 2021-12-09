#[cfg(feature = "import")]
pub mod import {
	use nom::{
		branch::{
			alt,
			permutation
		},
		bytes::complete::{
			tag,
			tag_no_case
		},
		character::complete::{
			char,
			multispace1,
			u32
		},
		combinator::{
			opt,
			value
		},
		IResult,
		multi::{
			count,
			many0,
			separated_list1
		},
		number::complete::float,
		sequence::{
			delimited,
			preceded,
			tuple
		}
	};

	use ultraviolet::vec::Vec3;

	use meshio_core::nom_ext::{
		c_comment,
		curly,
		double_quoted
	};

	use crate::mdlx::{
		ChunkData,
		Extent,
		Filter,
		import::WC3ImportError,
		Layer,
		Seq,
		SeqFlags,
		Texture
	};

	/// Parses discardable content (comment, whitespace, line feeds)
	fn etc<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, WC3ImportError<'a>>
	where
		F: Fn(&'a str) -> IResult<&'a str, O>,
	{
		let discard = |i| -> IResult<&'a str, ()> {
			value((),
				many0(alt((
					c_comment,
					value((), multispace1)
				)))
			)(i)
		};

		move |input: &'a str| {
			let (input, _) = discard(input)?;
			let (input, output) = inner(input)?;
			let (input, _) = discard(input)?;
			let (input, _) = opt(char(','))(input)?;
			let (input, _) = discard(input)?;

			Ok((input, output))
		}
	}

	/// Parses a static qualified field
	fn static_field<'a>(input: &'a str) -> IResult<&'a str, bool, WC3ImportError<'a>> {
		let (input, st) = opt(etc(tag_no_case("static")))(input)?;
		Ok((input, st.is_some()))
	}

	/// Parses a [`Vec3`] of whitespace-delimited, comma-separated floats delimited by curly braces
	fn vec3<'a>(input: &'a str) -> IResult<&'a str, Vec3, MdlImportError<'a>> {
		let (input, f3vec) = curly(separated_list1(etc(char(',')), float))(input)?;
		Ok((input, Vec3::new(f3vec[0], f3vec[1], f3vec[2])))
	}

	fn anim<'a>(input: &'a str) -> IResult<&'a str, Seq, MdlImportError<'a>> {
		let (input, data) = preceded(etc(tag_no_case("Anim")),
			tuple((double_quoted, curly(permutation((
					interval,
					opt(rarity),
					minimum_extent,
					maximum_extent,
					bounds_radius,
					opt(move_speed),
					opt(non_looping)
				)))),
		)(input)?;

		Ok((input, Seq {
			name: data.0.to_string(),
			interval: [data.1.0.0, data.1.0.1],
			speed: match data.1.5 {
				Some(s) => s,
				_ => 0.0,
			},
			flags: match data.1.6 {
				Some(f) => f,
				_ => SeqFlags::Looping,
			},
			rarity: match data.1.1 {
				Some(r) => r,
				_ => 0.0,
			},
			sync_point: None,
			extent: Extent {
				bounds_radius: data.1.4,
				min: data.1.2,
				max: data.1.3,
			},
		}))
	}

	fn bitmap<'a>(input: &'a str) -> IResult<&'a str, Texture, MdlImportError<'a>> {
		let (input, data) = curly(permutation((
				image,
				opt(replaceable_id)
			))
		)(input)?;

		Ok((input, Texture {
			replaceable_id: match data.1 {
				Some(id) => id,
				_ => 0,
			},
			file_name: data.0.to_string(),
			flags: 0,
		}))
	}

	fn blend_time<'a>(input: &'a str) -> IResult<&'a str, u32, MdlImportError<'a>> {
		preceded(etc(tag_no_case("BlendTime")), etc(u32))(input)
	}

	fn bounds_radius<'a>(input: &'a str) -> IResult<&'a str, f32, MdlImportError<'a>> {
		preceded(etc(tag_no_case("BoundsRadius")), etc(float))(input)
	}

	fn filter_mode<'a>(input: &'a str) -> IResult<&'a str, Filter, MdlImportError<'a>> {
		preceded(etc(tag_no_case("FilterMode")), alt((
			value(Filter::None, etc(tag_no_case("None"))),
			value(Filter::Transparent, etc(tag_no_case("Transparent"))),
		)))(input)
	}

	fn format_version<'a>(input: &'a str) -> IResult<&'a str, u32, MdlImportError<'a>> {
		preceded(etc(tag_no_case("FormatVersion")), etc(u32))(input)
	}

	fn image<'a>(input: &'a str) -> IResult<&'a str, &'a str, MdlImportError<'a>> {
		preceded(etc(tag_no_case("Image")), etc(double_quoted))(input)
	}

	fn interval<'a>(input: &'a str) -> IResult<&'a str, (u32, u32), MdlImportError<'a>> {
		preceded(etc(tag_no_case("Interval")), curly(tuple((etc(u32), etc(u32)))))(input)
	}

	fn layer<'a>(input: &'a str) -> IResult<&'a str, Layer, MdlImportError<'a>> {
		let (input, data) = preceded(etc(tag_no_case("Layer")), curly(permutation((
				filter_mode,
				texture_id
			)),
		))(input)?;

		Ok((input, Layer {
			size: None,
			filter: data.0,
			shading: None,
			tex: data.1,
			tex_anim: None,
			coord: None,
			alpha: None,
			v800: None,

		}))
	}

	fn maximum_extent<'a>(input: &'a str) -> IResult<&'a str, Vec3, MdlImportError<'a>> {
		preceded(etc(tag_no_case("MaximumExtent")), etc(vec3))(input)
	}

	fn minimum_extent<'a>(input: &'a str) -> IResult<&'a str, Vec3, MdlImportError<'a>> {
		preceded(etc(tag_no_case("MinimumExtent")), etc(vec3))(input)
	}

	fn model<'a>(input: &'a str) -> IResult<&'a str, ChunkData, MdlImportError<'a>> {
		let (input, data) = preceded(etc(tag_no_case("Model")),
			tuple((double_quoted, curly(permutation((
					blend_time,
					minimum_extent,
					maximum_extent
				)),
			)))
		)(input)?;

		Ok((input, ChunkData::ModelInfo {
			name: data.0.to_string(),
			extent: Extent {
				bounds_radius: 0.0,
				min: data.1.1,
				max: data.1.2,
			},
			blend_time: data.1.0,
		}))
	}

	fn move_speed<'a>(input: &'a str) -> IResult<&'a str, f32, MdlImportError<'a>> {
		preceded(etc(tag_no_case("MoveSpeed")), etc(float))(input)
	}

	fn non_looping<'a>(input: &'a str) -> IResult<&'a str, SeqFlags, MdlImportError<'a>> {
		value(SeqFlags::NonLooping, etc(tag_no_case("NonLooping")))(input)
	}

	fn rarity<'a>(input: &'a str) -> IResult<&'a str, f32, MdlImportError<'a>> {
		preceded(etc(tag_no_case("Rarity")), etc(float))(input)
	}

	fn replaceable_id<'a>(input: &'a str) -> IResult<&'a str, u32, MdlImportError<'a>> {
		preceded(etc(tag_no_case("ReplaceableId")), etc(u32))(input)
	}

	fn sequences<'a>(input: &'a str) -> IResult<&'a str, Vec<Seq>, MdlImportError<'a>> {
		let (input, num_seqs) = preceded(etc(tag_no_case("Sequences")), u32)(input)?;
		delimited(etc(char('{')), count(anim, num_seqs), etc(char('}')))(input)
	}

	fn texture_id<'a>(input: &'a str) -> IResult<&'a str, u32, MdlImportError<'a>> {
		preceded(tuple((static_field, etc(tag_no_case("TextureID")))), etc(u32))(input)
	}

	fn textures<'a>(input: &'a str) -> IResult<&'a str, Vec<Texture>, MdlImportError<'a>> {
		let (input, num_texs) = preceded(etc(tag_no_case("Textures")), etc(u32))(input)?;
		delimited(etc(char('{')), count(bitmap, num_texs), etc(char('}')))(input)
	}

	fn version<'a>(input: &'a str) -> IResult<&'a str, u32, MdlImportError<'a>> {
		preceded(etc(tag_no_case("Version")), delimited(etc(char('{')), format_version, etc(char('}'))))(input)
	}

	#[cfg(test)]
	mod tests {
		#[test]
		fn test_version() {
			let input = r#"
			// Converted Mon Oct 18 11:47:14 2021
			// Conv Version Mar  3 2007.
			Version {
				FormatVersion 800,
			}
			"#;

			assert_eq!(super::version(input), Ok(("", 800)));
		}
	}
}
