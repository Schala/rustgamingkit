use bitflags::bitflags;

use std::fmt::{
	Display,
	Formatter,
	self
};

use crate::Mode;

use rgk_processors_core::{
	Bus,
	Device
};

static OPCODES: [Opcode; 256] = [
	Opcode { mode: Mode::IMP, mnemonic: "BRK" },
	Opcode { mode: Mode::IZX, mnemonic: "ORA" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "ORA" },
	Opcode { mode: Mode::ZP0, mnemonic: "ASL" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "PHP" },
	Opcode { mode: Mode::IMM, mnemonic: "ORA" },
	Opcode { mode: Mode::IMP, mnemonic: "ASL" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },
	Opcode { mode: Mode::ABS, mnemonic: "ORA" },
	Opcode { mode: Mode::ABS, mnemonic: "ASL" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },

	// 1x
	Opcode { mode: Mode::REL, mnemonic: "BPL" },
	Opcode { mode: Mode::IZY, mnemonic: "ORA" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZY, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "ORA" },
	Opcode { mode: Mode::ZPX, mnemonic: "ASL" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "CLC" },
	Opcode { mode: Mode::ABY, mnemonic: "ORA" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "ORA" },
	Opcode { mode: Mode::ABX, mnemonic: "ASL" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },

	// 2x
	Opcode { mode: Mode::ABS, mnemonic: "JSR" },
	Opcode { mode: Mode::IZX, mnemonic: "AND" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "BIT" },
	Opcode { mode: Mode::ZP0, mnemonic: "AND" },
	Opcode { mode: Mode::ZP0, mnemonic: "ROL" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "PLP" },
	Opcode { mode: Mode::IMM, mnemonic: "AND" },
	Opcode { mode: Mode::IMP, mnemonic: "ROL" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::ABS, mnemonic: "BIT" },
	Opcode { mode: Mode::ABS, mnemonic: "AND" },
	Opcode { mode: Mode::ABS, mnemonic: "ROL" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },

	// 3x
	Opcode { mode: Mode::REL, mnemonic: "BMI" },
	Opcode { mode: Mode::IZY, mnemonic: "AND" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZY, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "AND" },
	Opcode { mode: Mode::ZPX, mnemonic: "ROL" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "SEC" },
	Opcode { mode: Mode::ABY, mnemonic: "AND" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "AND" },
	Opcode { mode: Mode::ABX, mnemonic: "ROL" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },

	// 4x
	Opcode { mode: Mode::IMP, mnemonic: "RTI" },
	Opcode { mode: Mode::IZX, mnemonic: "EOR" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "EOR" },
	Opcode { mode: Mode::ZP0, mnemonic: "LSR" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "PHA" },
	Opcode { mode: Mode::IMM, mnemonic: "EOR" },
	Opcode { mode: Mode::IMP, mnemonic: "LSR" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },
	Opcode { mode: Mode::ABS, mnemonic: "JMP" },
	Opcode { mode: Mode::ABS, mnemonic: "EOR" },
	Opcode { mode: Mode::ABS, mnemonic: "LSR" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },

	// 5x
	Opcode { mode: Mode::REL, mnemonic: "BVC" },
	Opcode { mode: Mode::IZY, mnemonic: "EOR" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZY, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "EOR" },
	Opcode { mode: Mode::ZPX, mnemonic: "LSR" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "CLI" },
	Opcode { mode: Mode::ABY, mnemonic: "EOR" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "EOR" },
	Opcode { mode: Mode::ABX, mnemonic: "LSR" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },

	// 6x
	Opcode { mode: Mode::IMP, mnemonic: "RTS" },
	Opcode { mode: Mode::IZX, mnemonic: "ADC" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "ADC" },
	Opcode { mode: Mode::ZP0, mnemonic: "ROR" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "PLA" },
	Opcode { mode: Mode::IMM, mnemonic: "ADC" },
	Opcode { mode: Mode::IMP, mnemonic: "ROR" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::IND, mnemonic: "JMP" },
	Opcode { mode: Mode::ABS, mnemonic: "ADC" },
	Opcode { mode: Mode::ABS, mnemonic: "ROR" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },

	// 7x
	Opcode { mode: Mode::REL, mnemonic: "BVS" },
	Opcode { mode: Mode::IZY, mnemonic: "ADC" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZY, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "ADC" },
	Opcode { mode: Mode::ZPX, mnemonic: "ROR" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "SEI" },
	Opcode { mode: Mode::ABY, mnemonic: "ADC" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "ADC" },
	Opcode { mode: Mode::ABX, mnemonic: "ROR" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },

	// 8x
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "STA" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "STY" },
	Opcode { mode: Mode::ZP0, mnemonic: "STA" },
	Opcode { mode: Mode::ZP0, mnemonic: "STX" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "DEY" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "TXA" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::ABS, mnemonic: "STY" },
	Opcode { mode: Mode::ABS, mnemonic: "STA" },
	Opcode { mode: Mode::ABS, mnemonic: "STX" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },

	// 9x
	Opcode { mode: Mode::REL, mnemonic: "BCC" },
	Opcode { mode: Mode::IZY, mnemonic: "STA" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZY, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "STY" },
	Opcode { mode: Mode::ZPX, mnemonic: "STA" },
	Opcode { mode: Mode::ZPY, mnemonic: "STX" },
	Opcode { mode: Mode::ZPY, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "TYA" },
	Opcode { mode: Mode::ABY, mnemonic: "STA" },
	Opcode { mode: Mode::IMP, mnemonic: "TXS" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "STA" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },

	// Ax
	Opcode { mode: Mode::IMM, mnemonic: "LDY" },
	Opcode { mode: Mode::IZX, mnemonic: "LDA" },
	Opcode { mode: Mode::IMM, mnemonic: "LDX" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "LDY" },
	Opcode { mode: Mode::ZP0, mnemonic: "LDA" },
	Opcode { mode: Mode::ZP0, mnemonic: "LDX" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "TAY" },
	Opcode { mode: Mode::IMM, mnemonic: "LDA" },
	Opcode { mode: Mode::IMP, mnemonic: "TAX" },
	Opcode { mode: Mode::IMM, mnemonic: "LXA" },
	Opcode { mode: Mode::ABS, mnemonic: "LDY" },
	Opcode { mode: Mode::ABS, mnemonic: "LDA" },
	Opcode { mode: Mode::ABS, mnemonic: "LDX" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },

	// Bx
	Opcode { mode: Mode::REL, mnemonic: "BCS" },
	Opcode { mode: Mode::IZY, mnemonic: "LDA" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZY, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "LDY" },
	Opcode { mode: Mode::ZPX, mnemonic: "LDA" },
	Opcode { mode: Mode::ZPY, mnemonic: "LDX" },
	Opcode { mode: Mode::ZPY, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "CLV" },
	Opcode { mode: Mode::ABY, mnemonic: "LDA" },
	Opcode { mode: Mode::IMP, mnemonic: "TSX" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "LDY" },
	Opcode { mode: Mode::ABX, mnemonic: "LDA" },
	Opcode { mode: Mode::ABY, mnemonic: "LDX" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },

	// Cx
	Opcode { mode: Mode::IMM, mnemonic: "CPY" },
	Opcode { mode: Mode::IZX, mnemonic: "CMP" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "CPY" },
	Opcode { mode: Mode::ZP0, mnemonic: "CMP" },
	Opcode { mode: Mode::ZP0, mnemonic: "DEC" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "INY" },
	Opcode { mode: Mode::IMM, mnemonic: "CMP" },
	Opcode { mode: Mode::IMP, mnemonic: "DEX" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::ABS, mnemonic: "CPY" },
	Opcode { mode: Mode::ABS, mnemonic: "CMP" },
	Opcode { mode: Mode::ABS, mnemonic: "DEC" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },

	// Dx
	Opcode { mode: Mode::REL, mnemonic: "BNE" },
	Opcode { mode: Mode::IZY, mnemonic: "CMP" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZY, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "CMP" },
	Opcode { mode: Mode::ZPX, mnemonic: "DEC" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "CLD" },
	Opcode { mode: Mode::ABY, mnemonic: "CMP" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "CMP" },
	Opcode { mode: Mode::ABX, mnemonic: "DEC" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },

	// Ex
	Opcode { mode: Mode::IMM, mnemonic: "CPX" },
	Opcode { mode: Mode::IZX, mnemonic: "SBC" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZP0, mnemonic: "CPX" },
	Opcode { mode: Mode::ZP0, mnemonic: "SBC" },
	Opcode { mode: Mode::ZP0, mnemonic: "INC" },
	Opcode { mode: Mode::ZP0, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "INX" },
	Opcode { mode: Mode::IMM, mnemonic: "SBC" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IMM, mnemonic: "NOP" },
	Opcode { mode: Mode::ABS, mnemonic: "CPX" },
	Opcode { mode: Mode::ABS, mnemonic: "SBC" },
	Opcode { mode: Mode::ABS, mnemonic: "INC" },
	Opcode { mode: Mode::ABS, mnemonic: "NOP" },

	// Fx
	Opcode { mode: Mode::REL, mnemonic: "BEQ" },
	Opcode { mode: Mode::IZY, mnemonic: "SBC" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZY, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPX, mnemonic: "SBC" },
	Opcode { mode: Mode::ZPX, mnemonic: "INC" },
	Opcode { mode: Mode::ZPX, mnemonic: "NOP" },
	Opcode { mode: Mode::IMP, mnemonic: "SED" },
	Opcode { mode: Mode::ABY, mnemonic: "SBC" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::ABY, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" },
	Opcode { mode: Mode::ABX, mnemonic: "SBC" },
	Opcode { mode: Mode::ABX, mnemonic: "INC" },
	Opcode { mode: Mode::ABX, mnemonic: "NOP" }
];

struct Opcode<'a> {
	mode: Mode,
	mnemonic: &'a str,
}

bitflags! {
	/// Disassembler configuration flags
	#[derive(Default)]
	pub struct DisassemblyConfig: u8 {
		/// Output decimal values instead of hex
		const DECIMAL = 1;

		/// Show offsets
		const OFFSETS = 2;

		/// Display lowercase
		const LOWERCASE = 4;
	}
}

/// Represents one disassembled instruction
#[derive(Clone, Debug)]
pub struct Disassembly {
	cfg: DisassemblyConfig,
	addr: u16,
	code: String,
}

impl Disassembly {
	/// Returns one disassembled operation
	pub fn from_operation(cfg: DisassemblyConfig, bus: &Bus, offset: &mut usize) -> Disassembly {
		let start = *offset;
		let opcode = &OPCODES[bus.get_u8(*offset) as usize];
		let mut code = opcode.mnemonic.to_owned();
		*offset += 1;

		match opcode.mode {
			Mode::IMM => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" #{}", bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" #${:02X}", bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::ZP0 => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" {}", bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" ${:02X}", bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::ZPX => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" {}, X", bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" ${:02X}, X", bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::ZPY => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" {}, Y", bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" ${:02X}, Y", bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::IZX => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" ({}, X)", bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" (${:02X}, X)", bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::IZY => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" ({}, Y)", bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" (${:02X}, Y)", bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::ABS => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" {}", bus.get_u16_le(*offset)).as_str();
				} else {
					code += format!(" ${:04X}", bus.get_u16_le(*offset)).as_str();
				}
				*offset += 2;
			},
			Mode::ABX => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" {}, X", bus.get_u16_le(*offset)).as_str();
				} else {
					code += format!(" ${:04X}, X", bus.get_u16_le(*offset)).as_str();
				}
				*offset += 2;
			},
			Mode::ABY => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" {}, Y", bus.get_u16_le(*offset)).as_str();
				} else {
					code += format!(" ${:04X}, Y", bus.get_u16_le(*offset)).as_str();
				}
				*offset += 2;
			},
			Mode::IND => {
				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" ({})", bus.get_u16_le(*offset)).as_str();
				} else {
					code += format!(" (${:04X})", bus.get_u16_le(*offset)).as_str();
				}
				*offset += 2;
			},
			Mode::REL => {
				let value = (bus.get_i8(*offset) as i16) + 1;
				let rel = (*offset as i16) + value;

				if cfg.contains(DisassemblyConfig::DECIMAL) {
					code += format!(" {}", rel as u16).as_str();
				} else {
					code += format!(" ${:04X}", rel as u16).as_str();
				}
				*offset += 1;
			},
			_ => (), // implied address mode
		}

		if cfg.contains(DisassemblyConfig::LOWERCASE) {
			code = code.to_lowercase();
		}

		Disassembly { cfg, addr: start as u16, code }
	}

	/// Returns a range of disassembled operations in a vector
	pub fn from_range(cfg: DisassemblyConfig, bus: &Bus, offset: usize, end: usize) -> Vec<Disassembly> {
		let mut code = vec![];
		let mut offset = offset;

		while offset < end {
			code.push(Self::from_operation(cfg, &bus, &mut offset));
		}

		code
	}

	/// Returns the disassembly's code
	#[inline]
	pub fn get_code(&self) -> &str {
		self.code.as_str()
	}

	/// Returns the disassembly's offset
	#[inline]
	pub const fn get_offset(&self) -> u16 {
		self.addr
	}
}

impl Display for Disassembly {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if self.cfg.contains(DisassemblyConfig::OFFSETS) {
			write!(f, "{:04X}:\t", self.get_offset())?;
		}
		write!(f, "{}", self.get_code())
	}
}

#[cfg(test)]
mod tests {
	use rgk_processors_core::{
		Bus,
		Device
	};

	use super::*;

	/*#[test]
	fn test_disassemble() {
		// 100 Doors - https://rosettacode.org/wiki/100_doors#6502_Assembly
		let data = vec![0xa9,0x00,0xa2,0x64,0x95,0xc8,0xca,0xd0,0xfb,0x95,0xc8,0xa0,
			0x01,0xc0,0x65,0xb0,0x12,0x98,0xc9,0x65,0xb0,0x0a,0xaa,0xfe,0x00,0x02,0x84,0x01,0x65,
			0x01,0x90,0xf2,0xc8,0xd0,0xea,0xa2,0x64,0xbd,0x00,0x02,0x29,0x01,0x9d,0x00,0x02,0xca,
			0xd0,0xf5];

		let mut bus = Box::new(Bus::new(65536));
		bus.write(666, &data);

		let code = Disassembly::from_range(DisassemblyConfig::default(), &bus, 666, 666 + 48);

		for c in code.iter() {
			println!("{}", c);
		}
	}*/

	#[test]
	fn test_disassemble_with_config() {
		// 100 Doors - https://rosettacode.org/wiki/100_doors#6502_Assembly
		let data = vec![0xa9,0x00,0xa2,0x64,0x95,0xc8,0xca,0xd0,0xfb,0x95,0xc8,0xa0,
			0x01,0xc0,0x65,0xb0,0x12,0x98,0xc9,0x65,0xb0,0x0a,0xaa,0xfe,0x00,0x02,0x84,0x01,0x65,
			0x01,0x90,0xf2,0xc8,0xd0,0xea,0xa2,0x64,0xbd,0x00,0x02,0x29,0x01,0x9d,0x00,0x02,0xca,
			0xd0,0xf5];

		let mut bus = Box::new(Bus::new(65536));
		bus.write(666, &data);

		let cfg = DisassemblyConfig::DECIMAL | DisassemblyConfig::OFFSETS |
			DisassemblyConfig::LOWERCASE;
		let code = Disassembly::from_range(cfg, &bus, 666, 666 + 48);

		for c in code.iter() {
			println!("{}", c);
		}
	}
}
