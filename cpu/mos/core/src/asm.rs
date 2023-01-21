use bitflags::bitflags;
use std::io;
use thiserror::Error;

use nom::{
	branch::{
		alt,
		permutation
	},
	bytes::complete::{
		tag,
		tag_no_case,
		take_while
	},
	character::complete::{
		alpha1,
		alphanumeric1,
		char,
		digit1,
		hex_digit1,
		not_line_ending
	},
	combinator::{
		map,
		opt,
		value
	},
	IResult,
	multi::{
		many0_count,
		separated_list1
	},
	sequence::{
		delimited,
		pair,
		preceded,
		terminated
	}
};

use rgk_core::nom_ext::{
	double_quoted,
	ws
};

use crate::Mode;

pub enum Expression {
	Operation(Instruction, Mode, Option<u16>),
}


#[repr(u8)]
pub enum Instruction {
	ADC, /// add with carry
	AND, /// and
	ASL, /// arithmetical shift left
	BCC, /// branch on carry clear
	BCS, /// branch on carry set
	BEQ, /// branch on equal/zero set
	BIT, /// bit test
	BMI, /// branch on minus
	BNE, /// branch on not equal/zero clear
	BPL, /// branch on plus
	BRK, /// break
	BVC, /// branch on overflow clear
	BVS, /// branch on overflow set
	CLC, /// clear carry
	CLD, /// clear decimal
	CLI, /// clear interrupt disable
	CLV, /// clear overflow
	CMP, /// compare
	CPX, /// compare with X
	CPY, /// compare with Y
	DEC, /// decrement
	DEX, /// decrement X
	DEY, /// decrement Y
	EOR, /// exclusive or
	INC, /// increment
	INX, /// increment X
	INY, /// increment Y
	JMP, /// jump
	JSR, /// jump to subroutine
	LDA, /// load accumulator
	LDX, /// load X
	LDY, /// load Y
	LSR, /// logical shift right
	NOP, /// no operation
	ORA, /// or
	PHA, /// push accumulator to stack
	PHP, /// push processor status to stack
	PLA, /// pull accumulator from stack
	PLP, /// pull processor status from stack
	ROL, /// rotate left
	ROR, /// rotate right
	RTI, /// return from interrupt
	RTS, /// return from subroutine
	SBC, /// subtract with carry
	SEC, /// set carry
	SED, /// set decimal
	SEI, /// set interrupt disable
	STA, /// store accumulator
	STX, /// store X
	STY, /// store Y
	TAX, /// transfer accumulator to X
	TAY, /// transfer accumulator to Y
	TSX, /// transfer stack pointer to X
	TXA, /// transfer X to accumulator
	TXS, /// transfer X to stack pointer
	TYA, /// transfer Y to accumulator
}

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

/*
#[repr(u8)]
pub enum Arithmetic {
	Add,
	BitAnd,
	BitOr,
	BitXor,
	Div,
	Mul,
	ShiftLeft,
	ShiftRight,
	Sub
}

#[repr(u8)]
pub enum Prefix {
	Hi,
	Lo
}*/

/// Parses an absolute operation with X offset
fn abx(input: &str) -> IResult<&str, u16, LexerError> {
	terminated(int, ws(preceded(ws(char(',')), ws(tag_no_case("x")))))(input)
}

/// Parses an absolute operation with Y offset
fn aby(input: &str) -> IResult<&str, u16, LexerError> {
	terminated(int, ws(preceded(ws(char(',')), ws(tag_no_case("y")))))(input)
}

/*
/// Parses an arithmetic symbol
fn arith(input: &str) -> IResult<&str, Arithmetic, LexerError> {
	alt((
		value(Arithmetic::Add, char('+')),
		value(Arithmetic::BitAnd, char('&')),
		value(Arithmetic::BitOr, char('|')),
		value(Arithmetic::BitXor, char('^')),
		value(Arithmetic::Div, char('/')),
		value(Arithmetic::Mul, char('*')),
		value(Arithmetic::ShiftLeft, tag("<<")),
		value(Arithmetic::ShiftRight, tag(">>")),
		value(Arithmetic::Sub, char('-'))
	))(input)
}

/// Parses an assignment
fn assign(input: &str) -> IResult<&str, (&str, u16), LexerError> {
	pair(identifier, preceded(ws(alt((tag_no_case("equ"), char('=')))), int)(input)
}*/

/// Parses a binary literal prefixed with '%'
fn bin(input: &str) -> IResult<&str, u16, LexerError> {
	let (input, num) = preceded(char('%'), many1(alt((char('0', char('1'))))))(input)?;
	let Ok(digit) = u16::from_str_radix(num, 2) else {
		return LexerError::Bin(num.to_owned());
	}

	Ok((input, digit))
}

/*/// Parses and discards a line comment denoted by ';'
fn comment(input: &str) -> IResult<&str, (), LexerError> {
	value((), pair(char(';'), not_line_ending))(input)
}*/

/*/// Parses a byte array
fn db(input: &str) -> IResult<&str, Vec<u8>, LexerError> {
	preceded(ws(preceded(opt(char('.')), alt((tag_no_case("db"), tag_no_case("byte")))),
		separated_list1(map(int, |i| i as u8), ws(char(','))))(input)
}

/// Parses a word array
fn dw(input: &str) -> IResult<&str, Vec<u16>, LexerError> {
	preceded(ws(preceded(opt(char('.')), alt((tag_no_case("dw"), tag_no_case("word")))),
		separated_list1(map(int, |i| i as u16), ws(char(','))))(input)
}*/

/// Parses a hex literal
fn hex(input: &str) -> IResult<&str, u16, LexerError> {
	let (input, num) = hex_digit1(input)?;
	let Ok(digit) = u16::from_str_radix(num, 16) else {
		return LexerError::Hex(num.to_owned());
	}

	Ok((input, digit))
}

/*
/// Parses an identifier
fn identifier(input: &str) -> IResult<&str, &str, LexerError> {
	recognize(pair(alt((alpha1, char('_'))),
		many0_count(alt((alphanumeric1, char('_'))))))(input)
}*/

/// Parses an immediate operation
fn imm(input: &str) -> IResult<&str, u8, LexerError> {
	preceded(char('#'), map(int, |i| i as u8))(input)
}

/// Parses an indirect operation
fn ind(input: &str) -> IResult<&str, u16, LexerError> {
	delimited(char('('), int, char(')'))(input)
}

/// Parses an indirect operation with X offset
fn izx(input: &str) -> IResult<&str, u16, LexerError> {
	delimited(char('('), abx, char(')'))(input)
}

/// Parses an indirect operation with Y offset
fn izy(input: &str) -> IResult<&str, u16, LexerError> {
	delimited(char('('), aby, char(')'))(input)
}

/// Parses an integer
fn int(input: &str) -> IResult<&str, u16, LexerError> {
	alt((bin, digit, hex))(input)
}

/*/// Parses a label
fn label(input: &str) -> IResult<&str, &str, LexerError> {
	terminated(identifier, char(':'))(input)
}

/// Parses a local label
fn local(input: &str) -> IResult<&str, &str, LexerError> {
	preceded(alt((char('.'), char('@')))), label)(input)
}*/

/// Parses a mnemonic
fn mnemonic(input: &str) -> IResult<&str, Instruction, LexerError> {
	alt((
		value(Instruction::ADC, tag_no_case("adc")),
		value(Instruction::AND, tag_no_case("and")),
		value(Instruction::ASL, tag_no_case("asl")),
		value(Instruction::BCC, tag_no_case("bcc")),
		value(Instruction::BCS, tag_no_case("bcs")),
		value(Instruction::BEQ, tag_no_case("beq")),
		value(Instruction::BIT, tag_no_case("bit")),
		value(Instruction::BMI, tag_no_case("bmi")),
		value(Instruction::BNE, tag_no_case("bne")),
		value(Instruction::BPL, tag_no_case("bpl")),
		value(Instruction::BRK, tag_no_case("brk")),
		value(Instruction::BVC, tag_no_case("bvc")),
		value(Instruction::BVS, tag_no_case("bvs")),
		value(Instruction::CLC, tag_no_case("clc")),
		value(Instruction::CLD, tag_no_case("cld")),
		value(Instruction::CLI, tag_no_case("cli")),
		value(Instruction::CLV, tag_no_case("clv")),
		value(Instruction::CMP, tag_no_case("cmp")),
		value(Instruction::CPX, tag_no_case("cpx")),
		value(Instruction::CPY, tag_no_case("cpy")),
		value(Instruction::DEC, tag_no_case("dec")),
		value(Instruction::DEX, tag_no_case("dex")),
		value(Instruction::DEY, tag_no_case("dey")),
		value(Instruction::EOR, tag_no_case("eor")),
		value(Instruction::INC, tag_no_case("inc")),
		value(Instruction::INX, tag_no_case("inx")),
		value(Instruction::INY, tag_no_case("iny")),
		value(Instruction::JMP, tag_no_case("jmp")),
		value(Instruction::JSR, tag_no_case("jsr")),
		value(Instruction::LDA, tag_no_case("lda")),
		value(Instruction::LDX, tag_no_case("ldx")),
		value(Instruction::LDY, tag_no_case("ldy")),
		value(Instruction::LSR, tag_no_case("lsr")),
		value(Instruction::NOP, tag_no_case("nop")),
		value(Instruction::ORA, tag_no_case("ora")),
		value(Instruction::PHA, tag_no_case("pha")),
		value(Instruction::PHP, tag_no_case("php")),
		value(Instruction::PLA, tag_no_case("pla")),
		value(Instruction::PLP, tag_no_case("plp")),
		value(Instruction::ROL, tag_no_case("rol")),
		value(Instruction::ROR, tag_no_case("ror")),
		value(Instruction::RTI, tag_no_case("rti")),
		value(Instruction::RTS, tag_no_case("rts")),
		value(Instruction::SBC, tag_no_case("sbc")),
		value(Instruction::SEC, tag_no_case("sec")),
		value(Instruction::SED, tag_no_case("sed")),
		value(Instruction::SEI, tag_no_case("sei")),
		value(Instruction::STA, tag_no_case("sta")),
		value(Instruction::STX, tag_no_case("stx")),
		value(Instruction::STY, tag_no_case("sty")),
		value(Instruction::TAX, tag_no_case("tax")),
		value(Instruction::TAY, tag_no_case("tay")),
		value(Instruction::TSX, tag_no_case("tsx")),
		value(Instruction::TXA, tag_no_case("txa")),
		value(Instruction::TXS, tag_no_case("txs")),
		value(Instruction::TYA, tag_no_case("tya"))
	))(input)
}

/*/// Parses an ORG directive
fn org(input: &str) -> IResult<&str, Register, LexerError> {
	preceded(ws(alt((tag("*="), preceded(opt(char('.')), tag_no_case("org"))))), int)(input)
}*/

/*
/// Parses a prefix symbol
fn prefix(input: &str) -> IResult<&str, Prefix, LexerError> {
	alt((
		value(Prefix::Hi, char('>')),
		value(Prefix::Lo, char('<'))
	))(input)
}*/

