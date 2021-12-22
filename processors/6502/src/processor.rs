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
	Device,
	opcodes::*
};

macro_rules! opcode_id {
	($b3: literal) => {
		[$b3[0], $b3[1], $b3[2]]
	}
}

pub const OPCODE_TABLE_SIZE: usize = 256;
pub const STACK_ADDR: u16 = 256;
pub const IRQ_ADDR: u16 = 0xFFFE;
pub const NMI_ADDR: u16 = 0xFFFA;
pub const RESET_ADDR: u16 = 0xFFFC;
pub static NOP: Opcode = Opcode::new(opcode_id!(b"NOP"), Processor::imp, Processor::nop, 2);

bitflags! {
	pub struct Status: u8 {
		const CARRY = 1;
		const ZERO = 2;
		const NO_INTERRUPTS = 4;
		const DECIMAL = 8;
		const BREAK = 16;
		const UNUSED = 32;
		const OVERFLOW = 64;
		const NEGATIVE = 128;
	}
}

impl Default for Status {
	fn default() -> Self {
		Status::UNUSED
	}
}

impl Display for Status {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if self.contains(Status::CARRY) {
			write!(f, "C")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::ZERO) {
			write!(f, "Z")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::NO_INTERRUPTS) {
			write!(f, "I")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::DECIMAL) {
			write!(f, "D")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::BREAK) {
			write!(f, "B")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::UNUSED) {
			write!(f, "U")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::OVERFLOW) {
			write!(f, "V")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::NEGATIVE) {
			write!(f, "N")
		} else {
			write!(f, "x")
		}
	}
}

pub type Operation = fn(this: &mut Processor) -> u8;
pub type AddressMode = fn(this: &mut Processor) -> u8;

#[derive(Clone, Debug)]
pub struct Opcode {
	pub id: [u8; 3],
	pub cycles: u8,
	pub op: Operation,
	pub addr_mode: AddressMode,
}

impl Opcode {
	pub fn new(id: [u8; 3], addr_mode: AddressMode, op: Operation, cycles: u8) {
		Opcode {
			id: id,
			cycles: cycles,
			op: op,
			addr_mode: addr_mode,
		}
	}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Registers {
	pub accumulator: u8,
	pub status: Status,
	pub x: u8,
	pub y: u8,
	pub counter: u16,
	pub stack_ptr: u16, // actually a 1 byte register, but this avoids casting every use
}

impl Display for Registers {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "STATUS: {}", self.status)?;
		writeln!(f, "PC: ${:X}", self.counter)?;
		writeln!(f, "A: ${:X}", self.accumulator)?;
		writeln!(f, "X: ${:X}", self.x)?;
		writeln!(f, "Y: ${:X}", self.y)?;
		writeln!(f, "STACK POINTER: ${:X}", self.stack_ptr)?;
	}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Cache {
	pub data: u8,
	pub opcode: u8,
	pub cycles: u8,
	pub abs_addr: u16,
	pub rel_addr: u16,
}

#[derive(Clone, Debug)]
pub struct Processor<'a> {
	pub bus: &'a Bus,
	pub registers: Registers,
	pub cache: Cache,
	pub opcodes: [Opcode; OPCODE_TABLE_SIZE],
}

impl<'a> Processor<'a> {
	pub fn new(bus: &mut 'a Bus) -> Processor {
		Processor {
			bus: bus,
			registers: Registers::default(),
			cache: Cache::default(),
			opcodes: [
				Opcode::new(opcode_id!(b"BRK"), Processor::imm, Processor::brk, 7),
				Opcode::new(opcode_id!(b"ORA"), Processor::izx, Processor::ora, 6),
				Opcode::new(opcode_id!(b"BRK"), Processor::imm, Processor::brk, 7),
				NOP,
				NOP,
				NOP,
				Opcode::new(opcode_id!(b"ORA"), Processor::zp0, Processor::ora, 3),
			],
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
		self.set_abs(self.counter + self.rel_addr());

		// need an additional cycle if different page
		if self.abs_hi() != (self.counter() & 0xFF00) {
			self.cache.cycles += 1;
		}

		// jump to the address
		self.set_pc(self.abs_addr());
	}

	/// Checks specified status flag
	#[inline]
	pub fn check(&self, flag: Status) -> bool {
		self.registers.status.contains(flag)
	}

	pub fn clock(&mut self) {
		if self.cache.cycles == 0 {
			// get and increment the counter
			self.cache.opcode = self.read(self.registers.counter);
			self.incr();

			// set our cycles, and see if we need any additional cycles
			self.cache.cycles = self.oc_cycles(self.oc_index());
			self.cache.cycles += self.oc_addr_mode(self.oc_index())(&mut self);
			self.cache.cycles += self.oc_op(self.oc_index())(&mut self);
		}

		self.cache.cycles -= 1;
	}

	/// Retrieve the program counter value
	#[inline]
	pub fn counter(&self) -> u8 {
		self.registers.counter
	}

	/// Fetch data from an operation
	pub fn fetch(&mut self) -> u8 {
		if !self.opcodes[self.cache.opcode as usize].addr_mode == Processor::imp {
			self.set_data(self.read(self.abs_addr()));
		}

		self.get_data()
	}

	#[inline]
	/// Fetch address
	pub fn fetch_addr(&mut self) -> u16 {
		((self.read(self.abs_addr()) as u16) << 8) | (self.read(self.abs_addr() + 1) as u16);
	}

	/// Gets the accumulator register value
	#[inline]
	pub fn get_a(&mut self) -> u8 {
		self.registers.accumulator
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

	/// Increment program counter registry by 1
	#[inline]
	pub fn incr(&mut self) {
		self.registers.counter += 1;
	}

	/// Interrupts the execution state
	#[inline]
	pub fn interrupt(new_abs_addr: u16, new_cycles: u8) {
		// write the counter's current value to stack
		self.stack_write_addr(self.counter());

		// write status register to stack too
		self.set_flag(Status::BREAK, false);
		self.set_flag(Status::UNUSED, true);
		self.set_flag(Status::NO_INTERRUPTS, true);
		self.stack_write(self.status_bits());

		// get the new counter value
		self.set_abs(new_abs_addr);
		self.set_pc(self.fetch_addr());

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
		self.registers.stack_ptr = 254;

		self.set_abs(RESET_ADDR);
		self.set_pc(self.fetch_addr());

		self.cache.rel_addr = 0;
		self.set_abs(0);
		self.set_data(0);

		self.cache.cycles = 8;
	}

	/// Sets accumulator register value
	#[inline]
	pub fn set_a(&mut self, value: u8) {
		self.registers.accumulator = value;
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

	/// Sets program counter register value
	#[inline]
	pub fn set_pc(&mut self, value: u16) {
		self.registers.counter = value;
	}

	/// Sets status register flag
	#[inline]
	pub fn set_flag(&mut self, flags: Status, condition: bool) {
		self.registers.status.set(flags, condition);
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
	pub fn stack_ptr(&mut self) -> u16 {
		self.registers.stack_ptr
	}

	/// Convenience function to read from stack
	#[inline]
	pub fn stack_read(&mut self) {
		self.registers.stack_ptr += 1;
		self.read(STACK_ADDR + self.stack_ptr());
	}

	/// Reads an address from stack
	#[inline]
	pub fn stack_read_addr(&mut self) -> u16 {
		(self.stack_read() as u16) | (self.stack_read() as u16) << 8
	}

	/// Convenience function to write to stack
	#[inline]
	pub fn stack_write(&mut self, data: u8) {
		self.write(STACK_ADDR + self.stack_ptr(), data);
		self.registers.stack_ptr -= 1;
	}

	/// Writes an address to stack
	#[inline]
	pub fn stack_write_addr(&mut self, addr: u16) {
		self.stack_write(((addr >> 8) & 255) as u8);
		self.stack_write((addr & 255) as u8);
	}

	/// Retrieve the registry status flags
	#[inline]
	pub fn status(&self) -> Status {
		self.registers.status
	}

	/// Retrieve the registry status flag bits
	#[inline]
	pub fn status_bits(&self) -> u8 {
		self.registers.status.bits()
	}

	include!("operations.rs")
}

impl<'a> Device for Processor<'a> {
	fn read(&self, address: u16) -> u8 {
		self.bus.read(address)
	}

	fn write(&mut self, address: u16; data: u8) {
		self.bus.write(address, data);
	}
}
