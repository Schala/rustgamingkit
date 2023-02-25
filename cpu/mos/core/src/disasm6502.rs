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

use crate::{
	IRQ_ADDR,
	Mode,
	MOS6502,
	NMI_ADDR,
	RES_ADDR,
	STACK_ADDR,
};

use rgk_processors_core::{
	Bus,
	DeviceBase,
	Disassembler,
	Region,
	RegionFlags,
	RegionMap,
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

#[derive(Debug)]
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
	rgns: RegionMap,
}

impl MOS6502Disassembler {
	/// Sets up the disassembler
	pub fn new(bus: Rc<RefCell<Bus>>, cfg: Option<DisassemblerConfig>) -> Self {
		Self {
			cfg: if let Some(conf) = cfg { conf } else { DisassemblerConfig::default() },
			bus,
			disasm: IndexMap::new(),
			rgns: RegionMap::new(),
		}
	}
}

impl Disassembler for MOS6502Disassembler {
	type ProcDev = MOS6502;

	fn add_region(&mut self, address: usize, region: Region) {
		if !self.region_exists(address) {
			self.rgns.insert(address, region);
		}
	}

	#[allow(unused_assignments)] // Rust will bitch about `code` assignment in conditionals
	fn analyze(&mut self, offset: &mut usize) -> (usize, String) {
		let start = *offset;
		let mut code = "".to_owned();
		let mut do_break = true;

		if let Some(r) = self.rgns.get(&*offset) {
			if r.is_ptr() {
				code.push('*');
			}

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
				&RegionType::Pointer => {
					let addr = self.bus.borrow().get_u16_le(*offset) as usize;
					if let Some(p) = self.rgns.get(&addr) {
						code += format!("\t{}", p.get_label()).as_str();
					} else {
						if self.cfg.contains(DisassemblerConfig::DECIMAL) {
							code += format!("\t{}", self.bus.borrow().get_u16_le(*offset)).as_str();
						} else {
							code += format!("\t${:04X}", self.bus.borrow().get_u16_le(*offset)).as_str();
						}
					}

					*offset += 2;
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
					if opbyte == 32 || opbyte == 76 {
						let addr = self.bus.borrow().get_u16_le(*offset) as usize;

						if let Some(r) = self.rgns.get(&addr) {
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

					if let Some(r) = self.rgns.get(&addr) {
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

					if let Some(r) = self.rgns.get(&addr) {
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
		}

		(start, code)
	}

	#[allow(unused_mut)] // Rust bitches it's unused when r.add_ref() is used
	fn generate_regions(&mut self, dev: &mut Self::ProcDev, start: usize) {
		let mut offset = start;
		let mut jumps = vec![];
		let mut counters = vec![];
		let mut end_reached = false;

		loop {
			if end_reached {
				end_reached = false;

				// return from a function
				if let Some(i) = counters.pop() {
					offset = i;
				} else {
					// ...or analyse the next entry in the jump table
					if self.region_exists(offset) {
						if let Some(i) = jumps.pop() {
							offset = i;
						} else {
							break;
						}
					}
				}
			}

			let op = dev.get_u8(offset) as usize;
			dbg!(offset, &OPCODES[op]);
			offset += 1;

			match op {
				// relative, continue to next byte but cache the jump
				16 | 48 | 80 | 112 | 144 | 176 | 208 | 240 => {
					let addr = ((offset as i32) + (dev.get_i8(offset) as i32) + 1) as u16;

					if self.region_exists(addr as usize) {
						let addrsz = addr as usize;
						if let Some(mut r) = self.rgns.get_mut(&addrsz) {
							r.add_ref(offset - 1);
						}
					} else {
						let mut r = Region::new(0, RegionType::Label,  RegionFlags::default(),
							format!("LAB_{:04X}", addr).as_str());
						r.add_ref(offset - 1);
						self.add_region(addr as usize, r);
					}

					jumps.push(addr as usize);
					offset += 1;
				},

				32 => { // JSR, label is a function, cache the counter at the next instruction
					let addr = dev.get_u16_le(offset) as usize;

					if self.region_exists(addr) {
						if let Some(mut r) = self.rgns.get_mut(&addr) {
							r.label_to_fn(Some(format!("FUN_{:04X}", addr & 65535).as_str()));
							r.add_ref(offset - 1);
						}
					} else {
						let mut r = Region::new(0, RegionType::Function,  RegionFlags::default(),
							format!("FUN_{:04X}", addr & 65535).as_str());
						r.add_ref(offset - 1);
						self.add_region(addr, r);
					}

					counters.push(offset + 2);
					offset = addr;
					dbg!(offset, &counters);
				},

				// returns
				64 | 96 => {
					end_reached = true;
					dbg!(&counters);
				}

				76 | 108 => { // JMP absolute or indirect
					let addr = dev.get_u16_le(offset);

					if self.region_exists(addr as usize) {
						let addrsz = addr as usize;
						if let Some(mut r) = self.rgns.get_mut(&addrsz) {
							r.add_ref(offset - 1);
						}
					} else {
						let mut r = Region::new(0, RegionType::Label, RegionFlags::default(),
							format!("LAB_{:04X}", addr).as_str());
						r.add_ref(offset - 1);
						self.add_region(addr as usize, r);
					}

					offset = addr as usize;
				},

				// zero page
				4..=7 | 36..=39 | 68..=71 | 100..=103 | 132..=135 | 164..=167 | 196..=199 | 228..=231 => {
					let addr = dev.get_u8(offset) as usize;

					if self.region_exists(addr) {
						if let Some(mut r) = self.rgns.get_mut(&addr) {
							r.add_ref(offset - 1);
						}
					} else {
						let mut r = Region::new(0, RegionType::Data, RegionFlags::default(),
							format!("DAT_{:04X}", addr).as_str());
						r.add_ref(offset - 1);
						self.add_region(addr as usize, r);
					}

					offset += 1;
				},

				// absolute
				12..=14 | 25 | 27..=30 | 44..=47 | 57 | 59..=63 | 77..=79 | 89 | 91..=95 | 109..=111 | 121 | 123..=127 |
				140..=143 | 153 | 155..=159 | 172..=175 | 185 | 187..=191 | 204..=207 | 217 | 219..=223 | 236..=239 |
				249 | 251..=255 => {
					let addr = dev.get_u16_le(offset) as usize;

					if self.region_exists(addr) {
						if let Some(mut r) = self.rgns.get_mut(&addr) {
							r.add_ref(offset - 1);
						}
					} else {
						let mut r = Region::new(0, RegionType::Data, RegionFlags::default(),
							format!("DAT_{:04X}", addr).as_str());
						r.add_ref(offset - 1);
						self.add_region(addr as usize, r);
					}

					offset += 2;
				},


				// rest
				1 | 9 | 17 | 19..=23 | 33 | 35 | 41 | 43 | 49 | 51..=55 | 65 | 67 | 73 | 75 | 81 | 83..=87 | 97 | 99 |
				105 | 107 | 113 | 115..=119 | 128..=131 | 137 | 139 | 145 | 147..=151 | 160..=163 | 169 | 171 | 177 |
				179..=183 | 192..=199 | 201 | 203 | 209 | 211..=215 | 224..=231 | 233 | 235 | 241 | 243..=247 => {
					offset += 1;
				},

				// implied
				_ => (),
			}
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
		if let Some(r) = self.rgns.get(&offset) {
			Some(r.get_label().to_owned())
		} else {
			None
		}
	}

	fn region_exists(&self, offset: usize) -> bool {
		self.rgns.contains_key(&offset)
	}

	fn run(&mut self, dev: &mut Self::ProcDev) {
		// add hardware vectors
		let irq_vec = dev.get_u16_le(IRQ_ADDR) as usize;
		let nmi_vec = dev.get_u16_le(NMI_ADDR) as usize;
		let res_vec = dev.get_u16_le(RES_ADDR) as usize;

		let mut irq_r = Region::new(0, RegionType::Function, RegionFlags::default(),
			format!("FUN_{:04X}", irq_vec).as_str());
		irq_r.add_ref(IRQ_ADDR);

		let mut nmi_r = Region::new(0, RegionType::Function, RegionFlags::default(),
			format!("FUN_{:04X}", nmi_vec).as_str());
		nmi_r.add_ref(NMI_ADDR);

		let mut res_r = Region::new(0, RegionType::Function, RegionFlags::default(),
			format!("FUN_{:04X}", res_vec).as_str());
		res_r.add_ref(RES_ADDR);

		self.add_regions(RegionMap::from([
			(0, Region::new(256, RegionType::Section, RegionFlags::default(), "ZERO_PAGE")),
			(IRQ_ADDR, Region::new(2, RegionType::Pointer, RegionFlags::PTR, "IRQ")),
			(NMI_ADDR, Region::new(2, RegionType::Pointer, RegionFlags::PTR, "NMI")),
			(RES_ADDR, Region::new(2, RegionType::Pointer, RegionFlags::PTR, "RES")),
			(STACK_ADDR, Region::new(256, RegionType::Unsigned8, RegionFlags::ARRAY, "STACK")),
			(irq_vec, irq_r),
			(nmi_vec, nmi_r),
			(res_vec, res_r),
		]));

		// walk through and process
		self.generate_regions(dev, res_vec);
		self.generate_regions(dev, irq_vec);
		self.generate_regions(dev, nmi_vec);
		self.rgns.sort_keys();
	}
}

impl Display for MOS6502Disassembler {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		for (o, c) in self.disasm.iter() {
			if let Some(r) = self.rgns.get(o) {
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
		let mut cpu = MOS6502::new(Rc::new(RefCell::new(bus)));
		let mut da = MOS6502Disassembler::new(cpu.get_bus(), Some(cfg));

		da.run(&mut cpu);

		println!("{}", da);
	}
}
