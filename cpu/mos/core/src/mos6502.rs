use bitflags::bitflags;

use std::{
	cell::RefCell,
	fmt::{
		Display,
		Formatter,
		self
	},
	rc::Rc
};

use rgk_processors_core::{
	Bus,
	Device,
	DeviceBase,
	hexdump,
	Processor
};

use crate::{
	Helper6502,
	IRQ_ADDR,
	ISA6502,
	RES_ADDR
};

/// Offset of program stack
pub const STACK_ADDR: usize = 256;

/// Offset of stack pointer initiation
const STACK_INIT: usize = 253;

bitflags! {
	/// 6502 state flags
	pub struct Status: u8 {
		/// Carry
		const C = 1;

		/// Zero
		const Z = 2;

		/// Disable interrupts
		const I = 4;

		/// Decimal mode
		const D = 8;

		/// Break
		const B = 16;

		/// Unused
		const U = 32;

		/// Overflow
		const V = 64;

		/// Negative
		const N = 128;
	}

	/// CPU flags
	pub struct MOS6502Flags: u8 {
		const DEBUG = 1;
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
	/// Absolute
	ABS,

	/// Absolute with X offset
	ABX,

	/// Absolute with Y offset
	ABY,

	/// Immediate
	IMM,

	/// Implied
	IMP,

	/// Indirect
	IND,

	/// Indirect with zero-page X offset
	IZX,

	/// Indirect with zero-page Y offset
	IZY,

	/// Relative
	REL,

	/// Zero page
	ZPG,

	/// Zero page with X offset
	ZPX,

	/// Zero page with Y offset
	ZPY,
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
			Self::ZPG => write!(f, "ZPG"),
			Self::ZPX => write!(f, "ZPG X"),
			Self::ZPY => write!(f, "ZPG Y"),
		}
	}
}

/// 6502 registers
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

	/// program counter, 16 bit
	pc: usize,

	/// stack pointer, 8 bit
	s: usize,
}

impl Display for Registers {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "P: {}", self.p)?;
		writeln!(f, "PC: ${:04X}\tSP: ${:02X}", self.pc, self.s)?;
		writeln!(f, "A: ${:02X}\tX: ${:02X}, Y: ${:02X}", self.a, self.x, self.y)
	}
}

/// 6502 cache
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
		writeln!(f, "Cycles remaining: {}", self.cycles)?;
		writeln!(f, "Last fetched absolute address: ${:X}", self.abs_addr)?;
		writeln!(f, "Last fetched relative address: {}", self.rel_addr as i8)
	}
}

/// The CPU itself
#[derive(Clone, Debug)]
pub struct MOS6502 {
	bus: Rc<RefCell<Bus>>,
	regs: Registers,
	cache: Cache,
}

impl MOS6502 {
	/// Initialises a new 6502, given a bus pointer
	pub fn new(bus: Rc<RefCell<Bus>>) -> MOS6502 {
		let mut cpu = MOS6502 {
			bus,
			regs: Registers {
				a: 0,
				p: Status::default(),
				x: 0,
				y: 0,
				pc: RES_ADDR,
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
		};

		cpu.reset();
		cpu
	}

	/// Checks specified status flag(s)
	const fn check_flag(&self, flag: Status) -> bool {
		self.regs.p.contains(flag)
	}

	/// Retrieves the currently cached address mode
	const fn get_mode(&self) -> Mode {
		self.cache.mode
	}

	/// Sets status register flag
	fn set_flag(&mut self, flags: Status, condition: bool) {
		self.regs.p.set(flags, condition);
	}

	/// Set cached address mode. Only address mode functions should use this!
	fn set_mode(&mut self, mode: Mode) {
		self.cache.mode = mode;
	}
}

impl Helper6502 for MOS6502 {
	fn add_cycles(&mut self, value: u8) {
		self.cache.cycles += value;
	}

	fn check_mode(&mut self, value: u16) {
		if self.get_mode() == Mode::IMP {
			self.set_a((value & 255) as u8);
		} else {
			self.write_last((value & 255) as u8);
		}
	}

	fn fetch(&mut self) -> u8 {
		if self.get_mode() != Mode::IMP {
			self.set_data(self.get_u8(self.get_abs_addr()));
		}

		self.get_data()
	}

	fn get_0(&self) -> bool {
		self.check_flag(Status::Z)
	}

	fn get_a(&self) -> u8 {
		self.regs.a
	}

	fn get_abs_addr(&self) -> usize {
		self.cache.abs_addr
	}

	fn get_carry(&self) -> bool {
		self.check_flag(Status::C)
	}

	fn get_carry_bit(&self) -> u16 {
		(self.get_carry() as u16) & 1
	}

	fn get_counter(&self) -> usize {
		self.regs.pc
	}

	fn get_cycles(&self) -> u8 {
		self.cache.cycles
	}

	fn get_data(&self) -> u8 {
		self.cache.data
	}

	fn get_neg(&self) -> bool {
		self.check_flag(Status::N)
	}

	fn get_opcode(&self) -> usize {
		self.cache.opcode as usize
	}

	fn get_overflow(&self) -> bool {
		self.check_flag(Status::V)
	}

	fn get_p_bits(&self) -> u8 {
		self.regs.p.bits()
	}

	fn get_rel_addr(&self) -> usize {
		self.cache.rel_addr
	}

	fn get_sp(&self) -> usize {
		self.regs.s
	}

	fn get_x(&self) -> u8 {
		self.regs.x
	}

	fn get_y(&self) -> u8 {
		self.regs.y
	}

	fn interrupt(&mut self, new_abs_addr: usize, new_cycles: u8) {
		// write the counter's current value to stack
		self.stack_write_ptr(self.get_counter());

		// write state register to stack too
		self.set_brk(false);
		self.set_flag(Status::U, true);
		self.set_int(true);
		self.stack_write(self.get_p_bits());

		// get the new counter value
		self.set_abs_addr(new_abs_addr);
		let addr = self.fetch_ptr();
		self.set_counter(addr);

		self.cache.cycles = new_cycles;
	}

	fn set_0_if(&mut self, value: u16) {
		self.set_flag(Status::Z, (value & 255) == 0)
	}

	fn set_a(&mut self, value: u8) {
		self.regs.a = value;
	}

	fn set_abs_addr(&mut self, value: usize) {
		self.cache.abs_addr = value;
	}

	fn set_brk(&mut self, condition: bool) {
		self.set_flag(Status::B, condition);
	}

	fn set_carry_if(&mut self, condition: bool) {
		self.set_flag(Status::C, condition);
	}

	fn set_counter(&mut self, value: usize) {
		self.regs.pc = value;
	}

	fn set_cycles(&mut self, value: u8) {
		self.cache.cycles = value;
	}

	fn set_data(&mut self, value: u8) {
		self.cache.data = value;
	}

	fn set_int(&mut self, condition: bool) {
		self.set_flag(Status::I, condition);
	}

	fn set_neg_if(&mut self, value: u16) {
		self.set_flag(Status::N, value & 128 != 0)
	}

	fn set_overflow_if(&mut self, condition: bool) {
		self.set_flag(Status::V, condition);
	}

	fn set_rel_addr(&mut self, value: usize) {
		self.cache.rel_addr = value;
	}

	fn set_sp(&mut self, value: usize) {
		self.regs.s = value;
	}

	fn set_x(&mut self, value: u8) {
		self.regs.x = value;
	}

	fn set_y(&mut self, value: u8) {
		self.regs.y = value;
	}

	fn stack_read(&mut self) -> u8 {
		self.regs.s += 1;
		self.get_u8(STACK_ADDR + self.get_sp())
	}

	fn stack_write(&mut self, data: u8) {
		self.write(STACK_ADDR + (self.get_sp() % 256), &[data]);
		self.regs.s -= 1;
	}

	fn stackdump(&self) -> String {
		let dump = self.read(STACK_ADDR, 256);
		hexdump(&dump[..], 2)
	}
}

impl Display for MOS6502 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "{}", &self.regs)?;
		writeln!(f, "{}", &self.cache)
	}
}

impl DeviceBase for MOS6502 {
	fn read(&self, address: usize, length: usize) -> Vec<u8> {
		self.bus.borrow().read(address, length)
	}

	fn write(&mut self, address: usize, data: &[u8]) {
		self.bus.borrow_mut().write(address, data);
	}
}

impl Device for MOS6502 {
	fn get_bus(&self) -> Rc<RefCell<Bus>> {
		Rc::clone(&self.bus)
	}
}

impl ISA6502 for MOS6502 {
	fn irq(&mut self) {
		if !self.check_flag(Status::I) {
			self.interrupt(IRQ_ADDR, 7);
		}
	}

	fn abs(&mut self) -> u8 {
		self.set_mode(Mode::ABS);
		let addr = self.read_rom_addr();
		self.set_abs_addr(addr);

		0
	}

	fn abx(&mut self) -> u8 {
		self.set_mode(Mode::ABX);

		let addr = self.read_rom_addr();
		self.set_abs_addr(addr + self.get_x_zp_addr());

		self.check_page(addr)
	}

	fn aby(&mut self) -> u8 {
		self.set_mode(Mode::ABY);

		let addr = self.read_rom_addr();
		self.set_abs_addr(addr + self.get_y_zp_addr());

		self.check_page(addr)
	}

	fn imm(&mut self) -> u8 {
		self.set_mode(Mode::IMM);
		self.set_abs_addr(self.get_counter());
		self.incr();
		0
	}

	fn imp(&mut self) -> u8 {
		self.set_mode(Mode::IMP);
		self.set_data(self.get_a());
		0
	}

	fn ind(&mut self) -> u8 {
		self.set_mode(Mode::IND);

		let ptr = self.read_rom_addr();

		if (ptr & 255) == 255 {
			// page boundary hardware bug
			self.set_abs_addr(self.get_ptr(ptr));
		} else {
			// normal behavior
			self.set_abs_addr(self.get_ptr(ptr));
		}

		0
	}

	fn izx(&mut self) -> u8 {
		self.set_mode(Mode::IZX);

		let t = self.read_rom_zp_addr();
		let lo = self.get_zp_addr((t + self.get_x_zp_addr()) & 255);
		let hi = self.get_zp_addr((t + self.get_x_zp_addr() + 1) & 255);

		self.set_abs_addr((hi << 8) | lo);
		0
	}

	fn izy(&mut self) -> u8 {
		self.set_mode(Mode::IZY);

		let t = self.read_rom_zp_addr();
		let lo = self.get_zp_addr(t & 255);
		let hi = self.get_zp_addr((t + 1) & 255);

		self.set_abs_addr(((hi << 8) | lo) + self.get_y_zp_addr());

		if self.get_abs_hi() != (hi << 8) { 1 } else { 0 }
	}

	fn rel(&mut self) -> u8 {
		self.set_mode(Mode::REL);
		self.cache.rel_addr = self.read_rom_zp_addr();

		// check_flag for signed bit
		if self.get_rel_addr() & 128 != 0 {
			self.cache.rel_addr |= 0xFF00;
		}

		0
	}

	fn zpg(&mut self) -> u8 {
		self.set_mode(Mode::ZPG);
		let addr = self.read_rom_zp_addr();
		self.set_abs_addr(addr);
		self.incr();
		self.cache.abs_addr &= 255;
		0
	}

	fn zpx(&mut self) -> u8 {
		self.set_mode(Mode::ZPX);
		let addr = self.read_rom_zp_addr();
		self.set_abs_addr(addr + self.get_x_zp_addr());
		self.cache.abs_addr &= 255;
		0
	}

	fn zpy(&mut self) -> u8 {
		self.set_mode(Mode::ZPY);
		let addr = self.read_rom_zp_addr();
		self.set_abs_addr(addr + self.get_y_zp_addr());
		self.cache.abs_addr &= 255;
		0
	}

	fn cld(&mut self) -> u8 {
		self.set_flag(Status::D, false);
		0
	}

	fn nop(&self) -> u8 {
		match self.get_opcode() {
			28 | 60 | 92 | 124 | 220 | 252 => 1,
			_ => 0,
		}
	}

	fn php(&mut self) -> u8 {
		self.set_flag(Status::B, true);
		self.set_flag(Status::U, true);
		self.stack_write(self.get_p_bits());
		self.set_flag(Status::B, false);
		self.set_flag(Status::U, false);

		0
	}

	fn plp(&mut self) -> u8 {
		self.regs.p = Status::from_bits_truncate(self.stack_read());
		self.set_flag(Status::U, true);

		0
	}

	fn rti(&mut self) -> u8 {
		// restore state flags
		self.regs.p = Status::from_bits_truncate(self.stack_read());
		self.regs.p.toggle(Status::B);
		self.regs.p.toggle(Status::U);

		// and counter
		let addr = self.stack_get_ptr();
		self.set_counter(addr);

		0
	}

	fn rts(&mut self) -> u8 {
		let addr = self.stack_get_ptr();
		self.set_counter(addr);
		0
	}

	fn sed(&mut self) -> u8 {
		self.set_flag(Status::D, true);
		0
	}
}

impl Processor for MOS6502 {
	fn clock(&mut self) {
		if self.get_cycles() == 0 {
			// always set unused flag
			self.set_flag(Status::U, true);

			// get and increment the counter
			self.cache.opcode = self.get_u8(self.get_counter()).into();
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
					let mode_cycles = self.zpg();
					let op_cycles = self.nop();
					self.add_cycles(2 + (mode_cycles & op_cycles));
				},
				5 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.ora();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				6 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.asl();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				7 => {
					let mode_cycles = self.zpg();
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
					let mode_cycles = self.zpg();
					let op_cycles = self.bit();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				37 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.and();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				38 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.rol();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				39 => {
					let mode_cycles = self.zpg();
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
					let mode_cycles = self.zpg();
					let op_cycles = self.nop();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				69 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.eor();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				70 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.lsr();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				71 => {
					let mode_cycles = self.zpg();
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
					let mode_cycles = self.zpg();
					let op_cycles = self.nop();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				101 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.adc();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				102 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.ror();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				103 => {
					let mode_cycles = self.zpg();
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
					let mode_cycles = self.zpg();
					let op_cycles = self.sty();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				133 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.sta();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				134 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.stx();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				135 => {
					let mode_cycles = self.zpg();
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
					let mode_cycles = self.zpg();
					let op_cycles = self.ldy();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				165 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.lda();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				166 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.ldx();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				167 => {
					let mode_cycles = self.zpg();
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
					let mode_cycles = self.zpg();
					let op_cycles = self.cpy();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				197 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.cmp();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				198 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.dec();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				199 => {
					let mode_cycles = self.zpg();
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
					let mode_cycles = self.zpg();
					let op_cycles = self.cpx();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				229 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.sbc();
					self.add_cycles(3 + (mode_cycles & op_cycles));
				},
				230 => {
					let mode_cycles = self.zpg();
					let op_cycles = self.inc();
					self.add_cycles(5 + (mode_cycles & op_cycles));
				},
				231 => {
					let mode_cycles = self.zpg();
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

	fn get_ptr(&self, offset: usize) -> usize {
		self.get_u16_le(offset).into()
	}

	fn get_ptr_size(&self) -> usize {
		2
	}

	fn reset(&mut self) {
		self.set_a(0);
		self.set_flag(Status::default(), true);
		self.set_x(0);
		self.set_y(0);
		self.set_sp(STACK_INIT);
		self.set_abs_addr(0);

		let addr = self.get_ptr(RES_ADDR);
		self.set_counter(addr);

		self.cache.rel_addr = 0;
		self.set_data(0);

		self.cache.cycles = 8;
	}
}

/*#[cfg(test)]
mod tests {
	use super::*;

	use crate::{
		DisassemblerConfig,
		MOS6502Disassembler
	};

	use std::io::stdin;

	#[test]
	fn test_exec() {
		let mario = include_bytes!("/home/admin/Downloads/Super Mario Bros (PC10).nes");

		/*let hundred_doors = vec![0xa9,0x00,0xa2,0x64,0x95,0xc8,0xca,0xd0,0xfb,0x95,0xc8,0xa0,
			0x01,0xc0,0x65,0xb0,0x12,0x98,0xc9,0x65,0xb0,0x0a,0xaa,0xfe,0x00,0x02,0x84,0x01,0x65,
			0x01,0x90,0xf2,0xc8,0xd0,0xea,0xa2,0x64,0xbd,0x00,0x02,0x29,0x01,0x9d,0x00,0x02,0xca,
			0xd0,0xf5];*/

		let mut bus = Bus::new(65536);
		bus.write(32768, &mario[16..32784]);
		//bus.write(32768, &hundred_doors[..]);
		//println!("{}", &bus);
		let mut cpu = MOS6502::new(Rc::new(RefCell::new(bus)));
		let cfg = DisassemblerConfig::LOWERCASE | DisassemblerConfig::OFFSETS;
		let mut da = MOS6502Disassembler::new(cpu.get_bus(), Some(cfg));

		let mut input = String::new();
		let mut offs = 0;
		while let Ok(_) = stdin().read_line(&mut input) {
			match input.chars().nth(0).unwrap() {
				'd' | 'D' => println!("{}", &da),
				'q' | 'Q' => break,
				's' | 'S' => println!("{}", cpu.stackdump()),
				_ => {
					loop {
						cpu.clock();

						if cpu.get_cycles() == 0 {
							break;
						}
					}

					println!("{}", &cpu);
				},
			}

			input.clear();
		}

		println!("stopped");
	}
}*/
