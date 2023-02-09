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
	DeviceMap,
	Disassembler,
	hexdump,
	Processor,
	RawRegionMap,
	Region,
	RegionFlags,
	RegionType
};

use crate::MOS6502Disassembler;

/// Offset of program stack
const STACK_ADDR: usize = 256;

/// Offset of stack pointer initiation
const STACK_INIT: usize = 253;

/// Offset of interrupt request vector
const IRQ_ADDR: usize = 65534;

/// Offset of non-maskable interrupt vector
const NMI_ADDR: usize = 65530;

/// Offset of reset vector
const RES_ADDR: usize = 65532;

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
		let cpu = MOS6502 {
			bus: Rc::clone(&bus),
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

		//cpu.generate_regions(0, 65536);

		cpu
	}

	/// Add additional cycles to the current operation
	pub fn add_cycles(&mut self, value: u8) {
		self.cache.cycles += value;
	}

	pub fn branch(&mut self) {
		self.add_cycles(1);
		let new_addr = ((self.get_counter() as isize) + (self.get_rel_addr() as isize)) & 65535;
		self.set_abs_addr(new_addr as usize);

		// need an additional cycle if different page
		if self.get_abs_hi() != (self.get_counter() & 0xFF00) {
			self.add_cycles(1);
		}

		// jump to the address
		self.set_counter(self.get_abs_addr());
	}

	/// Checks specified status flag(s)
	pub const fn check_flag(&self, flag: Status) -> bool {
		self.regs.p.contains(flag)
	}

	/// Assign to accumulator, or write to bus, depending on the address mode
	pub fn check_mode(&mut self, value: u16) {
		if self.get_mode() == Mode::IMP {
			self.set_a((value & 255) as u8);
		} else {
			self.write_last((value & 255) as u8);
		}
	}

	/// Check for page change
	pub const fn check_page(&self, addr: usize) -> u8 {
		if self.get_abs_hi() != (addr & 0xFF00) {
			1
		} else {
			0
		}
	}

	/// Fetch byte from an operation
	pub fn fetch(&mut self) -> u8 {
		if self.get_mode() != Mode::IMP {
			self.set_data(self.get_u8(self.get_abs_addr()));
		}

		self.get_data()
	}

	/// Fetch byte from an operation as 16-bit
	pub fn fetch16(&mut self) -> u16 {
		self.fetch() as u16
	}

	/// Fetch address
	pub fn fetch_addr(&mut self) -> usize {
		u16::from_le_bytes([self.fetch(), self.fetch()]).into()
	}

	/// Gets the accumulator register value
	pub const fn get_a(&self) -> u8 {
		self.regs.a
	}

	/// Gets the accumulator register value as a 16-bit value
	pub const fn get_a16(&self) -> u16 {
		self.regs.a as u16
	}

	/// Gets cached absolute address
	pub const fn get_abs_addr(&self) -> usize {
		self.cache.abs_addr
	}

	/// Gets cached absolute address' high byte
	pub const fn get_abs_hi(&self) -> usize {
		self.get_abs_addr() & 0xFF00
	}

	/// Gets the carry bit
	pub const fn get_carry(&self) -> u16 {
		(self.check_flag(Status::C) as u16) & 1
	}

	/// Gets the program counter register value
	pub const fn get_counter(&self) -> usize {
		self.regs.pc
	}

	/// Gets the remaining cycle count
	pub const fn get_cycles(&self) -> u8 {
		self.cache.cycles
	}

	/// Gets the currently cached data byte
	pub const fn get_data(&self) -> u8 {
		self.cache.data
	}

	/// Gets the currently cached data byte as 16-bit
	pub const fn get_data16(&self) -> u16 {
		self.cache.data as u16
	}

	/// Retrieves the currently cached address mode
	pub const fn get_mode(&self) -> Mode {
		self.cache.mode
	}

	/// Gets the currently cached opcode
	pub const fn get_opcode(&self) -> usize {
		self.cache.opcode as usize
	}

	/// Retrieve the registry state flag bits
	pub const fn get_p_bits(&self) -> u8 {
		self.regs.p.bits()
	}

	/// Gets cached relative address
	pub const fn get_rel_addr(&self) -> usize {
		self.cache.rel_addr
	}

	/// Gets the stack pointer register value
	pub const fn get_sp(&self) -> usize {
		self.regs.s
	}

	/// Gets the X register value
	pub const fn get_x(&self) -> u8 {
		self.regs.x
	}

	/// Gets the X register value as 16-bit
	pub const fn get_x16(&self) -> u16 {
		self.regs.x as u16
	}

	/// Gets X register value as a zero page address
	pub const fn get_x_zp_addr(&self) -> usize {
		self.regs.x as usize
	}

	/// Gets the Y register value
	pub const fn get_y(&self) -> u8 {
		self.regs.y
	}

	/// Gets the Y register value as 16-bit
	pub const fn get_y16(&self) -> u16 {
		self.regs.y as u16
	}

	/// Gets Y register value as a zero page address
	pub const fn get_y_zp_addr(&self) -> usize {
		self.regs.y as usize
	}

	/// Get zero-page address
	pub fn get_zp_addr(&self, address: usize) -> usize {
		self.get_u8(address) as usize
	}

	/// Increment program pc registry by 1
	pub fn incr(&mut self) {
		self.regs.pc += 1;
	}

	/// Interrupts the execution state
	pub fn interrupt(&mut self, new_abs_addr: usize, new_cycles: u8) {
		// write the counter's current value to stack
		self.stack_write_addr(self.get_counter());

		// write p register to stack too
		self.set_flag(Status::B, false);
		self.set_flag(Status::U, true);
		self.set_flag(Status::I, true);
		self.stack_write(self.get_p_bits());

		// get the new pc value
		self.set_abs_addr(new_abs_addr);
		let addr = self.fetch_addr();
		self.set_counter(addr);

		self.cache.cycles = new_cycles;
	}

	/// Reads an address from the RAM
	pub fn read_addr(&self, addr: usize) -> usize {
		self.bus.borrow().get_u16_le(addr) as usize
	}

	/// Reads a byte from the ROM
	pub fn read_rom(&mut self) -> u8 {
		let data = self.get_u8(self.get_counter());
		self.incr();

		data
	}

	/// Reads an address from the ROM
	pub fn read_rom_addr(&mut self) -> usize {
		u16::from_le_bytes([self.read_rom(), self.read_rom()]).into()
	}

	/// Reads an 8-bit address from the ROM
	pub fn read_rom_zp_addr(&mut self) -> usize {
		self.read_rom().into()
	}

	/// Sets accumulator register value
	pub fn set_a(&mut self, value: u8) {
		self.regs.a = value;
	}

	/// Sets cached absolute address
	pub fn set_abs_addr(&mut self, value: usize) {
		self.cache.abs_addr = value;
	}

	/// Sets program counter register value
	pub fn set_counter(&mut self, value: usize) {
		self.regs.pc = value;
	}

	/// Sets cycle count
	pub fn set_cycles(&mut self, value: u8) {
		self.cache.cycles = value;
	}

	/// Sets cached data
	pub fn set_data(&mut self, value: u8) {
		self.cache.data = value;
	}

	/// Sets status register flag
	pub fn set_flag(&mut self, flags: Status, condition: bool) {
		self.regs.p.set(flags, condition);
	}

	/// Set carry, negative, and/or zero bits of state flags register, given a value
	pub fn set_flags_cnz(&mut self, value: u16) {
		self.set_flag(Status::C, value > 255);
		self.set_flags_nz(value);
	}

	/// Set negative and/or zero bits of state flags register, given a value
	pub fn set_flags_nz(&mut self, value: u16) {
		self.set_if_0(value);
		self.set_if_neg(value);
	}

	/// Set the flag if the value is zero
	pub fn set_if_0(&mut self, value: u16) {
		self.set_flag(Status::Z, (value & 255) == 0)
	}

	/// Set the flag if the value is negative
	pub fn set_if_neg(&mut self, value: u16) {
		self.set_flag(Status::N, value & 128 != 0)
	}

	/// Set cached address mode. Only address mode functions should use this!
	pub fn set_mode(&mut self, mode: Mode) {
		self.cache.mode = mode;
	}

	/// Sets the relative address
	pub fn set_rel_addr(&mut self, value: usize) {
		self.cache.rel_addr = value;
	}

	/// Sets stack pointer
	pub fn set_sp(&mut self, value: usize) {
		self.regs.s = value;
	}

	/// Sets X register value
	pub fn set_x(&mut self, value: u8) {
		self.regs.x = value;
	}

	/// Sets Y register value
	pub fn set_y(&mut self, value: u8) {
		self.regs.y = value;
	}

	/// Convenience function to read from stack
	pub fn stack_read(&mut self) -> u8 {
		self.regs.s += 1;
		self.get_u8(STACK_ADDR + self.get_sp())
	}

	/// Reads an address from stack
	pub fn stack_read_addr(&mut self) -> usize {
		u16::from_le_bytes([self.stack_read(), self.stack_read()]) as usize
	}

	/// Convenience function to write to stack
	pub fn stack_write(&mut self, data: u8) {
		self.write(STACK_ADDR + self.get_sp(), &[data]);
		self.regs.s -= 1;
	}

	/// Returns a string of the stackdump
	pub fn stackdump(&self) -> String {
		let dump = self.read(STACK_ADDR, 256);
		hexdump(&dump[..], 2)
	}

	/// Writes an address to stack
	pub fn stack_write_addr(&mut self, addr: usize) {
		self.stack_write(((addr & 0xFF00) >> 8) as u8);
		self.stack_write((addr & 255) as u8);
	}

	/// Writes to the last absolute address
	pub fn write_last(&mut self, data: u8) {
		self.write(self.get_abs_addr(), &[data]);
	}

	// --- SYSTEM FLOW

	/// Sends an interrupt request if able
	pub fn irq(&mut self) {
		if !self.check_flag(Status::I) {
			self.interrupt(IRQ_ADDR, 7);
		}
	}

	/// Sends a non-maskable interrupt
	pub fn nmi(&mut self) {
		self.interrupt(NMI_ADDR, 8);
	}

	/// Resets the regs and cache
	pub fn res(&mut self) {
		self.set_a(0);
		self.set_flag(Status::default(), true);
		self.set_x(0);
		self.set_y(0);
		self.set_sp(STACK_INIT);

		self.set_abs_addr(RES_ADDR);
		let addr = self.fetch_addr();
		self.set_counter(addr);

		self.cache.rel_addr = 0;
		self.set_abs_addr(0);
		self.set_data(0);

		self.cache.cycles = 8;
	}

	// --- ADDRESS MODES

	/// Absolute address mode
	pub fn abs(&mut self) -> u8 {
		self.set_mode(Mode::ABS);
		let addr = self.read_rom_addr();
		self.set_abs_addr(addr);

		0
	}

	/// Absolute address mode with X register offset
	pub fn abx(&mut self) -> u8 {
		self.set_mode(Mode::ABX);

		let addr = self.read_rom_addr();
		self.set_abs_addr(addr + self.get_x_zp_addr());

		self.check_page(addr)
	}

	/// Absolute address mode with Y register offset
	pub fn aby(&mut self) -> u8 {
		self.set_mode(Mode::ABY);

		let addr = self.read_rom_addr();
		self.set_abs_addr(addr + self.get_y_zp_addr());

		self.check_page(addr)
	}

	/// Immediate address mode
	pub fn imm(&mut self) -> u8 {
		self.set_mode(Mode::IMM);
		self.incr();
		self.set_abs_addr(self.get_counter());
		0
	}

	/// Implied address mode
	pub fn imp(&mut self) -> u8 {
		self.set_mode(Mode::IMP);
		self.set_data(self.get_a());
		0
	}

	/// Indirect address mode (pointer access)
	pub fn ind(&mut self) -> u8 {
		self.set_mode(Mode::IND);

		let ptr = self.read_rom_addr();

		if (ptr & 255) == 255 {
			// page boundary hardware bug
			self.set_abs_addr(self.read_addr(ptr));
		} else {
			// normal behavior
			self.set_abs_addr(self.read_addr(ptr));
		}

		0
	}

	/// Indirect address mode of zero-page with X register offset
	pub fn izx(&mut self) -> u8 {
		self.set_mode(Mode::IZX);

		let t = self.read_rom_zp_addr();
		let lo = self.get_zp_addr((t + self.get_x_zp_addr()) & 255);
		let hi = self.get_zp_addr((t + self.get_x_zp_addr() + 1) & 255);

		self.set_abs_addr((hi << 8) | lo);
		0
	}

	/// Indirect address mode of zero-page with Y register offset
	pub fn izy(&mut self) -> u8 {
		self.set_mode(Mode::IZY);

		let t = self.read_rom_zp_addr();
		let lo = self.get_zp_addr(t & 255);
		let hi = self.get_zp_addr((t + 1) & 255);

		self.set_abs_addr(((hi << 8) | lo) + self.get_y_zp_addr());

		if self.get_abs_hi() != (hi << 8) { 1 } else { 0 }
	}

	/// Relative address mode (branching instructions)
	pub fn rel(&mut self) -> u8 {
		self.set_mode(Mode::REL);
		self.cache.rel_addr = self.read_rom_zp_addr();

		// check_flag for signed bit
		if self.get_rel_addr() & 128 != 0 {
			self.cache.rel_addr |= 0xFF00;
		}

		0
	}

	/// Zero-page address mode
	pub fn zpg(&mut self) -> u8 {
		self.set_mode(Mode::ZPG);
		let addr = self.read_rom_zp_addr();
		self.set_abs_addr(addr);
		self.incr();
		self.cache.abs_addr &= 255;
		0
	}

	/// Zero-page address mode with X register offset
	pub fn zpx(&mut self) -> u8 {
		self.set_mode(Mode::ZPX);
		let addr = self.read_rom_zp_addr();
		self.set_abs_addr(addr + self.get_x_zp_addr());
		self.cache.abs_addr &= 255;
		0
	}

	/// Zero-page address mode with Y register offset
	pub fn zpy(&mut self) -> u8 {
		self.set_mode(Mode::ZPY);
		let addr = self.read_rom_zp_addr();
		self.set_abs_addr(addr + self.get_y_zp_addr());
		self.cache.abs_addr &= 255;
		0
	}

	// --- OPERATIONS

	/// Addition with carry
	pub fn adc(&mut self) -> u8 {
		let fetch = self.fetch16();
		let tmp = self.get_a16() + (fetch + self.get_carry());

		self.set_flags_cnz(tmp);

		self.set_flag(Status::V, !(((self.get_a16() ^ self.get_data16()) &
			(self.get_a16() ^ tmp)) & 128) == 0);

		self.set_a((tmp & 255) as u8);

		1
	}

	/// Bitwise and
	pub fn and(&mut self) -> u8 {
		self.regs.a &= self.fetch();
		self.set_flags_nz(self.get_a16());

		1
	}

	/// Arithmetical left shift
	pub fn asl(&mut self) -> u8 {
		let tmp = self.fetch16() << 1;
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

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

	/// Bit test
	pub fn bit(&mut self) -> u8 {
		let fetch = self.fetch16();
		self.set_if_0(self.get_a16() & fetch);
		self.set_if_neg(self.get_data16());
		self.set_flag(Status::V, self.get_data() & 64 != 0);

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
		self.stack_write_addr(self.get_counter());
		self.set_flag(Status::B, true);
		self.stack_write(self.get_p_bits());
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
		self.set_flag(Status::D, false);
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

	/// Compare with accumulator
	pub fn cmp(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_flag(Status::C, self.get_a() >= fetch);
		self.set_flags_nz(self.get_a16() - self.get_data16());

		1
	}

	/// Compare with X
	pub fn cpx(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_flag(Status::C, self.get_a() >= fetch);
		self.set_flags_nz(self.get_x16() - self.get_data16());

		1
	}

	/// Compare with Y
	pub fn cpy(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_flag(Status::C, self.get_y() >= fetch);
		self.set_flags_nz(self.get_y16() - self.get_data16());

		1
	}

	/// Decrement value
	pub fn dec(&mut self) -> u8 {
		let b = self.fetch() - 1;
		self.write_last(b);
		self.set_flags_nz(b as u16);

		0
	}

	/// Decrement X register
	pub fn dex(&mut self) -> u8 {
		self.regs.x -= 1;
		self.set_flags_nz(self.get_x16());

		0
	}

	/// Decrement Y register
	pub fn dey(&mut self) -> u8 {
		self.regs.y -= 1;
		self.set_flags_nz(self.get_y16());

		0
	}

	/// Exclusive or
	pub fn eor(&mut self) -> u8 {
		self.regs.a ^= self.fetch();
		self.set_flags_nz(self.get_a16());

		1
	}

	/// Increment value
	pub fn inc(&mut self) -> u8 {
		let b = self.fetch() + 1;
		self.write_last(b);
		self.set_flags_nz(b as u16);

		0
	}

	/// Increment X register
	pub fn inx(&mut self) -> u8 {
		self.regs.x += 1;
		self.set_flags_nz(self.get_x16());

		0
	}

	/// Increment Y register
	pub fn iny(&mut self) -> u8 {
		self.regs.y += 1;
		self.set_flags_nz(self.get_y16());

		0
	}

	/// Jump to address
	pub fn jmp(&mut self) -> u8 {
		self.set_counter(self.get_abs_addr());
		0
	}

	/// Jump to subroutine
	pub fn jsr(&mut self) -> u8 {
		self.stack_write_addr(self.get_counter());
		self.set_counter(self.get_abs_addr());
		0
	}

	/// Load into accumulator
	pub fn lda(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_a(fetch);
		self.set_flags_nz(self.get_a16());
		1
	}

	/// Load into X
	pub fn ldx(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_x(fetch);
		self.set_flags_nz(self.get_x16());
		1
	}

	/// Load into Y
	pub fn ldy(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_y(fetch);
		self.set_flags_nz(self.get_y16());
		1
	}

	/// Logical right shift
	pub fn lsr(&mut self) -> u8 {
		let tmp = self.fetch16() >> 1;
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// No operation, illegal opcode filler
	pub fn nop(&self) -> u8 {
		match self.get_opcode() {
			28 | 60 | 92 | 124 | 220 | 252 => 1,
			_ => 0,
		}
	}

	/// Bitwise or
	pub fn ora(&mut self) -> u8 {
		self.regs.a |= self.fetch();
		self.set_flags_nz(self.get_a16());

		1
	}

	/// Push accumulator register to the stack
	pub fn pha(&mut self) -> u8 {
		self.stack_write(self.get_a());
		0
	}

	/// Push state register to the stack
	pub fn php(&mut self) -> u8 {
		self.set_flag(Status::B, true);
		self.set_flag(Status::U, true);
		self.stack_write(self.get_p_bits());
		self.set_flag(Status::B, false);
		self.set_flag(Status::U, false);

		0
	}

	/// Pop accumulator register from the stack
	pub fn pla(&mut self) -> u8 {
		let b = self.stack_read();
		self.set_a(b);
		self.set_flags_nz(self.get_a16());

		0
	}

	/// Pop state register from the stack
	pub fn plp(&mut self) -> u8 {
		self.regs.p = Status::from_bits_truncate(self.stack_read());
		self.set_flag(Status::U, true);

		0
	}

	/// Bit rotate left
	pub fn rol(&mut self) -> u8 {
		let tmp = self.fetch16().rotate_left(1);
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Bit rotate right
	pub fn ror(&mut self) -> u8 {
		let tmp = self.fetch16().rotate_right(1);
		self.set_flag(Status::C, (self.get_data() & 1) != 0);
		self.set_flags_nz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Restores state from interrupt
	pub fn rti(&mut self) -> u8 {
		// restore state flags
		self.regs.p = Status::from_bits_truncate(self.stack_read());
		self.regs.p.toggle(Status::B);
		self.regs.p.toggle(Status::U);

		// and counter
		let addr = self.stack_read_addr();
		self.set_counter(addr);

		0
	}

	/// Return from subroutine
	pub fn rts(&mut self) -> u8 {
		let addr = self.stack_read_addr();
		self.set_counter(addr);
		0
	}

	/// Subtraction with carry
	pub fn sbc(&mut self) -> u8 {
		let value = self.fetch16() ^ 255; // invert the value
		let tmp = self.get_a16() + value + self.get_carry();

		self.set_flags_cnz(tmp);
		self.set_flag(Status::V, (tmp ^ self.get_a16() & (tmp ^ value)) & 128 != 0);
		self.set_a((tmp & 255) as u8);

		1
	}

	/// Set carry bit
	pub fn sec(&mut self) -> u8 {
		self.set_flag(Status::C, true);
		0
	}

	/// Set decimal bit
	pub fn sed(&mut self) -> u8 {
		self.set_flag(Status::D, true);
		0
	}

	/// Set interrupt disable bit
	pub fn sei(&mut self) -> u8 {
		self.set_flag(Status::I, true);
		0
	}

	/// Store accumulator at address
	pub fn sta(&mut self) -> u8 {
		self.write_last(self.get_a());
		0
	}

	/// Store X at address
	pub fn stx(&mut self) -> u8 {
		self.write_last(self.get_x());
		0
	}

	/// Store Y at address
	pub fn sty(&mut self) -> u8 {
		self.write_last(self.get_y());
		0
	}

	/// Transfer accumulator to X
	pub fn tax(&mut self) -> u8 {
		self.set_x(self.get_a());
		self.set_flags_nz(self.get_x16());
		0
	}

	/// Transfer accumulator to Y
	pub fn tay(&mut self) -> u8 {
		self.set_y(self.get_a());
		self.set_flags_nz(self.get_y16());
		0
	}

	/// Transfer stack pointer to X
	pub fn tsx(&mut self) -> u8 {
		self.set_x(self.get_sp() as u8);
		self.set_flags_nz(self.get_x16());
		0
	}

	/// Transfer X to accumulator
	pub fn txa(&mut self) -> u8 {
		self.set_a(self.get_x());
		self.set_flags_nz(self.get_a16());
		0
	}

	/// Transfer X to stack pointer
	pub fn txs(&mut self) -> u8 {
		self.set_sp(self.get_x_zp_addr());
		0
	}

	/// Transfer Y to accumulator
	pub fn tya(&mut self) -> u8 {
		self.set_a(self.get_y());
		self.set_flags_nz(self.get_a16());
		0
	}
}

impl Display for MOS6502 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "{}", &self.regs);
		writeln!(f, "{}", &self.cache)
	}
}

impl DeviceMap for MOS6502 {
	fn add_region(&mut self, address: usize, region: Region) {
		self.bus.borrow_mut().add_region(address, region);
	}

	fn generate_regions(&mut self, start: usize, end: usize) {
		let irq_vec = self.get_u16_le(IRQ_ADDR);
		let nmi_vec = self.get_u16_le(NMI_ADDR);
		let res_vec = self.get_u16_le(RES_ADDR);

		let mut irq_r = Region::new(0, RegionType::Function, RegionFlags::default(),
			format!("FUN_{:04X}", irq_vec).as_str());
		irq_r.add_ref(IRQ_ADDR);

		let mut nmi_r = Region::new(0, RegionType::Function, RegionFlags::default(),
			format!("FUN_{:04X}", nmi_vec).as_str());
		nmi_r.add_ref(NMI_ADDR);

		let mut res_r = Region::new(0, RegionType::Function, RegionFlags::default(),
			format!("FUN_{:04X}", res_vec).as_str());
		res_r.add_ref(RES_ADDR);

		self.add_regions(RawRegionMap::from([
			(0, Region::new(256, RegionType::Section, RegionFlags::default(), "ZERO_PAGE")),
			(IRQ_ADDR, Region::new(2, RegionType::Pointer, RegionFlags::PTR, "IRQ")),
			(NMI_ADDR, Region::new(2, RegionType::Pointer, RegionFlags::PTR, "NMI")),
			(RES_ADDR, Region::new(2, RegionType::Pointer, RegionFlags::PTR, "RES")),
			(STACK_ADDR, Region::new(256, RegionType::Unsigned8, RegionFlags::ARRAY, "STACK")),
			(irq_vec as usize, irq_r),
			(nmi_vec as usize, nmi_r),
			(res_vec as usize, res_r),
		]));

		let mut offset = start;

		while offset < end {
			let op = self.get_u8(offset) as usize;
			offset += 1;

			match op {
				// relative
				16 | 48 | 80 | 112 | 144 | 176 | 208 | 240 => {
					let addr = ((offset as i32) + (self.get_i8(offset) as i32) + 1) as u16;

					if self.region_exists(addr as usize) {
						if let Some(r) = self.get_region_mut(addr as usize) {
							let mut r = r.borrow_mut();
							r.add_ref(offset - 1);
						}
					} else {
						let mut r = Region::new(0, RegionType::Label,  RegionFlags::default(),
							format!("LAB_{:04X}", addr).as_str());
						r.add_ref(offset - 1);
						self.add_region(addr as usize, r);
					}

					offset += 1;
				},

				32 => { // JSR, label is a function
					let addr = self.get_u16_le(offset) as usize;

					if self.region_exists(addr) {
						if let Some(r) = self.get_region_mut(addr as usize) {
							let mut r = r.borrow_mut();
							r.label_to_fn(Some(format!("FUN_{:04X}", addr & 65535).as_str()));
							r.add_ref(offset - 1);
						}
					} else {
						let mut r = Region::new(0, RegionType::Function,  RegionFlags::default(),
							format!("FUN_{:04X}", addr & 65535).as_str());
						r.add_ref(offset - 1);
						self.add_region(addr, r);
					}

					offset += 2;
				},

				76 | 108 => { // JMP absolute or indirect
					let addr = self.get_u16_le(offset);

					if self.region_exists(addr as usize) {
						if let Some(r) = self.get_region_mut(addr as usize) {
							let mut r = r.borrow_mut();
							r.add_ref(offset - 1);
						}
					} else {
						let mut r = Region::new(0, RegionType::Label, RegionFlags::default(),
							format!("LAB_{:04X}", addr).as_str());
						r.add_ref(offset - 1);
						self.add_region(addr as usize, r);
					}

					offset += 2;
				},

				// absolute
				12..=15 | 25 | 27..=31 | 44..=47 | 57 | 59..=63 | 77..=79 | 89 | 91..=95 | 109..=111 | 121 | 123..=127 |
				140..=143 | 153 | 155..=159 | 172..=175 | 185 | 187..=191 | 204..=207 | 217 | 219..=223 | 236..=239 |
				249 | 251..=255 => {
					offset += 2;
				},

				// indirect, zero page, immediate
				1 | 3..=7 | 9 | 11 | 17 | 19..=23 | 33 | 35..=39 | 41 | 43 | 49 | 51..=55 | 65 | 67..=71 | 73 | 75 |
				81 | 83..=87 | 97 | 99..=103 | 105 | 107 | 113 | 115..=119 | 128..=135 | 137 | 139 | 145 | 147..=151 |
				160..=167 | 169 | 171 | 177 | 179..=183 | 192..=199 | 201 | 203 | 209 | 211..=215 | 224..=231 | 233 |
				235 | 241 | 243..=247 => {
					offset += 1;
				},

				// implied
				_ => (),
			}
		}

		//self.sort_regions();
	}

	fn region_exists(&self, offset: usize) -> bool {
		self.bus.borrow().region_exists(offset)
	}

	fn sort_regions(&mut self) {
		self.bus.borrow_mut().sort_regions();
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

	fn get_region(&self, offset: usize) -> Option<Rc<RefCell<Region>>> {
		self.bus.borrow().get_region(offset)
	}

	fn get_region_mut(&mut self, offset: usize) -> Option<Rc<RefCell<Region>>> {
		self.bus.borrow_mut().get_region_mut(offset)
	}

	fn get_all_regions(&self) -> Vec<(usize, Region)> {
		self.bus.borrow().get_all_regions()
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

	fn get_ptr_size(&self) -> usize {
		2
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::{
		DisassemblerConfig,
		MOS6502Disassembler
	};

	use std::io::stdin;

	#[test]
	fn test_exec() {
		//let mario = include_bytes!("/home/admin/Downloads/Super Mario Bros (PC10).nes");

		let hundred_doors = vec![0xa9,0x00,0xa2,0x64,0x95,0xc8,0xca,0xd0,0xfb,0x95,0xc8,0xa0,
			0x01,0xc0,0x65,0xb0,0x12,0x98,0xc9,0x65,0xb0,0x0a,0xaa,0xfe,0x00,0x02,0x84,0x01,0x65,
			0x01,0x90,0xf2,0xc8,0xd0,0xea,0xa2,0x64,0xbd,0x00,0x02,0x29,0x01,0x9d,0x00,0x02,0xca,
			0xd0,0xf5];

		let mut bus = Bus::new(65536);
		//bus.write(32768, &mario[16..32784]);
		bus.write(32768, &hundred_doors[..]);

		let mut cpu = MOS6502::new(Rc::new(RefCell::new(bus)));
		let cfg = DisassemblerConfig::LOWERCASE | DisassemblerConfig::OFFSETS;
		let mut da = MOS6502Disassembler::new(cpu.get_bus(), Some(cfg));

		let mut input = String::new();
		let mut offs = 0;
		cpu.set_counter(32768);
		while let Ok(_) = stdin().read_line(&mut input) {
			match input.chars().nth(0).unwrap() {
				's' | 'S' => println!("{}", cpu.stackdump()),
				_ => {
					loop {
						cpu.clock();
						if cpu.get_cycles() == 0 {
							offs = cpu.get_counter();
							//dbg!(offs);
							let (_, code) = da.analyze(&mut offs);
							println!("{}", &cpu);
							println!("{code}");
							break;
						}
					}
				},
			}

			input.clear();
		}

		println!("stopped");
	}
}
