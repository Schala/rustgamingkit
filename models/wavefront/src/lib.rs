pub mod mtl;
pub mod obj;
pub mod refl;

use nom::{
	branch::alt,
	bytes::complete::take_while,
	character::{
		complete::multispace1,
		is_space
	},
	combinator::{
		opt,
		value
	},
	error::ParseError,
	IResult,
	multi::many0,
	number::complete::float,
	Parser,
	sequence::{
		preceded,
		tuple
	}
};

use ultraviolet::vec::Vec3;

use meshio_core::parser::hash_comment;

/// Parses discardable content (comment, whitespace, line feeds)
pub(crate) fn etc<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
	E: ParseError<&'a str>,
	F: Parser<&'a str, O, E>,
{
	preceded(
		many0(alt((
			hash_comment,
			value((), multispace1)
		))),
		inner
	)
}

/// Parses an identifier (object/group/material name, filename)
pub(crate) fn identifier<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
	E: ParseError<&'a str>
{
	take_while(move |c| !is_space(c as u8))(input)
}

/// Parses a 3D vector, with the last element optional, defaulting to `0`, used for UVWs
pub(crate) fn uvw<'a, E>(input: &'a str) -> IResult<&'a str, Vec3, E>
where
	E: ParseError<&'a str>
{
	let (input, f4vec) = tuple((etc(float), etc(float), opt(etc(float))))(input)?;
	
	Ok((input, Vec3::new(f4vec.0, f4vec.1, match f4vec.2 {
		Some(val) => val,
		_ => 0.0,
	})))
}
