use cfg_if::cfg_if;
use indexmap::IndexMap;
use thiserror::Error

use std::{
	collections::HashSet,
	cmp::{
		max,
		min
	},
	io
};;

use crate::Mode;

#[cfg(feature = "csg65ce02")]
use crate::csg65ce02::ExMode;

#[derive(Debug, Error)]
pub enum MOS6500AsmError {
	#[error("I/O error")]
	IO {
		#[from]
		source: io::Error,
	},
	#[error("Cannot resolve jump: {0}")]
	Jump(usize),

	#[error("Relative jump distance too high: {0}")]
	Relative(usize),

	#[error("Unsupported operation in the current mode: {0}")]
}

/// Code generation
fn codegen(&self, tree: IndexMap<usize, ast::Expression>) -> Result<Vec<u8>, MOS6500AsmError> {
	let mut code = vec![];

	for (offs, node) tree.iter() {
		match node {
			ast::Expression::Operation(op, mode, val) => {
				// resolve the jump
				if op == Instruction::JMP || Instruction::JSR {
					let raw_val = val.unwrap();

					if !self.ops.contains(raw_val) {
						return MOS6500AsmError::Jump(raw_val)
					}

					code.push(node.get_opcode());

					let max = max(offs as u16, raw_val) as i16;
					let min = min(offs as u16, raw_val) as i16;
					let diff = max - min;

					if diff < -128 || diff > 127 {
						return MOS6500AsmError::Relative(diff as usize)
					}

					let rel_val = ((diff + 2) as i8) as u8;
					code.push(rel_val);
				} else {
					code.push(node.get_opcode());

					match node.get_value_size() {
						1 => code.push(val.unwrap() as u8),
						2 => code.push(val.unwrap()),
						_ => (),
					}
				}
			},
		}
	}

	Ok(code)
}

/// Parses and generates the machine code from the input assembly
pub fn compile(&self) -> Vec<u8> {
	let mut input = self.input.as_str();
	let mut offs = HashSet::new();
	let mut ops = IndexMap::new();

	while let Ok((remaining, expr)) = lex::operation(input) {
		input = remaining;

		match expr {
			ast::Expression::Operation(_, mode, val) => {
				ops.insert(offs, expr);

				if mode == Mode::REL {
					ops.insert(
				}
			},
			_ => offs += expr.get_value_size() + 1;
		}
	}
}

/// Abstract syntax tree
mod ast {
	use crate::Mode;

	pub(crate) enum Expression {
		Operation(Instruction, Option<u8>, Mode, Option<u16>),
	}

	impl Expression {
		/// Gets the value size in bytes
		pub(crate) const fn get_value_size(&self) -> u8 {
			if let Expression::Operation(_, _, mode, _) = self {
				match mode {
					Mode::ABS | Mode::ABX | Mode::ABY => 2,
					Mode::IMP => 0,
					_ => 1,
				}
			} else {
				0 // placeholder
			}
		}

		/// Gets the byte opcode if the expression is an operation
		pub(crate) const fn get_opcode(&self) -> u8 {
			if let Expression::Operation(op, bit, mode, _) = self {
				match op {
					Instruction::BRK => 0,
					Instruction::PHP => 8,
					Instruction::BPL => 16,
					Instruction::CLC => 24,
					Instruction::JSR => 32,
					Instruction::PLP => 40,
					Instruction::BMI => 48,
					Instruction::SEC => 56,
					Instruction::RTI => 64,
					Instruction::PHA => 72,
					Instruction::BVC => 80,
					Instruction::CLI => 88,
					Instruction::RTS => 96,
					Instruction::PLA => 104,
					Instruction::BVS => 112,
					Instruction::SEI => 120,
					Instruction::DEY => 136,
					Instruction::TXA => 138,
					Instruction::BCC => 144,
					Instruction::TYA => 152,
					Instruction::TXS => 154,
					Instruction::TAY => 168,
					Instruction::TAX => 170,
					Instruction::BCS => 176,
					Instruction::CLV => 184,
					Instruction::TSX => 186,
					Instruction::INY => 200,
					Instruction::DEX => 202,
					Instruction::BNE => 208,
					Instruction::CLD => 216,
					Instruction::INX => 232,
					Instruction::BEQ => 240,
					Instruction::SED => 248,


					Instruction::ORA => {
						match mode {
							Mode::IZX => 1,
							Mode::ZP0 => 5,
							Mode::IMM => 9,
							Mode::ABS => 13,
							Mode::IZY => 17,
							Mode::ZPX => 21,
							Mode::ABY => 25,
							Mode::ABX => 29,
							_ => unreachable!(),
						}
					},

					Instruction::ASL => {
						match mode {
							Mode::ZP0 => 6,
							Mode::IMP => 10,
							Mode::ABS => 14,
							Mode::ZPX => 22,
							Mode::ABX => 30,
							_ => unreachable!(),
						}
					},

					Instruction::AND => {
						match mode {
							Mode::IZX => 33,
							Mode::ZP0 => 37,
							Mode::IMM => 41,
							Mode::ABS => 45,
							Mode::IZY => 49,
							Mode::ZPX => 53,
							Mode::ABY => 57,
							Mode::ABX => 61,
							_ => unreachable!(),
						}
					},

					Instruction::BIT => {
						match mode {
							Mode::ZP0 => 36,
							Mode::ABS => 44,
							_ => unreachable!(),
						}
					},

					Instruction::ROL => {
						match mode {
							Mode::ZP0 => 38,
							Mode::IMP => 42,
							Mode::ZPX => 54,
							Mode::ABX => 62,
							_ => unreachable!(),
						}
					},

					Instruction::EOR => {
						match mode {
							Mode::IZX => 65,
							Mode::ZP0 => 69,
							Mode::IMM => 73,
							Mode::ABS => 77,
							Mode::IZY => 81,
							Mode::ZPX => 85,
							Mode::ABY => 89,
							Mode::ABX => 93,
							_ => unreachable!(),
						}
					},

					Instruction::LSR => {
						match mode {
							Mode::ZP0 => 70,
							Mode::IMP => 74,
							Mode::ABS => 78,
							Mode::ZPX => 86,
							Mode::ABX => 94,
							_ => unreachable!(),
						}
					},

					Instruction::JMP => {
						match mode {
							Mode::ABS => 76,
							Mode::IND => 108,
							_ => unreachable!(),
						}
					},

					Instruction::ADC => {
						match mode {
							Mode::IZX => 97,
							Mode::ZP0 => 101,
							Mode::IMM => 105,
							Mode::ABS => 109,
							Mode::IZY => 113,
							Mode::ZPX => 117,
							Mode::ABY => 121,
							Mode::ABX => 125,
							_ => unreachable!(),
						}
					},

					Instruction::ROR => {
						match mode {
							Mode::ZP0 => 102,
							Mode::IMP => 106,
							Mode::ABS => 110,
							Mode::ZPX => 118,
							Mode::ZBX => 126,
							_ => unreachable!(),
						}
					},

					Instruction::STA => {
						match mode {
							Mode::IZX => 129,
							Mode::ZP0 => 133,
							Mode::ABS => 141,
							Mode::IZY => 145,
							Mode::ZPX => 149,
							Mode::ABY => 153,
							Mode::ABX => 157,
							_ => unreachable!(),
						}
					},

					Instruction::STY => {
						match mode {
							Mode::ZP0 => 132,
							Mode::ABS => 140,
							Mode::ZPX => 148,
							_ => unreachable!(),
						}
					},

					Instruction::STX => {
						match mode {
							Mode::ZP0 => 134,
							Mode::ABS => 142,
							Mode::ZPY => 150,
							_ => unreachable!(),
						}
					},

					Instruction::LDY => {
						match mode {
							Mode::IMM => 160,
							Mode::ZP0 => 164,
							Mode::ABS => 172,
							Mode::ZPX => 180,
							Mode::ABX => 188,
							_ => unreachable!(),
						}
					},

					Instruction::LDA => {
						match mode {
							Mode::IZX => 161,
							Mode::ZP0 => 165,
							Mode::IMM => 169,
							Mode::ABS => 173,
							Mode::IZY => 177,
							Mode::ZPX => 181,
							Mode::ABY => 185,
							Mode::ABX => 189,
							_ => unreachable!(),
						}
					},

					Instruction::LDX => {
						match mode {
							Mode::IMM => 162,
							Mode::ZP0 => 166,
							Mode::ABS => 174,
							Mode::ZPY => 182,
							Mode::ABY => 190,
							_ => unreachable!(),
						}
					},

					Instruction::CPY => {
						match mode {
							Mode::IMM => 192,
							Mode::ZP0 => 196,
							Mode::ABS => 204,
							_ => unreachable!(),
						}
					},

					Instruction::CMP => {
						match mode {
							Mode::IZX => 193,
							Mode::ZP0 => 197,
							Mode::IMM => 201,
							Mode::ABS => 205,
							Mode::IZY => 209,
							Mode::ZPX => 213,
							Mode::ABY => 217,
							Mode::ABX => 221,
							_ => unreachable!(),
						}
					},

					Instruction::DEC => {
						match mode {
							Mode::ZP0 => 198,
							Mode::ABS => 206,
							Mode::ZPX => 214,
							Mode::ABX => 222,
							_ => unreachable!(),
						}
					},

					Instruction::CPX => {
						match mode {
							Mode::IMM => 224,
							Mode::ZP0 => 228,
							Mode::ABS => 236,
							_ => unreachable!(),
						}
					},

					Instruction::SBC => {
						match mode {
							Mode::IZX => 225,
							Mode::ZP0 => 229,
							Mode::IMM => 233,
							Mode::ABS => 237,
							Mode::IZY => 241,
							Mode::ZPX => 245,
							Mode::ABY => 249,
							Mode::ABX => 253,
							_ => unreachable!(),
						}
					},

					Instruction::INC => {
						match mode {
							Mode::ZP0 => 230,
							Mode::ABS => 238,
							Mode::ZPX => 246,
							Mode::ABX => 254,
							_ => unreachable!(),
						}
					},

					_ => unreachable!(),
				}
			} else {
				panic!("Tried to get the opcode of a non-operation");
			}
		}
	}

	#[repr(u8)]
	pub(crate) enum Instruction {
		ADC, /// add with carry
		AND, /// and
		ASL, /// arithmetical shift left
		ASR, /// arithmetical shift right (signed), 65CE02 extension
		ASW, /// arithmetical shift left (word), 65CE02 extension
		AUG, /// 65CE02 extension
		BBR, /// branch on bit reset, 65CE02 extension
		BBS, /// branch on bit set, 65CE02 extension
		BCC, /// branch on carry clear
		BCS, /// branch on carry set
		BEQ, /// branch on equal/zero set
		BIT, /// bit test
		BMI, /// branch on minus
		BNE, /// branch on not equal/zero clear
		BPL, /// branch on plus
		BRA, /// branch always, 65CE02 extension
		BRK, /// break
		BSR, /// branch to subroutine, 65CE02 extension
		BVC, /// branch on overflow clear
		BVS, /// branch on overflow set
		CLC, /// clear carry
		CLD, /// clear decimal
		CLE, /// clear stack extend, 65CE02 extension
		CLI, /// clear interrupt disable
		CLV, /// clear overflow
		CMP, /// compare
		CPX, /// compare with X
		CPY, /// compare with Y
		CPZ, /// compare with Z, 65CE02 extension
		DEA, /// decrement accumulator, 65CE02 extension
		DEC, /// decrement
		DEW, /// decrement word, 65CE02 extension
		DEX, /// decrement X
		DEY, /// decrement Y
		DEZ, /// decrement Z, 65CE02 extension
		EOR, /// exclusive or
		INA, /// increment accumulator, 65CE02 extension
		INC, /// increment
		INW, /// increment word, 65CE02 extension
		INX, /// increment X
		INY, /// increment Y
		INZ, /// increment Z, 65CE02 extension
		JMP, /// jump
		JSR, /// jump to subroutine
		LDA, /// load accumulator
		LDX, /// load X
		LDY, /// load Y
		LDZ, /// load Z, 65CE02 extension
		LSR, /// logical shift right
		NEG, /// two's compliment negation, 65CE02 extension
		NOP, /// no operation
		ORA, /// or
		PHA, /// push accumulator to stack
		PHP, /// push processor status to stack
		PHW, /// push word to stack, 65CE02 extension
		PHX, /// push X to stack, 65CE02 extension
		PHY, /// push Y to stack, 65CE02 extension
		PHZ, /// push Z to stack, 65CE02 extension
		PLA, /// pull accumulator from stack
		PLP, /// pull processor status from stack
		PLX, /// pull X from stack, 65CE02 extension
		PLY, /// pull Y from stack, 65CE02 extension
		PLZ, /// pull Z from stack, 65CE02 extension
		RMB, /// reset memory bit, 65CE02 extension
		ROL, /// rotate left
		ROR, /// rotate right
		ROW, /// rotate left (word), 65CE02 extension
		RTI, /// return from interrupt
		RTN, /// return to an address offset in stack, 65CE02 extension
		RTS, /// return from subroutine
		SBC, /// subtract with carry
		SEC, /// set carry
		SED, /// set decimal
		SEE, /// set stack extend, 65CE02 extension
		SEI, /// set interrupt disable
		SMB, /// set memory bit, 65CE02 extension
		STA, /// store accumulator
		STX, /// store X
		STY, /// store Y
		STZ, /// extension: 65CE02: store Z, 65C02: store 0
		TAB, /// transfer accumulator to base page, 65CE02 extension
		TAX, /// transfer accumulator to X
		TAY, /// transfer accumulator to Y
		TAZ, /// transfer accumulator to Z, 65CE02 extension
		TBA, /// fransfer base page to accumulator, 65CE02 extension
		TRB, /// test and reset memory bits against accumulator, 65CE02 extension
		TSB, /// test and set memory bits against accumulator, 65CE02 extension
		TSX, /// transfer stack pointer to X
		TSY, /// transfer stack pointer to Y, 65CE02 extension
		TXA, /// transfer X to accumulator
		TXS, /// transfer X to stack pointer
		TYA, /// transfer Y to accumulator,
		TYS, /// transfer Y to stack pointer, 65CE02 extension
		TZA, /// transfer Z to accumulator, 65CE02 extension
	}

	/*
	#[repr(u8)]
	pub(crate) enum Arithmetic {
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
	pub(crate) enum Prefix {
		Hi,
		Lo
	}*/
}

/// Lexer
mod lex {
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
			terminated,
			tuple
		}
	};

	use std::io;
	use thiserror::Error;

	use super::ast::*;

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

	/// Parses an absolute operation
	fn abs(input: &str) -> IResult<&str, (Mode, u16), LexerError> {
		let (input, v) = int(input)?;
		Ok((input, (Mode::ABS, v)))
	}

	/// Parses an absolute operation with X offset
	fn abx(input: &str) -> IResult<&str, u16, LexerError> {
		let (input, v) = terminated(int, ws(preceded(ws(char(',')), ws(tag_no_case("x")))))(input)?;
		Ok((input, (Mode::ABX, v)))
	}

	/// Parses an absolute operation with Y offset
	fn aby(input: &str) -> IResult<&str, u16, LexerError> {
		let (input, v) = terminated(int, ws(preceded(ws(char(',')), ws(tag_no_case("y")))))(input)?;
		Ok((input, (Mode::ABY, v)))
	}

	/// Analyses code flow, identifying possible operations and labels
	pub(crate) analyze_flow(input: &str) -> IndexMap<Expression> {
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

	/// Parses and discards a line comment denoted by ';'
	fn comment(input: &str) -> IResult<&str, (), LexerError> {
		value((), pair(char(';'), not_line_ending))(input)
	}

	/*/// Parses a byte array
	fn db(input: &str) -> IResult<&str, Vec<u8>, LexerError> {
		preceded(ws(preceded(opt(char('.')), alt((tag_no_case("db"), tag_no_case("byt"), tag_no_case("byte")))),
			separated_list1(map(int, |i| i as u8), ws(char(','))))(input)
	}

	/// Parses a word array
	fn dw(input: &str) -> IResult<&str, Vec<u16>, LexerError> {
		preceded(ws(preceded(opt(char('.')), alt((tag_no_case("dw"), tag_no_case("word")))),
			separated_list1(map(int, |i| i as u16), ws(char(','))))(input)
	}*/

	/// Parses a hex literal prefixed  with '$'
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
	fn imm(input: &str) -> IResult<&str, (Mode, u16), LexerError> {
		let (input, v) = preceded(char('#'), map(int, |i| i as u16))(input)?;
		Ok((input, (Mode::IMM, v)))
	}

	/// Parses an indirect operation
	fn ind(input: &str) -> IResult<&str, (Mode, u16), LexerError> {
		let (input, v) = delimited(char('('), int, char(')'))(input)?;
		Ok((input, (Mode::IND, v)))
	}

	/// Parses an integer
	fn int(input: &str) -> IResult<&str, u16, LexerError> {
		alt((bin, digit, hex))(input)
	}

	/// Parses an indirect operation with X offset
	fn izx(input: &str) -> IResult<&str, (Mode, u16), LexerError> {
		let (input, v) = delimited(char('('),
			terminated(int, ws(preceded(ws(char(',')), ws(tag_no_case("x"))))
		), char(')'))(input)?;
		Ok((input, (Mode::IZX, v)))
	}

	/// Parses an indirect operation with Y offset
	fn izy(input: &str) -> IResult<&str, (Mode, u16), LexerError> {
		let (input, v) = delimited(char('('),
			terminated(int, ws(preceded(ws(char(',')), ws(tag_no_case("y"))))
		), char(')'))(input)?;
		Ok((input, (Mode::IZY, v)))
	}

	/*/// Parses a label
	fn label(input: &str) -> IResult<&str, &str, LexerError> {
		terminated(identifier, char(':'))(input)
	}*/

	/*
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

	/// Parses an assembly operation pending on address mode
	fn mode(input: &str) -> IResult<&str, (Mode, Option<u16>), LexerError> {
		let (input, (m, v)) = opt(alt((abs, abx, aby, imm, ind, izx, izy, rel, zp, zpx, zpy)))(input)

		if let Some(not_imp) = m {
			Ok((input, (m, v)))
		} else {
			Ok((input, (Mode::IMP, None)))
		}
	}

	/// Parses an assembly operation
	fn operation(input: &str) -> IResult<&str, Expression, LexerError> {
		let (input, t) = tuple((ws(mnemonic), opt(mode)))(input)?;

		Ok((input, Expression::Operation(
			t.0,
			t.1.0,
			t.1.1
		))
	}

	/*/// Parses an ORG directive
	fn org(input: &str) -> IResult<&str, u16, LexerError> {
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

	/// Parses a relative operation with Y offset
	fn rel(input: &str) -> IResult<&str, (Mode, u16), LexerError> {
		let (input, v) = int(input)?;
		Ok((input, (Mode::REL, v)))
	}

	/*
	/// Parses a string
	fn tx(input: &str) -> IResult<&str, Vec<u8>, LexerError> {
		preceded(ws(preceded(opt(char('.')), alt((tag_no_case("tx"), tag_no_case("text")))),
			double_quoted(map(int, |i| i as u8))))(input)
	}*/

	/// Parses a zero page operation
	fn zp(input: &str) -> IResult<&str, (Mode, u16), LexerError> {
		let (input, v) = int(input)?;
		Ok((input, (Mode::ZP0, v)))
	}

	/// Parses a zero page operation with X offset
	fn zpx(input: &str) -> IResult<&str, u16, LexerError> {
		let (input, v) = terminated(int, ws(preceded(ws(char(',')), ws(tag_no_case("x")))))(input)?;
		Ok((input, (Mode::ZPX, v)))
	}

	/// Parses a zero page operation with Y offset
	fn zpy(input: &str) -> IResult<&str, u16, LexerError> {
		let (input, v) = terminated(int, ws(preceded(ws(char(',')), ws(tag_no_case("y")))))(input)?;
		Ok((input, (Mode::ZPY, v)))
	}
}
