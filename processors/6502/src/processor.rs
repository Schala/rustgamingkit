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

pub const STACK_ADDR: u16 = 256;
pub const OPCODE_TABLE_SIZE: usize = 256;
pub const IRQ_ADDR: u16 = 0xFFFE;
pub const NMI_ADDR: u16 = 0xFFFA;
pub const RESET_ADDR: u16 = 0xFFFC;

pub const CYCLES: [u8; OPCODE_TABLE_SIZE] = [
	7, 6, 2, 2, 2, 3, 5, 2, 3, 2, 2, 2, 2, 4, 6, 2,
	2, 5, 2, 2, 2, 4, 6, 2, 2, 4, 2, 2, 2, 4, 7, 2,
	6, 6, 2, 2, 3, 3, 5, 2, 4, 2, 2, 2, 4, 4, 6, 2,
	2, 5, 2, 2, 2, 4, 6, 2, 2, 4, 2, 2, 2, 4, 7, 2,
	6, 6, 2, 2, 2, 3, 5, 2, 3, 2, 2, 2, 3, 4, 6, 2,
	2, 5, 2, 2, 2, 4, 6, 2, 2, 4, 2, 2, 2, 4, 7, 2,
];

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(usize)]
pub enum Opcode {
	BRK = 0,
	ORA_IZX,
	ORA_ZP0 = 5,
	ASL_ZP0,
	PHP = 8,
	ORA_IMM,
	ASL,
	ORA_ABS = 0xD,
	ASL_ABS,

	BPL_REL = 0x10,
	ORA_IZY,
	ORA_ZPX = 0x15,
	ASL_ZPX,
	CLC = 0x18,
	ORA_ABY,
	ORA_ABX = 0x1D,
	ASL_ABX,

	JSR_ABS = 0x20,
	AND_IZX,
	BIT_ZP0 = 0x24,
	AND_ZP0,
	ROL_ZP0,
	PLP = 0x28,
	AND_IMM,
	ROL,
	BIT_ABS = 0x2C,
	AND_ABS,
	ROL_ABS,

	BMI_REL = 0x30,
	AND_IZY,
	AND_ZPX = 0x35,
	ROL_ZPX,
	SEC = 0x38,
	AND_ABY,
	AND_ABX = 0x3D,
	ROL_ABX,

	RTI = 0x40,
	EOR_IZX,
	EOR_ZP0 = 0x45,
	LSR_ZP0,
	PHA = 0x48,
	EOR_IMM,
	LSR,
	JMP_ABS = 0x4C,
	EOR_ABS,
	LSR_ABS,

	BVC_REL = 0x50,
	EOR_IZY,
	EOR_ZPX = 0x55,
	LSR_ZPX,
	CLI = 0x58,
	EOR_ABY,
	EOR_ABX = 0x5D,
	LSR_ABX,

	RTS = 0x60,
	ADC_IZX,
	ADC_ZP0 = 0x65,
	ROR_ZP0,
	PLA = 0x68,
	ADC_IMM,
	ROR,
	JMP_IND = 0x6C,
	ABC_ABS,
	ROR_ABS,

	BVS_REL = 0x70,
	ADC_IZY,
	ADC_ZPX = 0x75,
	ROR_ZPX,
	SEI = 0x78,
	ADC_ABY,
	ADC_ABX = 0x7D,
	ROR_ABX,

	STA_IZX =
}

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
	pub bus: &'a Bus<'a>,
	pub registers: Registers,
	pub cache: Cache,
	pub opcodes: [Opcode; OPCODE_TABLE_SIZE],
}

impl<'a> Processor<'a> {
	pub fn new(bus: &'a mut Bus) -> Processor<'a> {
		Processor {
			bus: bus,
			registers: Registers::default(),
			cache: Cache::default(),
			opcodes: [NOP; OPCODE_TABLE_SIZE],
				/*Opcode::new(opcode_id!(b"BRK"), Processor::imm, Processor::brk, 7),
				Opcode::new(opcode_id!(b"ORA"), Processor::izx, Processor::ora, 6),
				Opcode::new(opcode_id!(b"BRK"), Processor::imm, Processor::brk, 7),
				NOP,
				NOP,
				NOP,
				Opcode::new(opcode_id!(b"ORA"), Processor::zp0, Processor::ora, 3),
			],*/
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
	pub fn interrupt(&mut self, new_abs_addr: u16, new_cycles: u8) {
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
		let _ = self.fetch();
		let carry = if self.check(Status::CARRY) { 1 } else { 0 };
		let temp16 = (self.get_a() as u16) + (self.get_data() as u16) + carry;

		self.set_flag(Status::CARRY, temp16 > 255);
		self.set_flag(Status::ZERO, temp16 & 255 == 0);
		self.set_flag(Status::NEGATIVE, temp16 & 128 != 0);

		self.set_flag(Status::OVERFLOW, !(((self.get_a() as u16) ^ (self.get_data() as u16)) &
			((self.get_a() as u16) ^ temp16)) & 128);

		self.set_a((temp16 & 255) as u8);

		1
	}

	/// Bitwise and
	pub fn and(&mut self) -> u8 {
		let _ = self.fetch();
		self.registers.accumulator &= self.get_data();
		self.set_flag(Status::ZERO, self.get_a() == 0);
		self.set_flag(Status::NEGATIVE, self.get_a() & 128 != 0);

		1
	}

	/// Arithmetical left shift
	pub fn asl(&mut self) -> u8 {
		let _ = self.fetch();
		let temp16 = (self.get_data() as u16) << 1;

		self.set_flag(Status::CARRY, temp16 & 0xFF > 0);
		self.set_flag(Status::ZERO, temp16 & 255 == 0);
		self.set_flag(Status::NEGATIVE, temp16 & 128 != 0);

		if self.oc_addr_mode(self.oc_index()) == Processor::imp {
			self.set_a((temp16 & 255) as u8);
		} else {
			self.write(self.abs_addr(), (temp16 & 255) as u8);
		}

		0
	}

	/// Branching if carry clear
	pub fn bcc(&mut self) -> u8 {
		if !self.check(Status::CARRY) {
			self.branch();
		}

		0
	}

	/// Branching if carry
	pub fn bcs(&mut self) -> u8 {
		if self.check(Status::CARRY) {
			self.branch();
		}

		0
	}

	/// Branching if carry
	pub fn beq(&mut self) -> u8 {
		if self.check(Status::ZERO) {
			self.branch();
		}

		0
	}

	/// Branching if negative
	pub fn bmi(&mut self) -> u8 {
		if self.check(Status::NEGATIVE) {
			self.branch();
		}

		0
	}

	/// Branching if not equal
	pub fn bne(&mut self) -> u8 {
		if !self.check(Status::ZERO) {
			self.branch();
		}

		0
	}

	/// Branching if positive
	pub fn bpl(&mut self) -> u8 {
		if !self.check(Status::NEGATIVE) {
			self.branch();
		}

		0
	}

	/// Program-sourced interrupt.
	pub fn brk(&mut self) -> u8 {
		// This differs slightly from self.interrupt()

		self.incr();

		self.set_flag(Status::NO_INTERRUPTS, true);
		self.stack_write_addr(self.counter());

		self.set_flag(Status::BREAK, true);
		self.stack_write(self.status_bits());
		self.set_flag(Status::BREAK, false);

		self.set_pc(self.read_addr(IRQ_ADDR));
		0
	}

	/// Branching if overflow
	pub fn bvc(&mut self) -> u8 {
		if self.check(Status::OVERFLOW) {
			self.branch();
		}

		0
	}

	/// Branching if not overflow
	pub fn bvs(&mut self) -> u8 {
		if !self.check(Status::OVERFLOW) {
			self.branch();
		}

		0
	}

	/// Clear carry status bit
	pub fn clc(&mut self) -> u8 {
		self.set_flag(Status::CARRY, false);
		0
	}

	/// Clear decimal status bit
	pub fn cld(&mut self) -> u8 {
		self.set_flag(Status::DECIMAL, false);
		0
	}

	/// Clear interrupt disable status bit
	pub fn cli(&mut self) -> u8 {
		self.set_flag(Status::NO_INTERRUPTS, false);
		0
	}

	/// Clear overflow status bit
	pub fn clv(&mut self) -> u8 {
		self.set_flag(Status::OVERFLOW, false);
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
		if !self.check(Status::NO_INTERRUPTS) {
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

	/// Push accumulator to the stack
	pub fn pha(&mut self) -> u8 {
		self.stack_write(self.get_a());
		0
	}

	/// Pop accumulator from the stack
	pub fn pla(&mut self) -> u8 {
		self.set_a(self.stack_read());
		self.set_flag(Status::ZERO, self.get_a() == 0);
		self.set_flag(Status::NEGATIVE, self.get_a() & 128 != 0);

		0
	}

	/// Relative address mode (branching instructions)
	pub fn rel(&mut self) -> u8 {
		self.cache.rel_addr = self.read(self.counter());
		self.incr();

		// check for signed bit
		if self.rel_addr() & 128 != 0 {
			self.cache.rel_addr |= 0xFF00;
		}

		0
	}

	/// Restores state from interrupt
	pub fn rti(&mut self) -> u8 {
		// restore status flags
		self.registers.status = Status::from_bits_truncate(self.stack_read());
		self.registers.status &= !Status::BREAK;
		self.registers.status &= !Status::UNUSED;

		// and counter
		self.set_pc(self.stack_read_rom_addr());

		0
	}

	/// Subtraction with carry
	pub fn sdc(&mut self) -> u8 {
		let _ = self.fetch();
		let value = (self.get_data() as u16) ^ 255; // invert the value
		let carry = if self.check(Status::CARRY) { 1 } else { 0 };
		let temp16 = (self.get_a() as u16) + value + carry;

		self.set_flag(Status::CARRY, temp16 & 0xFF00 != 0);
		self.set_flag(Status::ZERO, temp16 & 255 == 0);
		self.set_flag(Status::OVERFLOW, (temp16 ^ (self.get_a() as u16)) & (temp16 ^ value) & 128);
		self.set_flag(Status::NEGATIVE, temp16 & 128 != 0);

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
