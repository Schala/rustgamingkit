use std::io;
use thiserror::Error;

use nom::{
	branch::{
		alt,
		permutation
	},
	bytes::complete::{
		tag,
		tag_no_case
		take_while
	},
	character::complete::{
		char,
		not_line_ending
	},
	combinator::{
		value
	},
	IResult
};

use rgk_core::nom_ext::ws;

#[repr(u8)]
pub enum Instruction {
	NOP = 0, /// no operation
	AIN,
	BIN,
	CIN,
	LDIA,
	LDIB,
	RDEXP,
	WREXP,
	STA,
	STC,
	ADD,
	SUB,
	MULT,
	DIV,
	JMP,
	JMPZ,
	JMPC,
	JREG,
	LDAIN,
	STAOUT,
	LDLGE,
	STLGE,
	LDW,
	SWP,
	SWPC,
	PCR,
	BSL,
	BSR,
	AND,
	OR,
	NOT,
	BNK,
	BNKC,
	LDWB,
}

#[derive(Debug, Error)]
pub enum LexerError {
	#[error("I/O error")]
	IO {
		#[from]
		source: io::Error,
	},
}

/// Parses a mnemonic
fn mnemonic(input: &str) -> IResult<&str, Instruction, LexerError> {
	alt((
		value(Instruction::NOP, tag_no_case("nop")),
		value(Instruction::AIN, tag_no_case("ain")),
		value(Instruction::BIN, tag_no_case("bin")),
		value(Instruction::CIN, tag_no_case("cin")),
		value(Instruction::LDIA, tag_no_case("ldia")),
		value(Instruction::LDIB, tag_no_case("ldib")),
		value(Instruction::RDEXP, tag_no_case("rdexp")),
		value(Instruction::WREXP, tag_no_case("wrexp")),
		value(Instruction::STA, tag_no_case("sta")),
		value(Instruction::STC, tag_no_case("stc")),
		value(Instruction::ADD, tag_no_case("add")),
		value(Instruction::SUB, tag_no_case("sub")),
		value(Instruction::MULT, tag_no_case("mult")),
		value(Instruction::DIV, tag_no_case("div")),
		value(Instruction::JMP, tag_no_case("jmp")),
		value(Instruction::JMPZ, tag_no_case("jmpz")),
		value(Instruction::JMPC, tag_no_case("jmpc")),
		value(Instruction::JREG, tag_no_case("jreg")),
		value(Instruction::LDAIN, tag_no_case("ldain")),
		value(Instruction::STAOUT, tag_no_case("staout")),
		value(Instruction::LDLGE, tag_no_case("ldlge")),
		value(Instruction::STLGE, tag_no_case("stlge")),
		value(Instruction::LDW, tag_no_case("ldw")),
		value(Instruction::SWP, tag_no_case("swp")),
		value(Instruction::SWPC, tag_no_case("swpc")),
		value(Instruction::PCR, tag_no_case("pcr")),
		value(Instruction::BSL, tag_no_case("bsl")),
		value(Instruction::BSR, tag_no_case("bsr")),
		value(Instruction::AND, tag_no_case("and")),
		value(Instruction::OR, tag_no_case("or")),
		value(Instruction::NOT, tag_no_case("not")),
		value(Instruction::BNK, tag_no_case("bnk")),
		value(Instruction::BNKC, tag_no_case("bnkc")),
		value(Instruction::LDWB, tag_no_case("ldwb")),
	))(input)
}
