use bitflags::bitflags;
use indexmap::IndexMap;

use std::{
	cell::RefCell,
	fmt::{
		Display,
		Formatter,
		self
	},
	rc::Rc
};

use crate::Mode;

use rgk_processors_core::{
	Bus,
	Device,
	DeviceBase,
	Disassembler,
	RegionType
};

static OPCODES: [Opcode; 256] = [
	Opcode { mode: Mode::IMP, mnemonic: "BRK" },
	Opcode { mode: Mode::IZX, mnemonic: "ORA" },
	Opcode { mode: Mode::IMP, mnemonic: "NOP" },
	Opcode { mode: Mode::IZX, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPG, mnemonic: "ORA" },
	Opcode { mode: Mode::ZPG, mnemonic: "ASL" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
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
	Opcode { mode: Mode::ZPG, mnemonic: "BIT" },
	Opcode { mode: Mode::ZPG, mnemonic: "AND" },
	Opcode { mode: Mode::ZPG, mnemonic: "ROL" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
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
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPG, mnemonic: "EOR" },
	Opcode { mode: Mode::ZPG, mnemonic: "LSR" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
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
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
	Opcode { mode: Mode::ZPG, mnemonic: "ADC" },
	Opcode { mode: Mode::ZPG, mnemonic: "ROR" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
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
	Opcode { mode: Mode::ZPG, mnemonic: "STY" },
	Opcode { mode: Mode::ZPG, mnemonic: "STA" },
	Opcode { mode: Mode::ZPG, mnemonic: "STX" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
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
	Opcode { mode: Mode::ZPG, mnemonic: "LDY" },
	Opcode { mode: Mode::ZPG, mnemonic: "LDA" },
	Opcode { mode: Mode::ZPG, mnemonic: "LDX" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
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
	Opcode { mode: Mode::ZPG, mnemonic: "CPY" },
	Opcode { mode: Mode::ZPG, mnemonic: "CMP" },
	Opcode { mode: Mode::ZPG, mnemonic: "DEC" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
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
	Opcode { mode: Mode::ZPG, mnemonic: "CPX" },
	Opcode { mode: Mode::ZPG, mnemonic: "SBC" },
	Opcode { mode: Mode::ZPG, mnemonic: "INC" },
	Opcode { mode: Mode::ZPG, mnemonic: "NOP" },
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
	pub struct DisassemblerConfig: u8 {
		/// Output decimal values instead of hex
		const DECIMAL = 1;

		/// Show offsets
		const OFFSETS = 2;

		/// Display lowercase
		const LOWERCASE = 4;

		/// Apply type prefixes to appropriate regions
		const TYPES = 8;
	}
}

/// The disassembler itself
#[derive(Clone, Debug)]
pub struct MOS6502Disassembler {
	cfg: DisassemblerConfig,
	bus: Rc<RefCell<Bus>>,
	disasm: IndexMap<usize, String>,
}

impl MOS6502Disassembler {
	/// Sets up the disassembler
	pub fn new(bus: Rc<RefCell<Bus>>, cfg: Option<DisassemblerConfig>) -> Self {
		Self {
			cfg: if let Some(conf) = cfg { conf } else { DisassemblerConfig::default() },
			bus,
			disasm: IndexMap::new(),
		}
	}
}

impl Disassembler for MOS6502Disassembler {
	#[allow(unused_assignments)] // Rust will bitch about `code` assignment in conditionals
	fn analyze(&mut self, offset: &mut usize) {
		let start = *offset;
		let mut code = "".to_owned();
		let mut do_break = true;

		if let Some(r) = self.bus.borrow().get_region(*offset) {
			let r = r.borrow();

			match r.get_type() {
				&RegionType::Signed8 => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!("i8\t{}", self.bus.borrow().get_u8(*offset)).as_str();
					} else {
						code += format!("i8\t${:02X}", self.bus.borrow().get_u8(*offset)).as_str();
					}
					*offset += 1;

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					do_break = false;
				},
				&RegionType::Unsigned8 => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!("u8\t{}", self.bus.borrow().get_u8(*offset)).as_str();
					} else {
						code += format!("u8\t${:02X}", self.bus.borrow().get_u8(*offset)).as_str();
					}
					*offset += 1;

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					do_break = false;
				},
				&RegionType::Signed16 => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!("i16\t{}", self.bus.borrow().get_i16_le(*offset)).as_str();
					} else {
						code += format!("i16\t${:04X}", self.bus.borrow().get_i16_le(*offset)).as_str();
					}
					*offset += 2;

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					do_break = false;
				},
				&RegionType::Unsigned16 => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!("u16\t{}", self.bus.borrow().get_u16_le(*offset)).as_str();
					} else {
						code += format!("u16\t${:04X}", self.bus.borrow().get_u16_le(*offset)).as_str();
					}
					*offset += 2;

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					do_break = false;
				},
				_ => (),
			}
		}

		if do_break { // operation
			let opbyte = self.bus.borrow().get_u8(*offset) as usize;
			let opcode = &OPCODES[opbyte];
			code = opcode.mnemonic.to_owned();

			if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
				code = code.to_lowercase();
			}

			*offset += 1;

			match opcode.mode {
				Mode::IMM => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" #{}", self.bus.borrow().get_u8(*offset)).as_str();
					} else {
						code += format!(" #${:02X}", self.bus.borrow().get_u8(*offset)).as_str();
						if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
							code = code.to_lowercase();
						}
					}

					*offset += 1;
				},
				Mode::ZPG => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" {}", self.bus.borrow().get_u8(*offset)).as_str();
					} else {
						code += format!(" ${:02X}", self.bus.borrow().get_u8(*offset)).as_str();
						if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
							code = code.to_lowercase();
						}
					}

					*offset += 1;
				},
				Mode::ZPX => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" {}, X", self.bus.borrow().get_u8(*offset)).as_str();
					} else {
						code += format!(" ${:02X}, X", self.bus.borrow().get_u8(*offset)).as_str();
					}

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					*offset += 1;
				},
				Mode::ZPY => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" {}, Y", self.bus.borrow().get_u8(*offset)).as_str();
					} else {
						code += format!(" ${:02X}, Y", self.bus.borrow().get_u8(*offset)).as_str();
					}

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					*offset += 1;
				},
				Mode::IZX => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" ({}, X)", self.bus.borrow().get_u8(*offset)).as_str();
					} else {
						code += format!(" (${:02X}, X)", self.bus.borrow().get_u8(*offset)).as_str();
					}

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					*offset += 1;
				},
				Mode::IZY => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" ({}, Y)", self.bus.borrow().get_u8(*offset)).as_str();
					} else {
						code += format!(" (${:02X}, Y)", self.bus.borrow().get_u8(*offset)).as_str();
					}

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					*offset += 1;
				},
				Mode::ABS => {
					let addr = (self.bus.borrow().get_u16_le(*offset) + 2) as usize;

					if let Some(r) = self.bus.borrow().get_region(addr) {
						let r = r.borrow();
						if opbyte == 32 || opbyte == 76 {
							code += format!(" {}", r.get_label()).as_str();
						}
					} else {
						if self.cfg.contains(DisassemblerConfig::DECIMAL) {
							code += format!(" {}", self.bus.borrow().get_u16_le(*offset)).as_str();
						} else {
							code += format!(" ${:04X}", self.bus.borrow().get_u16_le(*offset)).as_str();

							if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
								code = code.to_lowercase();
							}
						}
					}

					*offset += 2;
				},
				Mode::ABX => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" {}, X", self.bus.borrow().get_u16_le(*offset)).as_str();
					} else {
						code += format!(" ${:04X}, X", self.bus.borrow().get_u16_le(*offset)).as_str();
					}

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					*offset += 2;
				},
				Mode::ABY => {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" {}, Y", self.bus.borrow().get_u16_le(*offset)).as_str();
					} else {
						code += format!(" ${:04X}, Y", self.bus.borrow().get_u16_le(*offset)).as_str();
					}

					if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
						code = code.to_lowercase();
					}

					*offset += 2;
				},
				Mode::IND => {
					let addr = (self.bus.borrow().get_u16_le(*offset) + 2) as usize;

					if let Some(r) = self.bus.borrow().get_region(addr) {
						let r = r.borrow();
						code += format!(" {}", r.get_label()).as_str();
					} else {
						if self.cfg.contains(DisassemblerConfig::DECIMAL) {
							code += format!(" ({})", self.bus.borrow().get_u16_le(*offset)).as_str();
						} else {
							code += format!(" (${:04X})", self.bus.borrow().get_u16_le(*offset)).as_str();

							if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
								code = code.to_lowercase();
							}
						}
					}
					*offset += 2;
				},
				Mode::REL => {
					let addr = ((*offset as i32) + (self.bus.borrow().get_i8(*offset) as i32) + 1) as usize;

					if let Some(r) = self.bus.borrow().get_region(addr) {
						let r = r.borrow();
						code += format!(" {}", r.get_label()).as_str();
					} else {
						if self.cfg.contains(DisassemblerConfig::DECIMAL) {
							code += format!(" {}", addr as u16).as_str();
						} else {
							code += format!(" ${:04X}", addr as u16).as_str();

							if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
								code = code.to_lowercase();
							}
						}
					}
					*offset += 1;
				},
				_ => (),
			}

			self.disasm.insert(start, code);
		}
	}

	fn get_code_at_offset(&self, offset: usize) -> Option<String> {
		if let Some(s) = self.disasm.get(&offset) {
			Some(s.to_string())
		} else {
			None
		}
	}

	fn get_label_at_offset(&self, offset: usize) -> Option<String> {
		if let Some(r) = self.bus.borrow().get_region(offset) {
			let r = r.borrow();
			Some(r.get_label().to_owned())
		} else {
			None
		}
	}
}

impl Display for MOS6502Disassembler {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		for (o, c) in self.disasm.iter() {
			if let Some(r) = self.bus.borrow().get_region(*o) {
				let r = r.borrow();
				write!(f, "\n\t{}:\t\t; REFS: ", r.get_label())?;
				for x in r.get_refs().iter() {
					write!(f, "{:04X} ", x)?;
				}
				writeln!(f, "")?;
			}

			if self.cfg.contains(DisassemblerConfig::OFFSETS) {
				write!(f, "{:04X}:\t", o)?;
			}
			writeln!(f, "{}", c)?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use rgk_processors_core::{
		Bus,
		Disassembler
	};

	use super::*;
	use crate::MOS6502;

	/*#[test]
	fn test_disassemble() {
		// 100 Doors - https://rosettacode.org/wiki/100_doors#6502_Assembly
		let data = vec![0xa9,0x00,0xa2,0x64,0x95,0xc8,0xca,0xd0,0xfb,0x95,0xc8,0xa0,
			0x01,0xc0,0x65,0xb0,0x12,0x98,0xc9,0x65,0xb0,0x0a,0xaa,0xfe,0x00,0x02,0x84,0x01,0x65,
			0x01,0x90,0xf2,0xc8,0xd0,0xea,0xa2,0x64,0xbd,0x00,0x02,0x29,0x01,0x9d,0x00,0x02,0xca,
			0xd0,0xf5];

		let mut bus = Rc::new(Bus::new(65536));
		self.bus.borrow().write(666, &data);
		let bus = bus;

		let mut da = Disassembler::new(Rc::clone(&bus), None);
		da.from_range(666, 666 + 48);

		println!("{}", da);
	}

	#[test]
	fn test_disassemble_with_config() {
		// 100 Doors - https://rosettacode.org/wiki/100_doors#6502_Assembly
		let data = vec![0xa9,0x00,0xa2,0x64,0x95,0xc8,0xca,0xd0,0xfb,0x95,0xc8,0xa0,
			0x01,0xc0,0x65,0xb0,0x12,0x98,0xc9,0x65,0xb0,0x0a,0xaa,0xfe,0x00,0x02,0x84,0x01,0x65,
			0x01,0x90,0xf2,0xc8,0xd0,0xea,0xa2,0x64,0xbd,0x00,0x02,0x29,0x01,0x9d,0x00,0x02,0xca,
			0xd0,0xf5];

		let mut bus = Rc::new(Bus::new(65536));
		self.bus.borrow().write(666, &data);
		let bus = bus;

		let cfg = DisassemblerConfig::DECIMAL | DisassemblerConfig::OFFSETS |
			DisassemblerConfig::LOWERCASE;
		let mut da = Disassembler::new(Rc::clone(&bus), Some(cfg));
		da.from_range(666, 666 + 48);

		println!("{}", da);
	}*/

	/*#[test]
	fn test_disassemble_with_labels() {
		// 100 Doors - https://rosettacode.org/wiki/100_doors#6502_Assembly
		let data = vec![0xa9,0x00,0xa2,0x64,0x95,0xc8,0xca,0xd0,0xfb,0x95,0xc8,0xa0,
			0x01,0xc0,0x65,0xb0,0x12,0x98,0xc9,0x65,0xb0,0x0a,0xaa,0xfe,0x00,0x02,0x84,0x01,0x65,
			0x01,0x90,0xf2,0xc8,0xd0,0xea,0xa2,0x64,0xbd,0x00,0x02,0x29,0x01,0x9d,0x00,0x02,0xca,
			0xd0,0xf5];

		let mut bus = Bus::new(65536);
		self.bus.borrow().write(0, &data[..]);

		let cfg = DisassemblerConfig::AUTO_REGIONS | DisassemblerConfig::OFFSETS;
		let mut da = Disassembler::new(Rc::new(bus), Some(cfg));
		da.from_range(0, 48);

		println!("{}", da);
	}*/

	/*#[test]
	fn test_generate_regions() {
		// 100 Doors - https://rosettacode.org/wiki/100_doors#6502_Assembly
		/*let data = vec![0xa9,0x00,0xa2,0x64,0x95,0xc8,0xca,0xd0,0xfb,0x95,0xc8,0xa0,
			0x01,0xc0,0x65,0xb0,0x12,0x98,0xc9,0x65,0xb0,0x0a,0xaa,0xfe,0x00,0x02,0x84,0x01,0x65,
			0x01,0x90,0xf2,0xc8,0xd0,0xea,0xa2,0x64,0xbd,0x00,0x02,0x29,0x01,0x9d,0x00,0x02,0xca,
			0xd0,0xf5];*/
		let mario = include_bytes!("/home/admin/Downloads/Super Mario Bros (PC10).nes");

		let mut bus = Bus::new(65536);
		self.bus.borrow().write(32768, &mario[16..32784]);

		let cfg = DisassemblerConfig::AUTO_REGIONS | DisassemblerConfig::OFFSETS;
		let mut da = Disassembler::new(Rc::new(bus), Some(cfg));
		da.from_range(32768, 65530);

		let rm = da.get_regions();

		for (o, r) in rm.iter() {
			println!("{:04X}: {}", o, r.get_label());
		}
	}*/

	#[test]
	fn test_disassemble_nes_rom() {
		let mario = include_bytes!("/home/admin/Downloads/Super Mario Bros (PC10).nes");

		let mut bus = Bus::new(65536);
		bus.write(32768, &mario[16..32784]);

		let cfg = DisassemblerConfig::LOWERCASE | DisassemblerConfig::OFFSETS;
		let cpu = MOS6502::new(Rc::new(RefCell::new(bus)));
		let mut da = MOS6502Disassembler::new(cpu.get_bus(), Some(cfg));

		da.analyze_range(32768, 65536);

		println!("{}", da);
	}
}
