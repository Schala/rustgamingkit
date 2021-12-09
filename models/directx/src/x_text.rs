pub mod ast {
	use uuid::Uuid;

	pub enum ArraySize {
		ID(String),
		Literal(usize),
	}

	pub enum VarType {
		Array(Box<VarType>, Vec<ArraySize>),
		Byte,
		Char,
		CString,
		Custom(String),
		Double,
		Dword,
		Float,
		Text,
		UChar,
		Unicode,
		Word,
	}

	pub type VarField = (VarType, String);
}

#[cfg(feature = "import")]
pub mod import {
	use nom::{
		branch::alt,
		bytes::complete::{
			tag_no_case,
			take
		},
		character::{
			complete::{
				char,
				u32
			},
			is_alphanumeric
		},
		combinator::{
			map_res,
			value
		},
		IResult,
		multi::many0,
		number::complete::float,
		sequence::{
			delimited,
			separated_pair,
			tuple
		}
	};

	use uuid::{
		self,
		Uuid
	};

	use super::ast::*;

	use rgk_core::nom_ext::{
		c_comment,
		double_quoted,
		hash_comment,
		ws
	};

	fn comment(input: &str) -> IResult<&str, ()> {
		alt((c_comment, hash_comment))(input)
	}

	fn guid(input: &str) -> IResult<&str, Result<Uuid, uuid::Error>> {
		delimited(
			ws(char('<')),
			map_res(take(36), |mut s: &str| Uuid::parse_str(s)),
			ws(char('>'))
		)(input)
	}

	fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
		take_while(|c as u8| is_alphanumeric(c) || c == b'_')(input)
	}

	fn var_type(input: &str) -> IResult<&str, XVarType> {
		alt((
			value(XVarType::Byte, tag_no_case("byte")),
			value(XVarType::Char, tag_no_case("char")),
			value(XVarType::Cstring, tag_no_case("cstring")),
			value(XVarType::Double, tag_no_case("double")),
			value(XVarType::Dword, tag_no_case("dword")),
			value(XVarType::Float, tag_no_case("float")),
			value(XVarType::Text, tag_no_case("string")),
			value(XVarType::Uchar, tag_no_case("uchar")),
			value(XVarType::Unicode, tag_no_case("unicode")),
			value(XVarType::Word, tag_no_case("word"))
		))(input)
	}
}
