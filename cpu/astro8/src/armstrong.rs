use nom::{
	branch::{
		alt,
		permutation
	},
	bytes::complete::{
		tag
	},
	character::complete::{
		char
	},
	IResult,
	multi::many1,
	sequence::{
		delimited,
		preceded
	}
};

use std::{
	collections::HashMap,
	io
};

use thiserror::Error;

use rgk_core::nom_ext::{
	double_quoted,
	ws
};

#[derive(Debug, Error)]
pub enum LexerError {
	#[error("Not a valid binary literal: {0}")]
	Bin(String),
	#[error("Not a valid hex literal: {0}")]
	Hex(String),
	#[error("I/O error")]
	IO {
		#[from]
		source: io::Error,
	},
}

/// Parses a binary literal prefixed with '0b'
fn bin(input: &str) -> IResult<&str, u16, LexerError> {
	let (input, num) = preceded(tag('%'), many1(alt((char('0', char('1'))))))(input)?;
	let Ok(digit) = u16::from_str_radix(num, 2) else {
		return LexerError::Bin(num.to_owned());
	}

	Ok((input, digit))
}

/// Parses an 'asm' block
fn lex_asm(input: &str) -> IResult<&str, String, LexerError> {
	delimited(ws(tag("asm\"")), ws(char('"')))
}

/// Parses a variable change
fn lex_change(input: &str) -> IResult<&str, (String, u16), LexerError> {
	preceded(ws(tag("change")), ws(
}

/// Parses a variable definition
fn lex_define(input: &str) -> IResult<&str, (String, u16), LexerError> {
	let (input, var) = preceded(tag("define"),
}

/// Parses a 'goto' statement
fn lex_goto(input: &str) -> IResult<&str, (String, u16), LexerError> {
	preceded(ws(tag("goto")), ws(
}

/// Parses an 'if' statement
fn lex_if(input: &str) -> IResult<&str, (String, u16), LexerError> {
	delimited(ws(tag("if")), ws(tag("endif")))(input)
}
