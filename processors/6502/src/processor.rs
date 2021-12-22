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
			],
		}
	}

	#[inline]
	pub fn branch(&mut self) {
		self.cache.cycles += 1;
		self.cache.abs_addr = self.registers.counter + self.cache.rel_addr;

		// need an additional cycle if different page
		if (self.cache.abs_addr & 0xFF00) != (self.registers.counter & 0xFF00) {
			self.cache.cycles += 1;
		}

		// jump to the address
		self.registers.counter = self.cache.abs_addr;
	}

	pub fn clock(&mut self) {
		if self.cache.cycles == 0 {
			// get and increment the counter
			self.cache.opcode = self.read(self.registers.counter);
			self.registers.counter += 1;

			// set our cycles, and see if we need any additional cycles
			self.cache.cycles = opcodes[self.cache.opcode as usize].cycles;
			self.cache.cycles += self.opcodes[self.cache.opcode as usize].addr_mode();
			self.cache.cycles += self.opcodes[self.cache.opcode as usize].op();
		}

		self.cache.cycles -= 1;
	}

	/// Fetch data from an operation
	pub fn fetch(&mut self) -> u8 {
		if !self.opcodes[self.cache.opcode as usize].addr_mode == Processor::imp {
			self.cache.data = self.read(self.cache.abs_addr);
		}

		self.cache.data
	}

	/// Interrupts the execution state
	#[inline]
	pub fn interrupt(new_abs_addr: u16, new_cycles: u8) {
		// write the counter's current value to stack
		self.stack_write(((self.registers.counter >> 8) & 255) as u8);
		self.stack_write((self.registers.counter & 255) as u8);

		// write status register to stack too
		self.registers.status.set(Status::BREAK, false);
		self.registers.status.set(Status::UNUSED, true);
		self.registers.status.set(Status::NO_INTERRUPTS, true);
		self.stack_write(self.registers.status.bits());

		// get the new counter value
		self.cache.abs_addr = new_abs_addr;
		let lo = self.read(self.cache.abs_addr) as u16;
		let hi = self.read(self.cache.abs_addr + 1) as u16;
		self.registers.counter = (hi << 8) | lo;

		self.cache.cycles = new_cycles;
	}

	/// Resets the registers and cache
	pub fn reset(&mut self) {
		self.registers.accumulator = 0;
		self.registers.status = Status::default();
		self.registers.x = 0;
		self.registers.y = 0;
		self.registers.stack_ptr = 254;

		self.cache.abs_addr = RESET_ADDR;
		let lo = self.read(self.cache.abs_addr) as u16;
		let hi = self.read(self.cache.abs_addr + 1) as u16;
		self.registers.counter = (hi << 8) | lo;

		self.cache.rel_addr = 0;
		self.cache.abs_addr = 0;
		self.cache.data = 0;

		self.cache.cycles = 8;
	}

	/// Convenience function to read from stack
	#[inline]
	pub fn stack_read(&mut self) {
		self.registers.stack_ptr += 1;
		self.read(STACK_ADDR + self.registers.stack_ptr);
	}

	/// Convenience function to write to stack
	#[inline]
	pub fn stack_write(&mut self, data: u8) {
		self.write(STACK_ADDR + self.registers.stack_ptr, data);
		self.registers.stack_ptr -= 1;
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
