use bitflags::bitflags;

use std::{
	default::Default,
	fmt::{
		Display,
		Formatter,
		self
	}
};

use crate::{
	Bus,
	Device
};

/// Offset of program stack
pub const STACK_ADDR: u16 = 256;

/// Offset of interrupt request vector
pub const IRQ_ADDR: u16 = 65534;

/// Offset of non-maskable interrupt vector
pub const NMI_ADDR: u16 = 65530;

/// Offset of reset vector
pub const RESET_ADDR: u16 = 65532;

bitflags! {
	/// CPU state flags
	pub struct Status: u8 {
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

		if self.contains(Status::DECIMAL) {
			write!(f, "D")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::B) {
			write!(f, "B")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::U) {
			write!(f, "U")?;
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

/// CPU registers
#[derive(Clone, Copy, Debug, Default)]
pub struct Registers {
	/// accumulator
	a: u8,

	/// state flags
	p: Status,

	/// general purpose
	x: u8,

	/// general purpose
	y: u8,

	/// program counter
	pc: u16,

	/// The stack pointer is actually a 1 byte register, but this avoids casting every use
	s: u16,
}

impl Display for Registers {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "P: {}", self.p)?;
		writeln!(f, "PC: ${:X}", self.pc)?;
		writeln!(f, "A: ${:X}", self.a)?;
		writeln!(f, "X: ${:X}", self.x)?;
		writeln!(f, "Y: ${:X}", self.y)?;
		writeln!(f, "SP: ${:X}", self.s)?;
	}
}

/// CPU cache
#[derive(Clone, Copy, Debug, Default)]
pub struct Cache {
	/// last fetched byte
	data: u8,

	/// last fetched opcode
	opcode: u8,

	/// remaining cycles on current operation
	cycles: u8,

	/// last absolute address
	abs_addr: u16,

	/// last relative address is actually 1 byte, but this avoids casting every use
	rel_addr: u16,
}

/// The CPU itself
#[derive(Clone, Debug)]
pub struct Processor<'a> {
	/// Reference to the parent bus
	bus: &'a Bus<'a>,

	registers: Registers,
	cache: Cache,
}

impl<'a> Processor<'a> {
	/// Initialises a new processor, given a mutable bus reference
	pub fn new(bus: &'a mut Bus) -> Processor<'a> {
		Processor {
			bus,
			registers: Registers::default(),
			cache: Cache::default(),
		}
	}

	/// Gets cached absolute address
	#[inline]
	pub fn abs_addr(&mut self) -> u16 {
		self.registers.abs_addr
	}

	/// Gets cached absolute address' high byte
	#[inline]
	pub fn abs_hi(&mut self) -> u16 {
		self.registers.abs_addr & 0xFF00
	}

	#[inline]
	pub fn branch(&mut self) {
		self.cache.cycles += 1;
		self.set_abs(self.pc + self.rel_addr());

		// need an additional cycle if different page
		if self.abs_hi() != (self.counter() & 0xFF00) {
			self.cache.cycles += 1;
		}

		// jump to the address
		self.set_counter(self.abs_addr());
	}

	/// Checks specified p flag
	#[inline]
	pub fn check_flag(&self, flag: Status) -> bool {
		self.registers.p.contains(flag)
	}

	pub fn clock(&mut self) {
		if self.cache.cycles == 0 {
			// get and increment the pc
			self.cache.opcode = self.read(self.registers.pc);
			self.incr();

			// set our cycles, and see if we need any additional cycles
			self.cache.cycles = self.oc_cycles(self.oc_index());
			self.cache.cycles += self.oc_addr_mode(self.oc_index())(&mut self);
			self.cache.cycles += self.oc_op(self.oc_index())(&mut self);
		}

		self.cache.cycles -= 1;
	}

	/// Retrieve the program pc value
	#[inline]
	pub fn counter(&self) -> u8 {
		self.registers.pc
	}

	/// Fetch data from an operation
	pub fn fetch(&mut self) -> u8 {
		if !self.opcodes[self.cache.opcode as usize].addr_mode == Processor::imp {
			self.set_data(self.read(self.abs_addr()));
		}

		self.get_data()
	}

	/// Fetch address
	#[inline]
	pub fn fetch_addr(&mut self) -> u16 {
		((self.read(self.abs_addr()) as u16) << 8) | (self.read(self.abs_addr() + 1) as u16)
	}

	/// Gets the a register value
	#[inline]
	pub fn get_a(&mut self) -> u8 {
		self.registers.a
	}

	/// Gets the currently cached data byte
	#[inline]
	pub fn get_data(&mut self) -> u8 {
		self.cache.data
	}

	/// Gets the X register value
	#[inline]
	pub fn get_x(&mut self) -> u8 {
		self.registers.x
	}

	/// Gets the Y register value
	#[inline]
	pub fn get_y(&mut self) -> u8 {
		self.registers.y
	}

	/// Increment program pc registry by 1
	#[inline]
	pub fn incr(&mut self) {
		self.registers.pc += 1;
	}

	/// Interrupts the execution state
	#[inline]
	pub fn interrupt(&mut self, new_abs_addr: u16, new_cycles: u8) {
		// write the pc's current value to stack
		self.stack_write_addr(self.counter());

		// write p register to stack too
		self.set_flag(Status::B, false);
		self.set_flag(Status::U, true);
		self.set_flag(Status::I, true);
		self.stack_write(self.p_bits());

		// get the new pc value
		self.set_abs(new_abs_addr);
		self.set_counter(self.fetch_addr());

		self.cache.cycles = new_cycles;
	}

	/// Returns the specified opcode's address mode
	#[inline]
	pub fn oc_addr_mode(&mut self, i: usize) -> AddressMode {
		self.opcodes[i].addr_mode
	}

	/// Returns the specified opcode's cycle count
	#[inline]
	pub fn oc_cycles(&mut self, i: usize) -> u8 {
		self.opcodes[i].cycles
	}

	/// Returns the cached opcode index
	#[inline]
	pub fn oc_index(&mut self) -> usize {
		self.cache.opcode as usize
	}

	/// Returns the specified opcode's operation
	#[inline]
	pub fn oc_op(&mut self, i: usize) -> Operation {
		self.opcodes[i].op
	}

	/// Reads an address from the RAM
	#[inline]
	pub fn read_addr(&mut self, addr: u16) -> u16 {
		self.read(addr) | (self.read(addr) << 8)
	}

	/// Reads an address from the ROM
	#[inline]
	pub fn read_rom_addr(&mut self) -> u16 {
		let lo = self.read(self.counter()) as u16;
		self.incr();

		let hi = self.read(self.counter()) as u16;
		self.incr();

		(hi << 8) | lo
	}

	/// Gets cached relative address
	#[inline]
	pub fn rel_addr(&mut self) -> u16 {
		self.cache.rel_addr
	}

	/// Resets the registers and cache
	pub fn reset(&mut self) {
		self.set_a(0);
		self.set_flag(Status::default());
		self.set_x(0);
		self.set_y(0);
		self.registers.s = 254;

		self.set_abs(RESET_ADDR);
		self.set_counter(self.fetch_addr());

		self.cache.rel_addr = 0;
		self.set_abs(0);
		self.set_data(0);

		self.cache.cycles = 8;
	}

	/// Sets a register value
	#[inline]
	pub fn set_a(&mut self, value: u8) {
		self.registers.a = value;
	}

	/// Sets cached absolute address
	#[inline]
	pub fn set_abs(&mut self, value: u16) {
		self.cache.abs_addr = value;
	}

	/// Sets cached data
	#[inline]
	pub fn set_data(&mut self, value: u8) {
		self.cache.data = value;
	}

	/// Sets program pc register value
	#[inline]
	pub fn set_counter(&mut self, value: u16) {
		self.registers.pc = value;
	}

	/// Sets p register flag
	#[inline]
	pub fn set_flag(&mut self, flags: Status, condition: bool) {
		self.registers.p.set(flags, condition);
	}

	/// Sets X register value
	#[inline]
	pub fn set_x(&mut self, value: u8) {
		self.registers.x = value;
	}

	/// Sets Y register value
	#[inline]
	pub fn set_y(&mut self, value: u8) {
		self.registers.y = value;
	}

	/// Gets the stack pointer value
	#[inline]
	pub fn s(&mut self) -> u16 {
		self.registers.s
	}

	/// Convenience function to read from stack
	#[inline]
	pub fn stack_read(&mut self) {
		self.registers.s += 1;
		self.read(STACK_ADDR + self.s());
	}

	/// Reads an address from stack
	#[inline]
	pub fn stack_read_addr(&mut self) -> u16 {
		(self.stack_read() as u16) | (self.stack_read() as u16) << 8
	}

	/// Convenience function to write to stack
	#[inline]
	pub fn stack_write(&mut self, data: u8) {
		self.write(STACK_ADDR + self.s(), data);
		self.registers.s -= 1;
	}

	/// Writes an address to stack
	#[inline]
	pub fn stack_write_addr(&mut self, addr: u16) {
		self.stack_write(((addr >> 8) & 255) as u8);
		self.stack_write((addr & 255) as u8);
	}

	/// Retrieve the registry p flags
	#[inline]
	pub fn p(&self) -> Status {
		self.registers.p
	}

	/// Retrieve the registry p flag bits
	#[inline]
	pub fn p_bits(&self) -> u8 {
		self.registers.p.bits()
	}

	/// Absolute address mode
	pub fn abs(&mut self) -> u8 {
		self.set_abs(self.read_rom_addr());
		0
	}

	/// Absolute address mode with X register offset
	pub fn abx(&mut self) -> u8 {
		let addr = self.read_rom_addr() + self.get_x();
		self.set_abs(addr);

		if self.abs_hi() != (addr & 0xFF00) {
			1
		} else {
			0
		}
	}

	/// Absolute address mode with Y register offset
	pub fn aby(&mut self) -> u8 {
		let addr = self.read_rom_addr() + self.get_y();
		self.set_abs(addr);

		if self.abs_hi() != (addr & 0xFF00) {
			1
		} else {
			0
		}
	}

	/// Addition with carry
	pub fn adc(&mut self) -> u8 {
		let _ = self.fetch_byte();
		let carry = if self.check_flag(Status::C) { 1 } else { 0 };
		let temp16 = (self.get_a() as u16) + (self.get_data() as u16) + carry;

		self.set_flag(Status::C, temp16 > 255);
		self.set_flag(Status::Z, temp16 & 255 == 0);
		self.set_flag(Status::N, temp16 & 128 != 0);

		self.set_flag(Status::V, !(((self.get_a() as u16) ^ (self.get_data() as u16)) &
			((self.get_a() as u16) ^ temp16)) & 128);

		self.set_a((temp16 & 255) as u8);

		1
	}

	/// Bitwise and
	pub fn and(&mut self) -> u8 {
		let _ = self.fetch_byte();
		self.registers.a &= self.get_data();
		self.set_flag(Status::Z, self.get_a() == 0);
		self.set_flag(Status::N, self.get_a() & 128 != 0);

		1
	}

	/// Arithmetical left shift
	pub fn asl(&mut self) -> u8 {
		let _ = self.fetch_byte();
		let temp16 = (self.get_data() as u16) << 1;

		self.set_flag(Status::C, temp16 & 0xFF > 0);
		self.set_flag(Status::Z, temp16 & 255 == 0);
		self.set_flag(Status::N, temp16 & 128 != 0);

		if self.oc_addr_mode(self.oc_index()) == Processor::imp {
			self.set_a((temp16 & 255) as u8);
		} else {
			self.write(self.abs_addr(), (temp16 & 255) as u8);
		}

		0
	}

	/// Branching if carry clear
	pub fn bcc(&mut self) -> u8 {
		if !self.check_flag(Status::C) {
			self.branch();
		}

		0
	}

	/// Branching if carry
	pub fn bcs(&mut self) -> u8 {
		if self.check_flag(Status::C) {
			self.branch();
		}

		0
	}

	/// Branching if equal (zero)
	pub fn beq(&mut self) -> u8 {
		if self.check_flag(Status::Z) {
			self.branch();
		}

		0
	}

	/// Branching if negative
	pub fn bmi(&mut self) -> u8 {
		if self.check_flag(Status::N) {
			self.branch();
		}

		0
	}

	/// Branching if not equal (non-zero)
	pub fn bne(&mut self) -> u8 {
		if !self.check_flag(Status::Z) {
			self.branch();
		}

		0
	}

	/// Branching if positive
	pub fn bpl(&mut self) -> u8 {
		if !self.check_flag(Status::N) {
			self.branch();
		}

		0
	}

	/// Program-sourced interrupt.
	pub fn brk(&mut self) -> u8 {
		// This differs slightly from self.interrupt()

		self.incr();

		self.set_flag(Status::I, true);
		self.stack_write_addr(self.counter());

		self.set_flag(Status::B, true);
		self.stack_write(self.p_bits());
		self.set_flag(Status::B, false);

		self.set_counter(self.read_addr(IRQ_ADDR));
		0
	}

	/// Branching if overflow
	pub fn bvc(&mut self) -> u8 {
		if self.check_flag(Status::V) {
			self.branch();
		}

		0
	}

	/// Branching if not overflow
	pub fn bvs(&mut self) -> u8 {
		if !self.check_flag(Status::V) {
			self.branch();
		}

		0
	}

	/// Clear carry bit
	pub fn clc(&mut self) -> u8 {
		self.set_flag(Status::C, false);
		0
	}

	/// Clear decimal bit
	pub fn cld(&mut self) -> u8 {
		self.set_flag(Status::DECIMAL, false);
		0
	}

	/// Clear interrupt disable bit
	pub fn cli(&mut self) -> u8 {
		self.set_flag(Status::I, false);
		0
	}

	/// Clear overflow bit
	pub fn clv(&mut self) -> u8 {
		self.set_flag(Status::V, false);
		0
	}

	/// Immediate address mode
	pub fn imm(&mut self) -> u8 {
		self.incr();
		self.set_abs(self.counter());
		0
	}

	/// Implied address mode
	pub fn imp(&mut self) -> u8 {
		self.set_data(self.get_a());
		0
	}

	/// Indirect address mode (pointer access)
	pub fn ind(&mut self) -> u8 {
		let ptr = self.read_rom_addr();

		if (ptr & 255) == 255 {
			// page boundary hardware bug
			self.set_abs(((self.read(ptr & 0xFF00) as u16) << 8) | self.read(ptr));
		} else {
			// normal behavior
			self.set_abs(((self.read(ptr + 1) as u16) << 8) | self.read(ptr));
		}

		0
	}

	/// Interrupt request
	pub fn irq(&mut self) {
		if !self.check_flag(Status::I) {
			self.interrupt(IRQ_ADDR, 7);
		}
	}

	/// Indirect address mode of zero-page with X register offset
	pub fn izx(&mut self) -> u8 {
		let t = self.read(self.counter());
		self.incr();

		let lo = self.read((t + (self.get_x() as u16)) & 255);
		let hi = self.read((t + (self.get_x() as u16) + 1) & 255);

		self.set_abs((hi << 8) | lo);
		0
	}

	/// Indirect address mode of zero-page with Y register offset
	pub fn izy(&mut self) -> u8 {
		let t = self.read(self.counter());
		self.incr();

		let lo = self.read(t & 255) as u16;
		let hi = self.read((t + 1) & 255) as u16;

		self.set_abs(((hi << 8) | lo) + (self.get_y() as u16));

		if self.abs_hi() != (hi << 8) {
			1
		} else {
			0
		}
	}

	/// Non-maskable interrupt
	pub fn nmi(&mut self) {
		self.interrupt(NMI_ADDR, 8);
	}

	/// No operation, illegal opcode filler
	pub fn nop(&self) {
	}

	/// Push a to the stack
	pub fn pha(&mut self) -> u8 {
		self.stack_write(self.get_a());
		0
	}

	/// Pop a from the stack
	pub fn pla(&mut self) -> u8 {
		self.set_a(self.stack_read());
		self.set_flag(Status::Z, self.get_a() == 0);
		self.set_flag(Status::N, self.get_a() & 128 != 0);

		0
	}

	/// Relative address mode (branching instructions)
	pub fn rel(&mut self) -> u8 {
		self.cache.rel_addr = self.read(self.counter());
		self.incr();

		// check_flag for signed bit
		if self.rel_addr() & 128 != 0 {
			self.cache.rel_addr |= 0xFF00;
		}

		0
	}

	/// Restores state from interrupt
	pub fn rti(&mut self) -> u8 {
		// restore p flags
		self.registers.p = Status::from_bits_truncate(self.stack_read());
		self.registers.p &= !Status::B;
		self.registers.p &= !Status::U;

		// and pc
		self.set_counter(self.stack_read_rom_addr());

		0
	}

	/// Subtraction with carry
	pub fn sdc(&mut self) -> u8 {
		let _ = self.fetch_byte();
		let value = (self.get_data() as u16) ^ 255; // invert the value
		let carry = if self.check_flag(Status::C) { 1 } else { 0 };
		let temp16 = (self.get_a() as u16) + value + carry;

		self.set_flag(Status::C, temp16 & 0xFF00 != 0);
		self.set_flag(Status::Z, temp16 & 255 == 0);
		self.set_flag(Status::V, (temp16 ^ (self.get_a() as u16)) & (temp16 ^ value) & 128);
		self.set_flag(Status::N, temp16 & 128 != 0);

		self.set_a((temp16 & 255) as u8);

		1
	}

	/// Zero-page address mode
	pub fn zp0(&mut self) -> u8 {
		self.set_abs(self.read(self.counter()));
		self.incr();
		self.cache.abs_addr &= 255;
		0
	}

	/// Zero-page address mode with X register offset
	pub fn zpx(&mut self) -> u8 {
		self.set_abs(self.read(self.counter()) + (self.get_x() as u16));
		self.incr();
		self.cache.abs_addr &= 255;
		0
	}

	/// Zero-page address mode with Y register offset
	pub fn zpy(&mut self) -> u8 {
		self.set_abs(self.read(self.counter()) + (self.get_y() as u16));
		self.incr();
		self.cache.abs_addr &= 255;
		0
	}

}

impl<'a> Device for Processor<'a> {
	fn read(&self, address: u16) -> u8 {
		self.bus.read(address)
	}

	fn write(&mut self, address: u16, data: u8) {
		self.bus.write(address, data);
	}
}
