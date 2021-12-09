use nom::{
	branch::alt,
	bytes::complete::{
		tag,
		take_till,
		take_while
	},
	character::complete::{
		char,
		multispace0,
		not_line_ending
	},
	combinator::value,
	error::ParseError,
	IResult,
	multi::{
		count,
		length_data
	},
	number::complete::{
		float,
		le_f32,
		u8
	},
	Parser,
	sequence::{
		delimited,
		pair
	},
};

use std::str::{
	from_utf8,
	Utf8Error
};

use ultraviolet::vec::{
	Vec2,
	Vec3,
	Vec4
};

/// Parses a C-style line comment
pub fn c_comment<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
	E: ParseError<&'a str>
{
	value((), pair(tag("//"), not_line_ending))(input)
}

/// Parses the inner contents of a pair of curly braces
pub fn curly<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
	E: ParseError<&'a str>,
	F: Parser<&'a str, O, E>,
{
	delimited(ws(char('{')), inner, ws(char('}')))
}

/// Parses the contents of a double-quoted string
pub fn double_quoted<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
	E: ParseError<&'a str>
{
	delimited(
		char('"'),
		take_while(move |c| c != '"'),
		char('"')
	)(input)
}

/// Parses a hash-prefixed ('#') line comment
pub fn hash_comment<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
	E: ParseError<&'a str>
{
	value((), pair(char('#'), not_line_ending))(input)
}

/// Parses a [`Vec3`] of whitespace-delimited floats
pub fn vec3ws<'a, E>(input: &'a str) -> IResult<&'a str, Vec3, E>
where
	E: ParseError<&'a str>
{
	let (input, f3vec) = count(ws(float), 3)(input)?;

	Ok((input, Vec3::new(f3vec[0], f3vec[1], f3vec[2])))
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
///
/// From https://github.com/Geal/nom/blob/master/doc/nom_recipes.md with minor edits
pub fn ws<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
	E: ParseError<&'a str>,
	F: Parser<&'a str, O, E>,
{
	delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use ultraviolet::vec::Vec3;
    use nom::character::complete::alphanumeric1;
    use nom::error::Error;

	#[test]
	fn test_double_quoted() {
		assert_eq!(super::double_quoted::<'_, Error<&str>>("\"Hi there\""), Ok(("", "Hi there")));
		assert_ne!(super::double_quoted::<'_, Error<&str>>("Hi there"), Ok(("", "Hi there")));
	}

	#[test]
	fn test_vec3ws() {
        assert_eq!(super::vec3ws::<'_, Error<&str>>("0.1 2.3  4.5"), Ok(("", Vec3::new(0.1, 2.3, 4.5))));
	}
}
