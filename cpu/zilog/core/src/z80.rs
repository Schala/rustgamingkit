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

bitflags! {
	/// Z80 state flags
	pub struct Status: u8 {
		const S = 1;
		const Z = 2;
		const H = 8;
		const PV = 32;
		const N = 64;
		const C = 128;
	}
}

impl Display for Status {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if self.contains(Status::S) {
			write!(f, "S")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::Z) {
			write!(f, "Z")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::H) {
			write!(f, "H")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::PV) {
			write!(f, "P")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::N) {
			write!(f, "N")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::C) {
			write!(f, "C")?;
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

/// Register specified with an opcode
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum RegisterType {
	A, AF, B, BC, C, D, DE, E, H, HL, L, SP,
}

/// Address modes
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Mode {
	A8ToReg,
	A16ToReg,
	D8,
	D8ToReg,
	D8ToRegAddr,
	D16,
	D16ToReg,
	Implied,
	Reg,
	RegAddr,
	RegAddrToReg,
	RegToA8,
	RegToA16,
	RegToReg,
	RegToRegAddr,
}

/// Z80 registers
#[derive(Clone, Copy, Debug)]
pub struct Registers {
	a: u8,
	f: Status,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	h: u8,
	l: u8,
	i: u8,
	r: u8,
	x: u16,
	y: u16,
	sp: usize,
	pc: usize,
}

impl Display for Registers {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "A: ${:02X},\tF: {},\tAF: ${:04X}", self.a, self.f, u16::from_le_bits([self.f.bits(), self.a]))?;
		writeln!(f, "B: ${:02X},\tC: ${:02X},\tBC: ${:04X}", self.b, self.c, u16::from_le_bits([self.c, self.b]))?;
		writeln!(f, "D: ${:02X},\tE: ${:02X},\tDE: ${:04X}", self.d, self.e, u16::from_le_bits([self.e, self.d]))?;
		writeln!(f, "H: ${:02X},\tL: ${:02X},\tHL: ${:04X}", self.h, self.l, u16::from_le_bits([self.l, self.h]))?;
		writeln!(f, "I: ${:02X},\tR: ${:02X}", self.i, self.r)?;
		writeln!(f, "X: ${:04X},\tY: ${:04X}", self.x, self.y)?;
		writeln!(f, "SP: ${:02X},\tPC: ${:04X}", self.s & 255, self.pc & 65535)
	}
}

/// Z80 cache
#[derive(Clone, Copy, Debug)]
pub struct Cache {
	data: u8,
	opcode: u8,
	addr: usize,
}

impl Display for Cache {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Last fetched byte: ${:X}", self.data)?;
		writeln!(f, "Last fetched opcode: ${:X}", self.opcode)?;
		writeln!(f, "Cycles remaining: {}", self.cycles)?;
		writeln!(f, "Current address: ${:X}", self.addr)?;
	}
}

pub struct Z80 {
	bus: Rc<RefCell<Bus>>,
	regs: Registers,
	cache: Cache,
}

impl Z80 {
	/// Initialises a new Z80, given a bus pointer
	pub fn new(bus: Rc<Bus>) -> Z80 {
		Z80 {
			bus,
			regs: Registers {
				a: 0,
				f: Status::default(),
				b: 0,
				c: 0,
				d: 0,
				e: 0,
				h: 0,
				i: 0,
				l: 0,
				r: 0,
				x: 0,
				y: 0,
				s: 0,
				pc: 0,
			},
		}
	}

	/// Gets the 8-bit accumulator register
	const fn get_a(&self) -> u8 {
		self.regs.a
	}

	/// Gets the AF register bits
	const fn get_af(&self) -> u16 {
		u16::from_le_bytes([self.get_f_bits(), self.get_a()])
	}

	/// Gets the B register
	const fn get_b(&self) -> u8 {
		self.regs.b
	}

	/// Gets the BC register
	const fn get_bc(&self) -> u16 {
		u16::from_le_bytes([self.get_c(), self.get_b()])
	}

	/// Gets the C register
	const fn get_c(&self) -> u8 {
		self.regs.c
	}

	/// Gets the program counter
	const fn get_counter(&self) -> usize {
		self.regs.pc
	}

	/// Gets the D register
	const fn get_d(&self) -> u8 {
		self.regs.d
	}

	/// Gets the DE register
	const fn get_de(&self) -> u16 {
		u16::from_le_bytes([self.get_e(), self.get_d()])
	}

	/// Gets the E register
	const fn get_e(&self) -> u8 {
		self.regs.e
	}

	/// Gets the flags register bits
	const fn get_f_bits(&self) -> u8 {
		self.regs.f.bits()
	}

	/// Gets the H register
	const fn get_h(&self) -> u8 {
		self.regs.h
	}

	/// Gets the interrupt vector
	const fn get_interrupt(&self) -> u8 {
		self.regs.i
	}

	/// Gets the 16-bit accumulator register
	const fn get_hl(&self) -> u16 {
		u16::from_le_bytes([self.get_l(), self.get_h()])
	}

	/// Gets the L register
	const fn get_l(&self) -> u8 {
		self.regs.l
	}

	/// Gets the refresh counter
	const fn get_refresh_counter(&self) -> u8 {
		self.regs.r
	}

	/// Gets the stack pointer
	const fn get_sp(&self) -> usize {
		self.regs.s
	}

	/// Gets the X index register
	const fn get_x(&self) -> u16 {
		self.regs.x
	}

	/// Gets the Y index register
	const fn get_y(&self) -> u16 {
		self.regs.y
	}

	/// Increments the program counter
	fn incr(&mut self) {
		self.regs.pc += 1;
	}

	/// Sets the 8-bit accumulator register
	fn set_a(&mut self, value: u8)  {
		self.regs.a = value;
	}

	/// Sets the AF register bits
	fn set_af(&mut self, value: u16) {
		let bytes = value.to_le_bytes();
		self.set_a(bytes[1]);
		self.regs.f = Status::from_bits_truncate(bytes[0] as u32);
	}

	/// Sets the B register
	fn set_b(&mut self, value: u8)  {
		self.regs.b = value;
	}

	/// Sets the BC register
	fn set_bc(&mut self, value: u16) {
		let bytes = value.to_le_bytes();
		self.set_b(bytes[1]);
		self.set_c(bytes[0]);
	}

	/// Sets the C register
	fn set_c(&mut self, value: u8)  {
		self.regs.c = value;
	}

	/// Sets the program counter
	fn set_counter(&mut self, value: usize) {
		self.regs.pc = value;
	}

	/// Sets the D register
	fn set_d(&mut self, value: u8)  {
		self.regs.d = value;
	}

	/// Sets the DE register
	fn set_de(&mut self, value: u16) {
		let bytes = value.to_le_bytes();
		self.set_d(bytes[1]);
		self.set_e(bytes[0]);
	}

	/// Sets the E register
	fn set_e(&mut self, value: u8)  {
		self.regs.e = value;
	}

	/// Sets the H register
	fn set_h(&mut self, value: u8)  {
		self.regs.h = value;
	}

	/// Sets the 16-bit accumulator register
	fn set_hl(&mut self, value: u16) {
		let bytes = value.to_le_bytes();
		self.set_h(bytes[1]);
		self.set_l(bytes[0]);
	}

	/// Sets the interrupt vector
	fn set_interrupt(&mut self, value: u8)  {
		self.regs.i = value;
	}

	/// Sets the L register
	fn set_l(&mut self, value: u8)  {
		self.regs.l = value;
	}

	/// Sets the refresh counter
	fn set_refresh_counter(&mut self, value: u8)  {
		self.regs.r = value;
	}

	/// Sets the stack pointer
	fn set_sp(&mut self, value: usize) {
		self.regs.s = value;
	}

	/// Sets the X index register
	fn set_x(&mut self, value: u16) {
		self.regs.x = value;
	}

	/// Sets the Y index register
	fn set_y(&mut self, value: u16) {
		self.regs.y = value;
	}
}

impl DeviceBase for Z80 {
	fn read(&self, address: usize, length: usize) -> Vec<u8> {
		self.bus.borrow().read(address, length)
	}

	fn write(&mut self, address: usize, data: &[u8]) {
		self.bus.borrow().write(address, data);
	}
}

impl Device for Z80 {
	fn get_bus(&self) -> Rc<Bus> {
		Rc::clone(&self.bus)
	}
}

impl Processor for Z80 {
	fn clock(&mut self) {
	}

	fn get_ptr_size(&self) -> usize {
		2
	}

	fn reset(&mut self) {
	}
}
