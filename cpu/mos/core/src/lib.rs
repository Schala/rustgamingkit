pub mod disasm;

use bitflags::bitflags;

use std::{
	default::Default,
	fmt::{
		Display,
		Formatter,
		self
	}
};

use rgk_processors_core::{
	Bus,
	Device
};

pub use disasm::*;

/// Offset of program stack
const STACK_ADDR: usize = 256;

/// Offset of stack pointer initiation
const STACK_INIT: usize = 253;

/// Offset of interrupt request vector
const IRQ_ADDR: usize = 65534;

/// Offset of non-maskable interrupt vector
#[allow(dead_code)]
const NMI_ADDR: usize = 65530;

/// Offset of reset vector
const RESET_ADDR: usize = 65532;

bitflags! {
	/// MOS6500 state flags
	struct Status: u8 {
		const C = 1;
		const Z = 2;
		const I = 4;
		const D = 8;
		const B = 16;
		const U = 32;
		const V = 64;
		const N = 128;
	}
}

impl Default for Status {
	fn default() -> Self {
		Status::U
	}
}

impl Display for Status {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if self.contains(Status::C) {
			write!(f, "C")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::Z) {
			write!(f, "Z")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::I) {
			write!(f, "I")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::D) {
			write!(f, "D")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::B) {
			write!(f, "B")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::V) {
			write!(f, "V")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::N) {
			write!(f, "N")
		} else {
			write!(f, "x")
		}
	}
}

/// Address mode
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
	ABS, ABX, ABY, IMM, IMP, IND, IZX, IZY, REL, ZP0, ZPX, ZPY,
}

impl Display for Mode {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match &self {
			Self::ABS => write!(f, "ABS"),
			Self::ABX => write!(f, "ABS X"),
			Self::ABY => write!(f, "ABS Y"),
			Self::IMM => write!(f, "IMM"),
			Self::IMP => write!(f, "IMP"),
			Self::IND => write!(f, "IND"),
			Self::IZX => write!(f, "IND X"),
			Self::IZY => write!(f, "IND Y"),
			Self::REL => write!(f, "REL"),
			Self::ZP0 => write!(f, "ZPG"),
			Self::ZPX => write!(f, "ZPG X"),
			Self::ZPY => write!(f, "ZPG Y"),
		}
	}
}

/// MOS6500 registers
#[derive(Clone, Copy, Debug)]
pub struct Registers {
	/// accumulator
	a: u8,

	/// state flags
	p: Status,

	/// general purpose
	x: u8,

	/// general purpose
	y: u8,

	/// program counter, actually 16 bit
	pc: usize,

	/// stack pointer, 8 bit
	s: usize,
}

impl Display for Registers {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "P: {}", self.p)?;
		writeln!(f, "PC: ${:04X}", self.pc)?;
		writeln!(f, "A: ${:02X}", self.a)?;
		writeln!(f, "X: ${:02X}", self.x)?;
		writeln!(f, "Y: ${:02X}", self.y)?;
		writeln!(f, "SP: ${:02X}", self.s)
	}
}

/// MOS6500 cache
#[derive(Clone, Copy, Debug)]
pub struct Cache {
	/// last fetched byte
	data: u8,

	/// remaining cycles on current operation
	cycles: u8,

	/// last fetched opcode's associated mode
	mode: Mode,

	/// last relative address is actually 1 byte, but this avoids casting every use
	rel_addr: usize,

	/// last fetched opcode, actually 1 byte, but this avoids casting every use
	opcode: usize,

	/// last absolute address, actually 2 bytes, but this avoids casting every use
	abs_addr: usize,
}

impl Display for Cache {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Last fetched byte: ${:X}", self.data)?;
		writeln!(f, "Last fetched opcode: ${:X}", self.opcode)?;
		writeln!(f, "Cycles remaining: ${:X}", self.cycles)?;
		writeln!(f, "Last fetched absolute address: ${:X}", self.abs_addr)?;
		writeln!(f, "Last fetched relative address: {}", self.rel_addr as i8)
	}
}

/// The CPU itself
#[derive(Clone, Debug)]
pub struct MOS6500 {
	bus: Box<Bus>,
	regs: Registers,
	cache: Cache,
}

impl MOS6500 {
	/// Initialises a new MOS6500, given a bus pointer
	pub fn new(bus: Box<Bus>) -> MOS6500 {
		MOS6500 {
			bus,
			regs: Registers {
				a: 0,
				p: Status::default(),
				x: 0,
				y: 0,
				pc: RESET_ADDR,
				s: STACK_INIT,
			},
			cache: Cache {
				data: 0,
				cycles: 0,
				mode: Mode::IMP,
				abs_addr: 0,
				rel_addr: 0,
				opcode: 0,
			},
		}
	}

	/// Add additional cycles to the current operation
	#[inline]
	fn add_cycles(&mut self, value: u8) {
		self.cache.cycles += value;
	}

	fn branch(&mut self) {
		self.add_cycles(1);
		self.set_abs(self.get_counter() + self.get_rel_addr());

		// need an additional cycle if different page
		if self.get_abs_hi() != (self.get_counter() & 0xFF00) {
			self.add_cycles(1);
		}

		// jump to the address
		self.set_counter(self.get_abs_addr());
	}

	/// Checks specified status flag(s)
	const fn check_flag(&self, flag: Status) -> bool {
		self.regs.p.contains(flag)
	}

	/// Assign to accumulator, or write to bus, depending on the address mode
	#[inline]
	fn check_mode(&mut self, value: u16) {
		if self.get_mode() == Mode::IMP {
			self.set_a((value & 255) as u8);
		} else {
			self.write_last((value & 255) as u8);
		}
	}

	/// Check for page change
	const fn check_page(&self, addr: usize) -> u8 {
		if self.get_abs_hi() != (addr & 0xFF00) {
			1
		} else {
			0
		}
	}

	pub fn clock(&mut self) {
		if self.get_cycles() == 0 {
			// always set unused flag
			self.set_flag(Status::U, true);

			// get and increment the counter
			self.cache.opcode = self.get_u8(self.get_counter().into()).into();
			self.incr();

			match self.get_opcode() {
				0 => {
					let mode_cycles = self.imp();
					let op_cycles = self.brk();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				1 => {
					let mode_cycles = self.izx();
					let op_cycles = self.ora();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				2 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				3 => {
					let mode_cycles = self.izx();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				4 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				5 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.ora();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				6 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.asl();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				7 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				8 => {
					let mode_cycles = self.imp();
					let op_cycles = self.php();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				9 => {
					let mode_cycles = self.imm();
					let op_cycles = self.ora();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				10 => {
					let mode_cycles = self.imp();
					let op_cycles = self.asl();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				11 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				12 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				13 => {
					let mode_cycles = self.abs();
					let op_cycles = self.ora();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				14 => {
					let mode_cycles = self.abs();
					let op_cycles = self.asl();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				15 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				16 => {
					let mode_cycles = self.rel();
					let op_cycles = self.bpl();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				17 => {
					let mode_cycles = self.izy();
					let op_cycles = self.ora();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				18 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				19 => {
					let mode_cycles = self.izy();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				20 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				21 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.ora();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				22 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.asl();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				23 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				24 => {
					let mode_cycles = self.imp();
					let op_cycles = self.clc();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				25 => {
					let mode_cycles = self.aby();
					let op_cycles = self.ora();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				26 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				27 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				28 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				29 => {
					let mode_cycles = self.abx();
					let op_cycles = self.ora();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				30 => {
					let mode_cycles = self.abx();
					let op_cycles = self.asl();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				31 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				32 => {
					let mode_cycles = self.abs();
					let op_cycles = self.jsr();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				33 => {
					let mode_cycles = self.izx();
					let op_cycles = self.and();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				34 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				35 => {
					let mode_cycles = self.izx();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				36 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.bit();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				37 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.and();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				38 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.rol();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				39 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				40 => {
					let mode_cycles = self.imp();
					let op_cycles = self.plp();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				41 => {
					let mode_cycles = self.imm();
					let op_cycles = self.and();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				42 => {
					let mode_cycles = self.imp();
					let op_cycles = self.rol();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				43 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				44 => {
					let mode_cycles = self.abs();
					let op_cycles = self.bit();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				45 => {
					let mode_cycles = self.abs();
					let op_cycles = self.and();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				46 => {
					let mode_cycles = self.abs();
					let op_cycles = self.rol();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				47 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				48 => {
					let mode_cycles = self.rel();
					let op_cycles = self.bmi();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				49 => {
					let mode_cycles = self.izy();
					let op_cycles = self.and();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				50 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				51 => {
					let mode_cycles = self.izy();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				52 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				53 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.and();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				54 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.rol();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				55 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				56 => {
					let mode_cycles = self.imp();
					let op_cycles = self.sec();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				57 => {
					let mode_cycles = self.aby();
					let op_cycles = self.and();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				58 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				59 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				60 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				61 => {
					let mode_cycles = self.abx();
					let op_cycles = self.and();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				62 => {
					let mode_cycles = self.abx();
					let op_cycles = self.rol();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				63 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				64 => {
					let mode_cycles = self.imp();
					let op_cycles = self.rti();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				65 => {
					let mode_cycles = self.izx();
					let op_cycles = self.eor();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				66 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				67 => {
					let mode_cycles = self.izx();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				68 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				69 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.eor();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				70 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.lsr();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				71 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				72 => {
					let mode_cycles = self.imp();
					let op_cycles = self.pha();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				73 => {
					let mode_cycles = self.imm();
					let op_cycles = self.eor();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				74 => {
					let mode_cycles = self.imp();
					let op_cycles = self.lsr();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				75 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				76 => {
					let mode_cycles = self.abs();
					let op_cycles = self.jmp();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				77 => {
					let mode_cycles = self.abs();
					let op_cycles = self.eor();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				78 => {
					let mode_cycles = self.abs();
					let op_cycles = self.lsr();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				79 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				80 => {
					let mode_cycles = self.rel();
					let op_cycles = self.bvc();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				81 => {
					let mode_cycles = self.izy();
					let op_cycles = self.eor();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				82 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				83 => {
					let mode_cycles = self.izy();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				84 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				85 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.eor();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				86 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.lsr();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				87 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				88 => {
					let mode_cycles = self.imp();
					let op_cycles = self.cli();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				89 => {
					let mode_cycles = self.aby();
					let op_cycles = self.eor();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				90 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				91 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				92 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				93 => {
					let mode_cycles = self.abx();
					let op_cycles = self.eor();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				94 => {
					let mode_cycles = self.abx();
					let op_cycles = self.lsr();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				95 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				96 => {
					let mode_cycles = self.imp();
					let op_cycles = self.rts();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				97 => {
					let mode_cycles = self.izx();
					let op_cycles = self.adc();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				98 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				99 => {
					let mode_cycles = self.izx();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				100 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				101 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.adc();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				102 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.ror();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				103 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				104 => {
					let mode_cycles = self.imp();
					let op_cycles = self.pla();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				105 => {
					let mode_cycles = self.imm();
					let op_cycles = self.adc();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				106 => {
					let mode_cycles = self.imp();
					let op_cycles = self.ror();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				107 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				108 => {
					let mode_cycles = self.ind();
					let op_cycles = self.jmp();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				109 => {
					let mode_cycles = self.abs();
					let op_cycles = self.adc();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				110 => {
					let mode_cycles = self.abs();
					let op_cycles = self.ror();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				111 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				112 => {
					let mode_cycles = self.rel();
					let op_cycles = self.bvs();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				113 => {
					let mode_cycles = self.izy();
					let op_cycles = self.adc();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				114 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				115 => {
					let mode_cycles = self.izy();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				116 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				117 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.adc();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				118 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.ror();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				119 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				120 => {
					let mode_cycles = self.imp();
					let op_cycles = self.sei();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				121 => {
					let mode_cycles = self.aby();
					let op_cycles = self.adc();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				122 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				123 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				124 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				125 => {
					let mode_cycles = self.abx();
					let op_cycles = self.adc();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				126 => {
					let mode_cycles = self.abx();
					let op_cycles = self.ror();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				127 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				128 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				129 => {
					let mode_cycles = self.izx();
					let op_cycles = self.sta();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				130 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				131 => {
					let mode_cycles = self.izx();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				132 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.sty();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				133 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.sta();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				134 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.stx();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				135 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				136 => {
					let mode_cycles = self.imp();
					let op_cycles = self.dey();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				137 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				138 => {
					let mode_cycles = self.imp();
					let op_cycles = self.txa();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				139 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				140 => {
					let mode_cycles = self.abs();
					let op_cycles = self.sty();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				141 => {
					let mode_cycles = self.abs();
					let op_cycles = self.sta();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				142 => {
					let mode_cycles = self.abs();
					let op_cycles = self.stx();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				143 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				144 => {
					let mode_cycles = self.rel();
					let op_cycles = self.bcc();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				145 => {
					let mode_cycles = self.izy();
					let op_cycles = self.sta();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				146 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				147 => {
					let mode_cycles = self.izy();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				148 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.sty();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				149 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.sta();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				150 => {
					let mode_cycles = self.zpy();
					let op_cycles = self.stx();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				151 => {
					let mode_cycles = self.zpy();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				152 => {
					let mode_cycles = self.imp();
					let op_cycles = self.tya();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				153 => {
					let mode_cycles = self.aby();
					let op_cycles = self.sta();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				154 => {
					let mode_cycles = self.imp();
					let op_cycles = self.txs();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				155 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				156 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				157 => {
					let mode_cycles = self.abx();
					let op_cycles = self.sta();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				158 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				159 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				160 => {
					let mode_cycles = self.imm();
					let op_cycles = self.ldy();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				161 => {
					let mode_cycles = self.izx();
					let op_cycles = self.lda();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				162 => {
					let mode_cycles = self.imm();
					let op_cycles = self.ldx();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				163 => {
					let mode_cycles = self.izx();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				164 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.ldy();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				165 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.lda();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				166 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.ldx();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				167 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				168 => {
					let mode_cycles = self.imp();
					let op_cycles = self.tay();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				169 => {
					let mode_cycles = self.imm();
					let op_cycles = self.lda();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				170 => {
					let mode_cycles = self.imp();
					let op_cycles = self.tax();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				171 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				172 => {
					let mode_cycles = self.abs();
					let op_cycles = self.ldy();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				173 => {
					let mode_cycles = self.abs();
					let op_cycles = self.lda();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				174 => {
					let mode_cycles = self.abs();
					let op_cycles = self.ldx();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				175 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				176 => {
					let mode_cycles = self.rel();
					let op_cycles = self.bcs();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				177 => {
					let mode_cycles = self.izy();
					let op_cycles = self.lda();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				178 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				179 => {
					let mode_cycles = self.izy();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				180 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.ldy();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				181 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.lda();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				182 => {
					let mode_cycles = self.zpy();
					let op_cycles = self.ldx();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				183 => {
					let mode_cycles = self.zpy();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				184 => {
					let mode_cycles = self.imp();
					let op_cycles = self.clv();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				185 => {
					let mode_cycles = self.aby();
					let op_cycles = self.lda();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				186 => {
					let mode_cycles = self.imp();
					let op_cycles = self.tsx();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				187 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				188 => {
					let mode_cycles = self.abx();
					let op_cycles = self.ldy();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				189 => {
					let mode_cycles = self.abx();
					let op_cycles = self.lda();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				190 => {
					let mode_cycles = self.aby();
					let op_cycles = self.ldx();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				191 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				192 => {
					let mode_cycles = self.imm();
					let op_cycles = self.cpy();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				193 => {
					let mode_cycles = self.izx();
					let op_cycles = self.cmp();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				194 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				195 => {
					let mode_cycles = self.izx();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				196 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.cpy();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				197 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.cmp();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				198 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.dec();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				199 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				200 => {
					let mode_cycles = self.imp();
					let op_cycles = self.iny();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				201 => {
					let mode_cycles = self.imm();
					let op_cycles = self.cmp();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				202 => {
					let mode_cycles = self.imp();
					let op_cycles = self.dex();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				203 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				204 => {
					let mode_cycles = self.abs();
					let op_cycles = self.cpy();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				205 => {
					let mode_cycles = self.abs();
					let op_cycles = self.cmp();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				206 => {
					let mode_cycles = self.abs();
					let op_cycles = self.dec();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				207 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				208 => {
					let mode_cycles = self.rel();
					let op_cycles = self.bne();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				209 => {
					let mode_cycles = self.izy();
					let op_cycles = self.cmp();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				210 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				211 => {
					let mode_cycles = self.izy();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				212 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				213 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.cmp();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				214 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.dec();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				215 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				216 => {
					let mode_cycles = self.imp();
					let op_cycles = self.cld();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				217 => {
					let mode_cycles = self.aby();
					let op_cycles = self.cmp();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				218 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				219 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				220 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				221 => {
					let mode_cycles = self.abx();
					let op_cycles = self.cmp();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				222 => {
					let mode_cycles = self.abx();
					let op_cycles = self.dec();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				223 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				224 => {
					let mode_cycles = self.imm();
					let op_cycles = self.cpx();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				225 => {
					let mode_cycles = self.izx();
					let op_cycles = self.sbc();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				226 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				227 => {
					let mode_cycles = self.izx();
					let op_cycles = self.nop();
					self.add_cycles(8 + (mode_cycles & op_cycles));
				},
				228 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.cpx();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				229 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.sbc();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				230 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.inc();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				231 => {
					let mode_cycles = self.zp0();
					let op_cycles = self.nop();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				232 => {
					let mode_cycles = self.imp();
					let op_cycles = self.inx();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				233 => {
					let mode_cycles = self.imm();
					let op_cycles = self.sbc();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				234 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				235 => {
					let mode_cycles = self.imm();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				236 => {
					let mode_cycles = self.abs();
					let op_cycles = self.cpx();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				237 => {
					let mode_cycles = self.abs();
					let op_cycles = self.sbc();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				238 => {
					let mode_cycles = self.abs();
					let op_cycles = self.inc();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				239 => {
					let mode_cycles = self.abs();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				240 => {
					let mode_cycles = self.rel();
					let op_cycles = self.beq();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				241 => {
					let mode_cycles = self.izy();
					let op_cycles = self.sbc();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				242 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(mode_cycles & op_cycles);
				},
				243 => {
					let mode_cycles = self.izy();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				244 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				245 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.sbc();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				246 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.inc();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				247 => {
					let mode_cycles = self.zpx();
					let op_cycles = self.nop();
					self.add_cycles(6 + (mode_cycles & op_cycles));
				},
				248 => {
					let mode_cycles = self.imp();
					let op_cycles = self.sed();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				249 => {
					let mode_cycles = self.aby();
					let op_cycles = self.sbc();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				250 => {
					let mode_cycles = self.imp();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				251 => {
					let mode_cycles = self.aby();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				252 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				253 => {
					let mode_cycles = self.abx();
					let op_cycles = self.sbc();
					self.add_cycles(4 + (mode_cycles & op_cycles));
				},
				254 => {
					let mode_cycles = self.abx();
					let op_cycles = self.inc();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				255 => {
					let mode_cycles = self.abx();
					let op_cycles = self.nop();
					self.add_cycles(7 + (mode_cycles & op_cycles));
				},
				_ => unreachable!(),
			}

			// always set unused flag
			self.set_flag(Status::U, true);
		}

		self.cache.cycles -= 1;
	}

	/// Fetch data from an operation
	fn fetch(&mut self) -> u8 {
		if self.get_mode() != Mode::IMP {
			self.set_data(self.get_u8(self.get_abs_addr().into()));
		}

		self.get_data()
	}

	/// Fetch data from an operation as 16-bit
	#[inline]
	fn fetch16(&mut self) -> u16 {
		self.fetch() as u16
	}

	/// Fetch address
	#[inline]
	fn fetch_addr(&mut self) -> usize {
		(((self.fetch() as u16) << 8) | (self.fetch() as u16)).into()
	}

	/// Gets the accumulator register value
	const fn get_a(&self) -> u8 {
		self.regs.a
	}

	/// Gets the accumulator register value as a 16-bit value
	const fn get_a16(&self) -> u16 {
		self.regs.a as u16
	}

	/// Gets cached absolute address
	const fn get_abs_addr(&self) -> usize {
		self.cache.abs_addr
	}

	/// Gets cached absolute address' high byte
	const fn get_abs_hi(&self) -> usize {
		self.get_abs_addr() & 0xFF00
	}

	/// Gets the carry bit
	const fn get_carry(&self) -> u16 {
		(self.check_flag(Status::C) as u16) & 1
	}

	/// Gets the program counter register value
	const fn get_counter(&self) -> usize {
		self.regs.pc
	}

	/// Gets the remaining cycle count
	const fn get_cycles(&self) -> u8 {
		self.cache.cycles
	}

	/// Gets the currently cached data byte
	const fn get_data(&self) -> u8 {
		self.cache.data
	}

	/// Gets the currently cached data byte as 16-bit
	const fn get_data16(&self) -> u16 {
		self.cache.data as u16
	}

	/// Retrieves the currently cached address mode
	const fn get_mode(&self) -> Mode {
		self.cache.mode
	}

	/// Gets the currently cached opcode
	const fn get_opcode(&self) -> usize {
		self.cache.opcode as usize
	}

	/// Retrieve the registry state flag bits
	#[inline]
	const fn get_p_bits(&self) -> u8 {
		self.regs.p.bits()
	}

		/// Gets cached relative address
	const fn get_rel_addr(&self) -> usize {
		self.cache.rel_addr
	}

	/// Gets the stack pointer register value
	const fn get_sp(&self) -> usize {
		self.regs.s
	}

	/// Gets the X register value
	const fn get_x(&self) -> u8 {
		self.regs.x
	}

	/// Gets the X register value as 16-bit
	const fn get_x16(&self) -> u16 {
		self.get_x() as u16
	}

	/// Gets X register value as a zero page address
	const fn get_x_zp_addr(&self) -> usize {
		self.get_x() as usize
	}

	/// Gets the Y register value
	const fn get_y(&self) -> u8 {
		self.regs.y
	}

	/// Gets the X register value as 16-bit
	const fn get_y16(&self) -> u16 {
		self.get_y() as u16
	}

	/// Gets Y register value as a zero page address
	const fn get_y_zp_addr(&self) -> usize {
		self.get_y() as usize
	}

	/// Get zero-page address
	#[inline]
	fn get_zp_addr(&self, address: usize) -> usize {
		self.get_u8(address) as usize
	}

	/// Increment program pc registry by 1
	#[inline]
	fn incr(&mut self) {
		self.regs.pc += 1;
	}

	/// Interrupts the execution state
	#[inline]
	fn interrupt(&mut self, new_abs_addr: usize, new_cycles: u8) {
		// write the counter's current value to stack
		self.stack_write_addr(self.get_counter());

		// write p register to stack too
		self.set_flag(Status::B, false);
		self.set_flag(Status::U, true);
		self.set_flag(Status::I, true);
		self.stack_write(self.get_p_bits());

		// get the new pc value
		self.set_abs(new_abs_addr);
		let addr = self.fetch_addr();
		self.set_counter(addr);

		self.cache.cycles = new_cycles;
	}

	/// Reads an address from the RAM
	#[inline]
	fn read_addr(&self, addr: usize) -> usize {
		self.bus.get_u16_le(addr) as usize
	}

	/// Reads a byte from the ROM
	#[inline]
	fn read_rom(&mut self) -> u8 {
		let data = self.get_u8(self.get_counter());
		self.incr();

		data
	}

	/// Reads an address from the ROM
	#[inline]
	fn read_rom_addr(&mut self) -> usize {
		u16::from_le_bytes([self.read_rom(), self.read_rom()]).into()
	}

	/// Reads an 8-bit address from the ROM
	#[inline]
	fn read_rom_zp_addr(&mut self) -> usize {
		self.read_rom().into()
	}

	/// Resets the regs and cache
	pub fn reset(&mut self) {
		self.set_a(0);
		self.set_flag(Status::default(), true);
		self.set_x(0);
		self.set_y(0);
		self.set_sp(STACK_INIT);

		self.set_abs(RESET_ADDR);
		let addr = self.fetch_addr();
		self.set_counter(addr);

		self.cache.rel_addr = 0;
		self.set_abs(0);
		self.set_data(0);

		self.cache.cycles = 8;
	}

	/// Sets a register value
	#[inline]
	fn set_a(&mut self, value: u8) {
		self.regs.a = value;
	}

	/// Sets cached absolute address
	#[inline]
	fn set_abs(&mut self, value: usize) {
		self.cache.abs_addr = value;
	}

	/// Sets program counter register value
	#[inline]
	fn set_counter(&mut self, value: usize) {
		self.regs.pc = value;
	}

	/// Sets cached data
	#[inline]
	fn set_data(&mut self, value: u8) {
		self.cache.data = value;
	}

	/// Sets status register flag
	#[inline]
	fn set_flag(&mut self, flags: Status, condition: bool) {
		self.regs.p.set(flags, condition);
	}

	/// Set carry, negative, and/or zero bits of state flags register, given a value
	#[inline]
	fn set_flags_cnz(&mut self, value: u16) {
		self.set_flag(Status::C, value > 255);
		self.set_flags_nz(value);
	}

	/// Set negative and/or zero bits of state flags register, given a value
	#[inline]
	fn set_flags_nz(&mut self, value: u16) {
		self.set_if_0(value);
		self.set_if_neg(value);
	}

	/// Set the flag if the value is zero
	#[inline]
	fn set_if_0(&mut self, value: u16) {
		self.set_flag(Status::Z, (value & 255) == 0)
	}

	/// Set the flag if the value is negative
	#[inline]
	fn set_if_neg(&mut self, value: u16) {
		self.set_flag(Status::N, value & 128 != 0)
	}

	/// Set cached address mode. Only address mode functions should use this!
	#[inline]
	fn set_mode(&mut self, mode: Mode) {
		self.cache.mode = mode;
	}

	/// Sets stack pointer
	#[inline]
	fn set_sp(&mut self, value: usize) {
		self.regs.s = value;
	}

	/// Sets X register value
	#[inline]
	fn set_x(&mut self, value: u8) {
		self.regs.x = value;
	}

	/// Sets Y register value
	#[inline]
	fn set_y(&mut self, value: u8) {
		self.regs.y = value;
	}

	/// Convenience function to read from stack
	#[inline]
	fn stack_read(&mut self) -> u8 {
		self.regs.s += 1;
		self.get_u8(STACK_ADDR + self.get_sp())
	}

	/// Reads an address from stack
	#[inline]
	fn stack_read_addr(&mut self) -> usize {
		u16::from_le_bytes([self.stack_read(), self.stack_read()]).into()
	}

	/// Convenience function to write to stack
	#[inline]
	fn stack_write(&mut self, data: u8) {
		self.write(STACK_ADDR + self.get_sp(), &[data]);
		self.regs.s -= 1;
	}

	/// Writes an address to stack
	#[inline]
	fn stack_write_addr(&mut self, addr: usize) {
		self.stack_write(((addr & 0xFF00) >> 8) as u8);
		self.stack_write((addr & 255) as u8);
	}

	/// Writes to the last absolute address
	#[inline]
	fn write_last(&mut self, data: u8) {
		self.write(self.get_abs_addr(), &[data]);
	}

	// --- INTERRUPTS

	/// Sends an interrupt request if able
	#[inline]
	fn irq(&mut self) {
		if !self.check_flag(Status::I) {
			self.interrupt(IRQ_ADDR, 7);
		}
	}

	#[inline]
	/// Sends a non-maskable interrupt
	fn nmi(&mut self) {
		self.interrupt(NMI_ADDR, 8);
	}

	// --- ADDRESS MODES

	/// Absolute address mode
	fn abs(&mut self) -> u8 {
		self.set_mode(Mode::ABS);
		let addr = self.read_rom_addr();
		self.set_abs(addr);

		0
	}

	/// Absolute address mode with X register offset
	fn abx(&mut self) -> u8 {
		self.set_mode(Mode::ABX);

		let addr = self.read_rom_addr();
		self.set_abs(addr + self.get_x_zp_addr());

		self.check_page(addr)
	}

	/// Absolute address mode with Y register offset
	fn aby(&mut self) -> u8 {
		self.set_mode(Mode::ABY);

		let addr = self.read_rom_addr();
		self.set_abs(addr + self.get_y_zp_addr());

		self.check_page(addr)
	}

		/// Immediate address mode
	fn imm(&mut self) -> u8 {
		self.set_mode(Mode::IMM);
		self.incr();
		self.set_abs(self.get_counter());
		0
	}

	/// Implied address mode
	fn imp(&mut self) -> u8 {
		self.set_mode(Mode::IMP);
		self.set_data(self.get_a());
		0
	}

	/// Indirect address mode (pointer access)
	fn ind(&mut self) -> u8 {
		self.set_mode(Mode::IND);

		let ptr = self.read_rom_addr();

		if (ptr & 255) == 255 {
			// page boundary hardware bug
			self.set_abs(self.read_addr(ptr));
		} else {
			// normal behavior
			self.set_abs(self.read_addr(ptr));
		}

		0
	}

	/// Indirect address mode of zero-page with X register offset
	fn izx(&mut self) -> u8 {
		self.set_mode(Mode::IZX);

		let t = self.read_rom_zp_addr();
		let lo = self.get_zp_addr((t + self.get_x_zp_addr()) & 255);
		let hi = self.get_zp_addr((t + self.get_x_zp_addr() + 1) & 255);

		self.set_abs((hi << 8) | lo);
		0
	}

	/// Indirect address mode of zero-page with Y register offset
	fn izy(&mut self) -> u8 {
		self.set_mode(Mode::IZY);

		let t = self.read_rom_zp_addr();
		let lo = self.get_zp_addr(t & 255);
		let hi = self.get_zp_addr((t + 1) & 255);

		self.set_abs(((hi << 8) | lo) + self.get_y_zp_addr());

		if self.get_abs_hi() != (hi << 8) { 1 } else { 0 }
	}

	/// Relative address mode (branching instructions)
	fn rel(&mut self) -> u8 {
		self.set_mode(Mode::REL);
		self.cache.rel_addr = self.read_rom_zp_addr();

		// check_flag for signed bit
		if self.get_rel_addr() & 128 != 0 {
			self.cache.rel_addr |= 0xFF00;
		}

		0
	}

	/// Zero-page address mode
	fn zp0(&mut self) -> u8 {
		self.set_mode(Mode::ZP0);
		let addr = self.read_rom_zp_addr();
		self.set_abs(addr);
		self.incr();
		self.cache.abs_addr &= 255;
		0
	}

	/// Zero-page address mode with X register offset
	fn zpx(&mut self) -> u8 {
		self.set_mode(Mode::ZPX);
		let addr = self.read_rom_zp_addr();
		self.set_abs(addr + self.get_x_zp_addr());
		self.cache.abs_addr &= 255;
		0
	}

	/// Zero-page address mode with Y register offset
	fn zpy(&mut self) -> u8 {
		self.set_mode(Mode::ZPY);
		let addr = self.read_rom_zp_addr();
		self.set_abs(addr + self.get_y_zp_addr());
		self.cache.abs_addr &= 255;
		0
	}

	// --- OPERATIONS

	/// Addition with carry
	fn adc(&mut self) -> u8 {
		let fetch = self.fetch16();
		let tmp = self.get_a16() + (fetch + self.get_carry());

		self.set_flags_cnz(tmp);

		self.set_flag(Status::V, !(((self.get_a16() ^ self.get_data16()) &
			(self.get_a16() ^ tmp)) & 128) == 0);

		self.set_a((tmp & 255) as u8);

		1
	}

	/// Bitwise and
	fn and(&mut self) -> u8 {
		self.regs.a &= self.fetch();
		self.set_flags_nz(self.get_a16());

		1
	}

	/// Arithmetical left shift
	fn asl(&mut self) -> u8 {
		let tmp = self.fetch16() << 1;
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Branching if carry clear
	fn bcc(&mut self) -> u8 {
		if !self.check_flag(Status::C) {
			self.branch();
		}

		0
	}

	/// Branching if carry
	fn bcs(&mut self) -> u8 {
		if self.check_flag(Status::C) {
			self.branch();
		}

		0
	}

	/// Branching if equal (zero)
	fn beq(&mut self) -> u8 {
		if self.check_flag(Status::Z) {
			self.branch();
		}

		0
	}

	/// Bit test
	fn bit(&mut self) -> u8 {
		let fetch = self.fetch16();
		self.set_if_0(self.get_a16() & fetch);
		self.set_if_neg(self.get_data16());
		self.set_flag(Status::V, self.get_data() & 64 != 0);

		0
	}

	/// Branching if negative
	fn bmi(&mut self) -> u8 {
		if self.check_flag(Status::N) {
			self.branch();
		}

		0
	}

	/// Branching if not equal (non-zero)
	fn bne(&mut self) -> u8 {
		if !self.check_flag(Status::Z) {
			self.branch();
		}

		0
	}

	/// Branching if positive
	fn bpl(&mut self) -> u8 {
		if !self.check_flag(Status::N) {
			self.branch();
		}

		0
	}

	/// Program-sourced interrupt.
	fn brk(&mut self) -> u8 {
		// This differs slightly from self.interrupt()

		self.incr();
		self.set_flag(Status::I, true);
		self.stack_write_addr(self.get_counter());
		self.set_flag(Status::B, true);
		self.stack_write(self.get_p_bits());
		self.set_flag(Status::B, false);
		self.set_counter(self.read_addr(IRQ_ADDR));

		0
	}

	/// Branching if overflow
	fn bvc(&mut self) -> u8 {
		if self.check_flag(Status::V) {
			self.branch();
		}

		0
	}

	/// Branching if not overflow
	fn bvs(&mut self) -> u8 {
		if !self.check_flag(Status::V) {
			self.branch();
		}

		0
	}

	/// Clear carry bit
	fn clc(&mut self) -> u8 {
		self.set_flag(Status::C, false);
		0
	}

	/// Clear decimal bit
	fn cld(&mut self) -> u8 {
		self.set_flag(Status::D, false);
		0
	}

	/// Clear interrupt disable bit
	fn cli(&mut self) -> u8 {
		self.set_flag(Status::I, false);
		0
	}

	/// Clear overflow bit
	fn clv(&mut self) -> u8 {
		self.set_flag(Status::V, false);
		0
	}

	/// Compare with accumulator
	fn cmp(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_flag(Status::C, self.get_a() >= fetch);
		self.set_flags_nz(self.get_a16() - self.get_data16());

		1
	}

	/// Compare with X
	fn cpx(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_flag(Status::C, self.get_a() >= fetch);
		self.set_flags_nz(self.get_x16() - self.get_data16());

		1
	}

	/// Compare with Y
	fn cpy(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_flag(Status::C, self.get_y() >= fetch);
		self.set_flags_nz(self.get_y16() - self.get_data16());

		1
	}

	/// Decrement accumulator register
	fn dec(&mut self) -> u8 {
		let tmp = self.fetch() - 1;
		self.write_last(tmp);
		self.set_flags_nz(tmp.into());

		0
	}

	/// Decrement X register
	fn dex(&mut self) -> u8 {
		self.regs.x -= 1;
		self.set_flags_nz(self.get_x16());

		0
	}

	/// Decrement Y register
	fn dey(&mut self) -> u8 {
		self.regs.y -= 1;
		self.set_flags_nz(self.get_y16());

		0
	}

	/// Exclusive or
	fn eor(&mut self) -> u8 {
		self.regs.a ^= self.fetch();
		self.set_flags_nz(self.get_a16());

		1
	}

	/// Increment accumulator register
	fn inc(&mut self) -> u8 {
		let tmp = self.fetch() + 1;
		self.write_last(tmp);
		self.set_flags_nz(tmp.into());

		0
	}

	/// Increment X register
	fn inx(&mut self) -> u8 {
		self.regs.x += 1;
		self.set_flags_nz(self.get_x16());

		0
	}

	/// Increment Y register
	fn iny(&mut self) -> u8 {
		self.regs.y += 1;
		self.set_flags_nz(self.get_y16());

		0
	}

	/// Jump to address
	fn jmp(&mut self) -> u8 {
		self.set_counter(self.get_abs_addr());
		0
	}

	/// Jump to subroutine
	fn jsr(&mut self) -> u8 {
		self.stack_write_addr(self.get_counter());
		self.set_counter(self.get_abs_addr());
		0
	}

	/// Load into accumulator
	fn lda(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_a(fetch);
		self.set_flags_nz(self.get_a16());
		1
	}

	/// Load into X
	fn ldx(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_x(fetch);
		self.set_flags_nz(self.get_x16());
		1
	}

	/// Load into Y
	fn ldy(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_y(fetch);
		self.set_flags_nz(self.get_y16());
		1
	}

	/// Logical right shift
	fn lsr(&mut self) -> u8 {
		let tmp = self.fetch16() >> 1;
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// No operation, illegal opcode filler
	fn nop(&self) -> u8 {
		match self.get_opcode() {
			28 | 60 | 92 | 124 | 220 | 252 => 1,
			_ => 0,
		}
	}

	/// Bitwise or
	fn ora(&mut self) -> u8 {
		self.regs.a |= self.fetch();
		self.set_flags_nz(self.get_a16());

		1
	}

	/// Push accumulator register to the stack
	fn pha(&mut self) -> u8 {
		self.stack_write(self.get_a());
		0
	}

	/// Push state register to the stack
	fn php(&mut self) -> u8 {
		self.set_flag(Status::B, true);
		self.set_flag(Status::U, true);
		self.stack_write(self.get_p_bits());
		self.set_flag(Status::B, false);
		self.set_flag(Status::U, false);

		0
	}

	/// Pop accumulator register from the stack
	fn pla(&mut self) -> u8 {
		let b = self.stack_read();
		self.set_a(b);
		self.set_flags_nz(self.get_a16());

		0
	}

	/// Pop state register from the stack
	fn plp(&mut self) -> u8 {
		self.regs.p = Status::from_bits_truncate(self.stack_read());
		self.set_flag(Status::U, true);

		0
	}

	/// Bit rotate left
	fn rol(&mut self) -> u8 {
		let tmp = self.fetch16().rotate_left(1);
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Bit rotate right
	fn ror(&mut self) -> u8 {
		let tmp = self.fetch16().rotate_right(1);
		self.set_flag(Status::C, (self.get_data() & 1) != 0);
		self.set_flags_nz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Restores state from interrupt
	fn rti(&mut self) -> u8 {
		// restore state flags
		self.regs.p = Status::from_bits_truncate(self.stack_read());
		self.regs.p &= !Status::B;
		self.regs.p &= !Status::U;

		// and counter
		let addr = self.stack_read_addr();
		self.set_counter(addr);

		0
	}

	/// Return from subroutine
	fn rts(&mut self) -> u8 {
		let addr = self.stack_read_addr();
		self.set_counter(addr);
		0
	}

	/// Subtraction with carry
	fn sbc(&mut self) -> u8 {
		let value = self.fetch16() ^ 255; // invert the value
		let tmp = self.get_a16() + value + self.get_carry();

		self.set_flags_cnz(tmp);
		self.set_flag(Status::V, (tmp ^ self.get_a16() & (tmp ^ value)) & 128 != 0);
		self.set_a((tmp & 255) as u8);

		1
	}

	/// Set carry bit
	fn sec(&mut self) -> u8 {
		self.set_flag(Status::C, true);
		0
	}

	/// Set decimal bit
	fn sed(&mut self) -> u8 {
		self.set_flag(Status::D, true);
		0
	}

	/// Set interrupt disable bit
	fn sei(&mut self) -> u8 {
		self.set_flag(Status::I, true);
		0
	}

	/// Store accumulator at address
	fn sta(&mut self) -> u8 {
		self.write_last(self.get_a());
		0
	}

	/// Store X at address
	fn stx(&mut self) -> u8 {
		self.write_last(self.get_x());
		0
	}

	/// Store Y at address
	fn sty(&mut self) -> u8 {
		self.write_last(self.get_y());
		0
	}

	/// Transfer accumulator to X
	fn tax(&mut self) -> u8 {
		self.set_x(self.get_a());
		self.set_flags_nz(self.get_x16());
		0
	}

	/// Transfer accumulator to Y
	fn tay(&mut self) -> u8 {
		self.set_y(self.get_a());
		self.set_flags_nz(self.get_y16());
		0
	}

	/// Transfer stack pointer to X
	fn tsx(&mut self) -> u8 {
		self.set_x(self.get_sp() as u8);
		self.set_flags_nz(self.get_x16());
		0
	}

	/// Transfer X to accumulator
	fn txa(&mut self) -> u8 {
		self.set_a(self.get_x());
		self.set_flags_nz(self.get_a16());
		0
	}

	/// Transfer X to stack pointer
	fn txs(&mut self) -> u8 {
		self.set_sp(self.get_x_zp_addr());
		0
	}

	/// Transfer Y to accumulator
	fn tya(&mut self) -> u8 {
		self.set_a(self.get_y());
		self.set_flags_nz(self.get_a16());
		0
	}
}

impl Device for MOS6500 {
	fn read(&self, address: usize, length: usize) -> Vec<u8> {
		self.bus.read(address, length)
	}

	fn write(&mut self, address: usize, data: &[u8]) {
		self.bus.write(address, data);
	}
}
