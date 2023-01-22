use bitflags::bitflags;
use indexmap::IndexMap;

use std::{
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
	NMI_ADDR,
	RESET_ADDR,
	STACK_ADDR
};

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
	pub struct DisassemblerConfig: u8 {
		/// Output decimal values instead of hex
		const DECIMAL = 1;

		/// Show offsets
		const OFFSETS = 2;

		/// Display lowercase
		const LOWERCASE = 4;

		/// Auto-generate labels
		const AUTO_LABELS = 8;
	}
}

/// Region type
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum RegionType {
	/// Region is labelled
	#[default]
	Label = 0,

	/// Region is a function
	Function,

	/// Region is data and should not be interpreted as an operation
	Data,
}

/// A region of disassembled code
#[derive(Clone, Debug)]
pub struct Region {
	kind: RegionType,
	label: String,
	refs: Vec<usize>,
}

impl Region {
	pub fn new(kind: RegionType, label: &str) -> Region {
		Region {
			kind,
			label: label.to_owned(),
			refs: vec![],
		}
	}

	#[inline]
	fn add_ref(&mut self, addr: usize) {
		self.refs.push(addr);
	}

	#[inline]
	pub fn get_label(&self) -> &str {
		self.label.as_str()
	}

	#[inline]
	pub fn get_refs(&self) -> &[usize] {
		self.refs.as_ref()
	}
}

/// The disassembler itself
#[derive(Clone, Debug)]
pub struct Disassembler {
	cfg: DisassemblerConfig,
	bus: Rc<Bus>,
	disasm: IndexMap<usize, String>,
	rgns: IndexMap<usize, Region>,
}

impl Disassembler {
	/// Sets up the disassembler
	pub fn new(bus: Rc<Bus>, cfg: Option<DisassemblerConfig>) -> Disassembler {
		Disassembler {
			cfg: if let Some(conf) = cfg { conf } else { DisassemblerConfig::default() },
			bus,
			disasm: IndexMap::new(),
			rgns: Self::init_regions(),
		}
	}

	/// Associates the specified region with the specified offset
	#[inline]
	pub fn add_region(&mut self, offset: usize, r: Region) {
		self.rgns.insert(offset, r);
	}

	/// Analyses the code by running an emulation, starting at the given offset
	/*pub fn analyze(&mut self, offset: usize) {
		let opbyte = self.bus.get_u8(*offset);


	}*/

	/// Adds one disassembled operation
	pub fn from_operation(&mut self, offset: &mut usize) {

		// If the region is data, do nothing
		if let Some(r) = self.rgns.get(offset) {
			if r.kind == RegionType::Data {
				return;
			}
		}

		let start = *offset;
		let opbyte = self.bus.get_u8(*offset) as usize;
		let opcode = &OPCODES[opbyte];
		let mut code = opcode.mnemonic.to_owned();
		*offset += 1;

		match opcode.mode {
			Mode::IMM => {
				if self.cfg.contains(DisassemblerConfig::DECIMAL) {
					code += format!(" #{}", self.bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" #${:02X}", self.bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::ZP0 => {
				if self.cfg.contains(DisassemblerConfig::DECIMAL) {
					code += format!(" {}", self.bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" ${:02X}", self.bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::ZPX => {
				if self.cfg.contains(DisassemblerConfig::DECIMAL) {
					code += format!(" {}, X", self.bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" ${:02X}, X", self.bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::ZPY => {
				if self.cfg.contains(DisassemblerConfig::DECIMAL) {
					code += format!(" {}, Y", self.bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" ${:02X}, Y", self.bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::IZX => {
				if self.cfg.contains(DisassemblerConfig::DECIMAL) {
					code += format!(" ({}, X)", self.bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" (${:02X}, X)", self.bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::IZY => {
				if self.cfg.contains(DisassemblerConfig::DECIMAL) {
					code += format!(" ({}, Y)", self.bus.get_u8(*offset)).as_str();
				} else {
					code += format!(" (${:02X}, Y)", self.bus.get_u8(*offset)).as_str();
				}
				*offset += 1;
			},
			Mode::ABS => {
				let addr = (self.bus.get_u16_le(*offset) + 2) as usize;

				if let Some(r) = self.rgns.get_mut(&addr) {
					if opbyte == 32 || opbyte == 76 {
						code += format!(" {}", r.label).as_str();
						r.add_ref(start);
					}
				} else {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" {}", self.bus.get_u16_le(*offset)).as_str();
					} else {
						code += format!(" ${:04X}", self.bus.get_u16_le(*offset)).as_str();
					}
				}

				*offset += 2;
			},
			Mode::ABX => {
				if self.cfg.contains(DisassemblerConfig::DECIMAL) {
					code += format!(" {}, X", self.bus.get_u16_le(*offset)).as_str();
				} else {
					code += format!(" ${:04X}, X", self.bus.get_u16_le(*offset)).as_str();
				}
				*offset += 2;
			},
			Mode::ABY => {
				if self.cfg.contains(DisassemblerConfig::DECIMAL) {
					code += format!(" {}, Y", self.bus.get_u16_le(*offset)).as_str();
				} else {
					code += format!(" ${:04X}, Y", self.bus.get_u16_le(*offset)).as_str();
				}
				*offset += 2;
			},
			Mode::IND => {
				let addr = (self.bus.get_u16_le(*offset) + 2) as usize;

				if let Some(r) = self.rgns.get_mut(&addr) {
					code += format!(" {}", r.label).as_str();
					r.add_ref(start);
				} else {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" ({})", self.bus.get_u16_le(*offset)).as_str();
					} else {
						code += format!(" (${:04X})", self.bus.get_u16_le(*offset)).as_str();
					}
				}
				*offset += 2;
			},
			Mode::REL => {
				let addr = ((*offset as i32) + (self.bus.get_i8(*offset) as i32) + 1) as usize;

				if let Some(r) = self.rgns.get_mut(&addr) {
					code += format!(" {}", r.label).as_str();
					r.add_ref(start);
				} else {
					if self.cfg.contains(DisassemblerConfig::DECIMAL) {
						code += format!(" {}", addr as u16).as_str();
					} else {
						code += format!(" ${:04X}", addr as u16).as_str();
					}
				}
				*offset += 1;
			},
			_ => (), // implied address mode
		}

		if self.cfg.contains(DisassemblerConfig::LOWERCASE) {
			code = code.to_lowercase();
		}

		self.disasm.insert(start, code);
	}

	/// Adds a range of disassembled operations
	pub fn from_range(&mut self, start: usize, end: usize) {
		let mut offset = start;

		if self.cfg.contains(DisassemblerConfig::AUTO_LABELS) {
			self.generate_regions(start, end);
		}

		while offset < end {
			self.from_operation(&mut offset);
		}
	}

	/// Generates and adds labels between the given start and end offsets
	fn generate_regions(&mut self, start: usize, end: usize) {
		let mut offset = start;

		while offset < end {
			let op = self.bus.get_u8(offset) as usize;
			let mode = OPCODES[op].mode;
			offset += 1;

			match mode {
				Mode::REL => {
					let addr = ((offset as i32) + (self.bus.get_i8(offset) as i32) + 1) as u16;
					self.rgns.insert(addr as usize, Region::new(RegionType::Label,
						format!("L_{:04X}", addr).as_str()));

					offset += 1;
				},
				Mode::ABS | Mode::IND => {
					let addr = self.bus.get_u16_le(offset) + 2;

					match op {
						32 => { // JSR, label is a function
							self.rgns.insert(addr as usize,
								Region::new(RegionType::Function, format!("F_{:04X}", addr).as_str()));
						},
						76 | 108 => { // JMP
							self.rgns.insert(addr as usize,
								Region::new(RegionType::Label, format!("L_{:04X}", addr).as_str()));
						},
						_ => (),
					}

					offset += 2;
				},
				_ => (),
			}
		}

		self.rgns.sort_keys()
	}

	/// Returns the code at the given offset, if any
	pub fn get_code_at_offset(&self, offset: usize) -> Option<&str> {
		if let Some(s) = self.disasm.get(&offset) {
			Some(s.as_str())
		} else {
			None
		}
	}

	/// Returns the label at the given offset, if any
	pub fn get_label_at_offset(&self, offset: usize) -> Option<&str> {
		if let Some(r) = self.rgns.get(&offset) {
			Some(r.label.as_str())
		} else {
			None
		}
	}

	/// Returns a reference to the map of regions
	#[inline]
	pub fn get_regions(&self) -> &IndexMap<usize, Region> {
		&self.rgns
	}

	/// Initialises the region map with hardcoded vectors
	fn init_regions() -> IndexMap<usize, Region> {
		let mut map = IndexMap::new();

		map.insert(IRQ_ADDR, Region::new(RegionType::Data, "IRQ"));
		map.insert(NMI_ADDR, Region::new(RegionType::Data, "NMI"));
		map.insert(RESET_ADDR, Region::new(RegionType::Data, "RESET"));
		map.insert(STACK_ADDR, Region::new(RegionType::Data, "STACK"));

		map
	}
}

impl Display for Disassembler {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		for (o, c) in self.disasm.iter() {
			if let Some(r) = self.rgns.get(o) {
				write!(f, "\n\t{}:\t\t; REFS: ", r.label)?;
				for x in r.refs.iter() {
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
		bus.write(666, &data);
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
		bus.write(666, &data);
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
		bus.write(0, &data[..]);

		let cfg = DisassemblerConfig::AUTO_LABELS | DisassemblerConfig::OFFSETS;
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
		bus.write(32768, &mario[16..32784]);

		let cfg = DisassemblerConfig::AUTO_LABELS | DisassemblerConfig::OFFSETS;
		let mut da = Disassembler::new(Rc::new(bus), Some(cfg));
		da.from_range(32768, 65530);

		let rm = da.get_regions();

		for (o, r) in rm.iter() {
			println!("{:04X}: {}", o, r.label);
		}
	}*/

	#[test]
	fn test_disassemble_nes_rom() {
		let mario = include_bytes!("/home/admin/Downloads/Super Mario Bros (PC10).nes");

		let mut bus = Bus::new(65536);
		bus.write(32768, &mario[16..32784]);

		let cfg = DisassemblerConfig::AUTO_LABELS | DisassemblerConfig::OFFSETS;
		let mut da = Disassembler::new(Rc::new(bus), Some(cfg));

		// https://datacrystal.romhacking.net/wiki/Super_Mario_Bros.:ROM_map
		/*da.add_region(0x85DF, Region::new(RegionType::Data, "SKY_BG_UNDERWATER"));
		da.add_region(0x85E0, Region::new(RegionType::Data, "SKY_BG_OVERWORLD"));
		da.add_region(0x85E1, Region::new(RegionType::Data, "BG_UNDERGROUND"));
		da.add_region(0x85E2, Region::new(RegionType::Data, "BG_CASTLE"));
		da.add_region(0x85E3, Region::new(RegionType::Data, "BG_NIGHT"));
		da.add_region(0x85E4, Region::new(RegionType::Data, "BG_WINTER"));
		da.add_region(0x85E5, Region::new(RegionType::Data, "BG_WINTER_NIGHT"));
		da.add_region(0x85E6, Region::new(RegionType::Data, "BG_6_3"));
		da.add_region(0x8C00, Region::new(RegionType::Data, "CHR_IDX_NORMAL_BLOCK"));
		da.add_region(0x8C08, Region::new(RegionType::Data, "CHR_IDX_BLOCK_8C08"));
		da.add_region(0x8C10, Region::new(RegionType::Data, "CHR_IDX_STAR_BLOCK"));
		da.add_region(0x8C12, Region::new(RegionType::Data, "CHR_IDX_POWERUP_BLOCK"));
		da.add_region(0x8C16, Region::new(RegionType::Data, "CHR_IDX_BEANSTALK_BLOCK"));
		da.add_region(0x8C1C, Region::new(RegionType::Data, "CHR_IDX_MULTICOIN_BLOCK"));
		da.add_region(0x8C20, Region::new(RegionType::Data, "CHR_IDX_BLOCK_8C20"));
		da.add_region(0x8C9C, Region::new(RegionType::Data, "CHR_IDX_COIN_QBLOCK"));
		da.add_region(0x8CA0, Region::new(RegionType::Data, "CHR_IDX_POWERUP_QBLOCK"));*/

		da.from_range(32768, 65530);

		println!("{}", da);
	}
}
