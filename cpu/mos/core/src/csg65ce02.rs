mod mos6502;

use bitflags::bitflags;

use std::fmt::{
	Display,
	Formatter,
	self
};

use mos6502::{
	Mode
	MOS6502,
	Status
};

use rgk_processors_core::{
	DeviceBase,
	Processor
};

/// Extended address mode
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExMode {
	/// Absolute
	ABS,

	/// Absolute word
	ABW,

	/// Absolute with X offset
	ABX,

	/// Absolute with Y offset
	ABY,

	/// Accumulative
	ACC,

	/// Base page
	BPG,

	/// Base page with X offset
	BPX,

	/// Base page with Y offset
	BPY,

	/// Base page with Z offset
	BPZ,

	/// Immediate
	IMM,

	/// Implied
	IMP,

	/// Immediate word
	IMW,

	/// Indirect
	IND,

	/// Indirect with zero-page X offset
	IZX,

	/// Indirect with zero-page Y offset
	IZY,

	/// Relative
	REL,

	/// Word relative
	WRL,

	/// Zero page
	ZPG,

	/// Zero page with X offset
	ZPX,

	/// Zero page with Y offset
	ZPY,
}

impl Display for ExMode {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match &self {
			Self::ABS => write!(f, "ABS"),
			Self::ABW => write!(f, "ABS W"),
			Self::ABX => write!(f, "ABS X"),
			Self::ABY => write!(f, "ABS Y"),
			Self::ACC => write!(f, "ACC"),
			Self::BPG => write!(f, "BPG"),
			Self::BPX => write!(f, "BPG X"),
			Self::BPY => write!(f, "BPG Y"),
			Self::BPZ => write!(f, "BPG Z"),
			Self::IMM => write!(f, "IMM"),
			Self::IMP => write!(f, "IMP"),
			Self::IMW => write!(f, "IMM W"),
			Self::IND => write!(f, "IND"),
			Self::IZX => write!(f, "IND X"),
			Self::IZY => write!(f, "IND Y"),
			Self::REL => write!(f, "REL"),
			Self::WRL => write!(f, "W REL"),
			Self::ZPG => write!(f, "ZPG"),
			Self::ZPX => write!(f, "ZPG X"),
			Self::ZPY => write!(f, "ZPG Y"),
		}
	}
}

impl From<Mode> for ExMode {
	fn from(mode: Mode) -> Self {
		match mode {
			Mode::ABS => ExMode::ABS,
			Mode::ABX => ExMode::ABX,
			Mode::ABY => ExMode::ABY,
			Mode::IMM => ExMode::IMM,
			Mode::IMP => ExMode::IMP,
			Mode::IND => ExMode::IND,
			Mode::IZX => ExMode::IZX,
			Mode::IZY => ExMode::IZY,
			Mode::REL => ExMode::REL,
			Mode::ZPG => ExMode::ZPG,
			Mode::ZPX => ExMode::ZPX,
			Mode::ZPY => ExMode::ZPY,
		}
	}
}

bitflags! {
	/// 65CE02 state flags
	struct ExStatus: u8 {
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

		/// Extend stack
		const E = 32;

		/// Overflow
		const V = 64;

		/// Negative
		const N = 128;
	}
}

impl Display for ExStatus {
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

		if self.contains(Status::E) {
			write!(f, "E")?;
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

impl From<Status> for ExStatus {
	fn from(status: Status) -> Self {
		// Basically the same. The unused flag becomes the extend stack flag
		let mut status = status;
		status.set(Status::U, false);
		Self::from_bits_truncate(status.bits())
	}
}

/// CSG 65CE02 extended registers
#[derive(Clone, Copy, Debug)]
struct ExRegisters {
	/// base page
	b: u8,

	/// extended state flags
	p: ExStatus,

	/// general purpose
	z: u8,
}

/// CSG 65CE02 fetch data
#[derive(Clone, Copy, Debug)]
enum Data {
	Byte(u8),
	Word(u16),
}

/// CSG 65CE02 extended cache
#[derive(Clone, Copy, Debug)]
struct ExCache {
	mode: ExMode,
	data: Data,
}

impl Display for ExCache {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Last fetched byte: ${:X}", self.get_data())?;
		writeln!(f, "Last fetched opcode: ${:X}", self.get_opcode())?;
		writeln!(f, "Cycles remaining: {}", self.self.get_cycles())?;
		writeln!(f, "Last fetched absolute address: ${:X}", self.get_abs_addr())?;
		writeln!(f, "Last fetched relative address: {}", self.get_rel_addr() as i8)
	}
}

#[derive(Clone, Debug)]
pub struct CSG65CE02 {
	base: MOS6502,
	regs: ExRegisters,
	cache: ExCache,
}

impl CSG65CE02 {
	/// Initialises a new 65CE02, given a bus pointer
	pub fn new(bus: Rc<Bus>) -> MOS6502 {
		CSG65CE02 {
			base: MOS6502::new(bus),
			regs: Registers {
				b: 0,
				p: ExStatus::default(),
				z: 0,
			},
			cache: Cache {
				mode: ExMode::IMP,
			},
		}
	}

	/// Checks specified status flag(s)
	const fn check_flag(&self, flag: ExStatus) -> bool {
		self.regs.p.contains(flag)
	}

	/// Assign to accumulator, or write to bus, depending on the address mode
	fn check_mode(&mut self, value: u16) {
		if self.get_mode() == ExMode::IMP {
			self.base.set_a((value & 255) as u8);
		} else {
			self.base.write_last((value & 255) as u8);
		}
	}

	/// Fetch word from an operation
	fn fetchw(&mut self) -> u16 {
		u16::from_le_bytes([self.self.fetch16(), self.base.fetch16()])
	}

	/// Gets the base page register value as a 16-bit value
	pub const fn get_b16(&self) -> u16 {
		self.regs.b as u16
	}

	/// Gets the carry bit
	pub const fn get_carry(&self) -> u16 {
		(self.check_flag(ExStatus::C) as u16) & 1
	}

	/// Retrieves the currently cached address mode
	pub const fn get_mode(&self) -> ExMode {
		self.cache.mode
	}

	/// Retrieve the registry state flag bits
	pub const fn get_p_bits(&self) -> u8 {
		self.regs.p.bits()
	}

	/// Gets the Z register value
	pub const fn get_z(&self) -> u8 {
		self.regs.z
	}

	/// Gets the Z register value as 16-bit
	pub const fn get_z16(&self) -> u16 {
		self.get_z() as u16
	}

	/// Interrupts the execution state
	pub fn interrupt(&mut self, new_abs_addr: usize, new_cycles: u8) {
		// write the counter's current value to stack
		self.base.stack_write_addr(self.get_counter());

		// write state register to stack too
		self.set_flag(ExStatus::B, false);
		self.set_flag(ExStatus::I, true);
		self.base.stack_write(self.get_p_bits());

		// get the new pc value
		self.base.set_abs(new_abs_addr);
		let addr = self.base.fetch_addr();
		self.base.set_counter(addr);

		self.base.set_cycles(new_cycles);
	}

	/// Sets base page register value
	pub fn set_b(&mut self, value: u8) {
		self.regs.b = value;
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
		self.set_flag(ExStatus::Z, (value & 255) == 0)
	}

	/// Set the flag if the value is negative
	pub fn set_if_neg(&mut self, value: u16) {
		self.set_flag(Status::N, value & 128 != 0)
	}

	/// Set cached address mode. Only address mode functions should use this!
	pub fn set_mode(&mut self, mode: Mode) {
		self.cache.mode = mode;
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

	/// Sets Z register value
	pub fn set_z(&mut self, value: u8) {
		self.regs.z = value;
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
		self.set_b(0);
		self.set_flag(Status::default(), true);
		self.set_x(0);
		self.set_y(0);
		self.set_z(0);
		self.set_sp(STACK_INIT);

		self.set_abs(RES_ADDR);
		let addr = self.fetch_addr();
		self.set_counter(addr);

		self.cache.rel_addr = 0;
		self.set_abs(0);
		self.set_data(0);

		self.cache.cycles = 8;
	}

	// --- ADDRESS MODES

	/// Absolute address mode
	pub fn abs(&mut self) -> u8 {
		self.set_mode(ExMode::ABS);
		let addr = self.base.read_rom_addr();
		self.base.set_abs(addr);

		0
	}

	/// Absolute address mode with X register offset
	pub fn abx(&mut self) -> u8 {
		self.set_mode(ExMode::ABX);

		let addr = self.base.read_rom_addr();
		self.base.set_abs_addr(addr + self.base.get_x_zp_addr());

		self.base.check_page(addr)
	}

	/// Absolute address mode with Y register offset
	pub fn aby(&mut self) -> u8 {
		self.set_mode(ExMode::ABY);

		let addr = self.base.read_rom_addr();
		self.base.set_abs_addr(addr + self.base.get_y_zp_addr());

		self.check_page(addr)
	}

	/// Immediate address mode
	pub fn imm(&mut self) -> u8 {
		self.set_mode(ExMode::IMM);
		self.base.incr();
		self.base.set_abs_addr(self.base.get_counter());
		0
	}

	/// Implied address mode
	pub fn imp(&mut self) -> u8 {
		self.set_mode(ExMode::IMP);
		self.base.set_data(self.base.get_a());
		0
	}

	/// Indirect address mode (pointer access)
	pub fn ind(&mut self) -> u8 {
		self.set_mode(ExMode::IND);

		let ptr = self.base.read_rom_addr();

		if (ptr & 255) == 255 {
			// page boundary hardware bug
			self.base.set_abs_addr(self.base.read_addr(ptr));
		} else {
			// normal behavior
			self.base.set_abs_addr(self.base.read_addr(ptr));
		}

		0
	}

	/// Indirect address mode of zero-page with X register offset
	pub fn izx(&mut self) -> u8 {
		self.set_mode(ExMode::IZX);

		let t = self.base.read_rom_zp_addr();
		let lo = self.base.get_zp_addr((t + self.base.get_x_zp_addr()) & 255);
		let hi = self.base.get_zp_addr((t + self.base.get_x_zp_addr() + 1) & 255);

		self.base.set_abs((hi << 8) | lo);
		0
	}

	/// Indirect address mode of zero-page with Y register offset
	pub fn izy(&mut self) -> u8 {
		self.set_mode(ExMode::IZY);

		let t = self.base.read_rom_zp_addr();
		let lo = self.base.get_zp_addr(t & 255);
		let hi = self.base.get_zp_addr((t + 1) & 255);

		self.base.set_abs_addr(((hi << 8) | lo) + self.base.get_y_zp_addr());

		if self.base.get_abs_hi() != (hi << 8) { 1 } else { 0 }
	}

	/// Relative address mode (branching instructions)
	pub fn rel(&mut self) -> u8 {
		self.set_mode(ExMode::REL);
		self.base.set_rel_addr(self.base.read_rom_zp_addr());

		// check_flag for signed bit
		if self.base.get_rel_addr() & 128 != 0 {
			self.set_rel_addr(self.base.get_rel_addr() | 0xFF00);
		}

		0
	}

	/// Zero-page address mode
	pub fn zpg(&mut self) -> u8 {
		self.base.set_mode(ExMode::ZPG);
		let addr = self.base.read_rom_zp_addr();
		self.base.set_abs_addr(addr);
		self.base.incr();
		self.base.set_abs_addr(self.base.get_abs_addr() & 255);
		0
	}

	/// Zero-page address mode with X register offset
	pub fn zpx(&mut self) -> u8 {
		self.set_mode(ExMode::ZPX);
		let addr = self.base.read_rom_zp_addr();
		self.set_abs_addr(addr + self.base.get_x_zp_addr());
		self.base.set_abs_addr(self.base.get_abs_addr() & 255);
		0
	}

	/// Zero-page address mode with Y register offset
	pub fn zpy(&mut self) -> u8 {
		self.set_mode(Mode::ZPY);
		let addr = self.read_rom_zp_addr();
		self.set_abs(addr + self.get_y_zp_addr());
		self.base.set_abs_addr(self.base.get_abs_addr() & 255);
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

	/// Arithmetical left shift (word)
	pub fn asw(&mut self) -> u8 {
		let tmp = self.fetchw() << 1;
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

	/// Branch always
	pub fn bra(&mut self) -> u8 {
		self.branch();
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

	/// Jump to address
	pub fn bru(&mut self) -> u8 {
		self.base.jmp()
	}

	/// Branch to subroutine
	pub fn bsr(&mut self) -> u8 {
		self.stack_write_addr(self.get_counter());
		self.branch();
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

	/// Clear stack extend bit
	pub fn cle(&mut self) -> u8 {
		self.set_flag(Status::E, false);
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

	/// Compare with Z
	pub fn cpz(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_flag(Status::C, self.get_z() >= fetch);
		self.set_flags_nz(self.get_z16() - self.get_data16());

		1
	}

	/// Decrement accumulator
	pub fn dex(&mut self) -> u8 {
		self.regs.a -= 1;
		self.set_flags_nz(self.get_a16());

		0
	}

	/// Decrement value
	pub fn dec(&mut self) -> u8 {
		let tmp = self.fetch16() - 1;
		self.write_last(tmp);
		self.set_flags_nz(tmp);

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

	/// Decrement Z register
	pub fn dez(&mut self) -> u8 {
		self.regs.z -= 1;
		self.set_flags_nz(self.get_z16());

		0
	}

	/// Exclusive or
	pub fn eor(&mut self) -> u8 {
		self.regs.a ^= self.fetch();
		self.set_flags_nz(self.get_a16());

		1
	}

	/// Increment accumulator register
	pub fn ina(&mut self) -> u8 {
		self.regs.a += 1;
		self.set_flags_nz(self.get_a16());

		0
	}

	/// Increment value
	pub fn inc(&mut self) -> u8 {
		let tmp = self.fetch16() + 1;
		self.base.write_last(tmp);
		self.set_flags_nz(tmp);

		0
	}

	/// Increment X register
	pub fn inx(&mut self) -> u8 {
		self.base.set_x(self.base.get_x() + 1);
		self.set_flags_nz(self.base.get_x16());

		0
	}

	/// Increment Y register
	pub fn iny(&mut self) -> u8 {
		self.base.set_y(self.base.get_y() + 1);
		self.set_flags_nz(self.base.get_y16());

		0
	}

	/// Increment Z register
	pub fn inz(&mut self) -> u8 {
		self.regs.z += 1;
		self.set_flags_nz(self.get_z16());

		0
	}

	/// Load into accumulator
	pub fn lda(&mut self) -> u8 {
		let b = self.fetch();
		self.base.set_a(fetch);
		self.set_flags_nz(self.base.get_a16());
		1
	}

	/// Load into X
	pub fn ldx(&mut self) -> u8 {
		let b = self.base.fetch();
		self.base.set_x(b);
		self.set_flags_nz(self.base.get_x16());
		1
	}

	/// Load into Y
	pub fn ldy(&mut self) -> u8 {
		let b = self.base.fetch();
		self.base.set_y(b);
		self.set_flags_nz(self.base.get_y16());
		1
	}

	/// Load into Z
	pub fn ldz(&mut self) -> u8 {
		let b = self.base.fetch();
		self.set_z(b);
		self.set_flags_nz(self.get_z16());
		1
	}

	/// Logical right shift
	pub fn lsr(&mut self) -> u8 {
		let tmp = self.fetch16() >> 1;
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Two's compliment negation. The following byte should be the accumulator.
	pub fn neg(&mut self) -> u8 {
		let tmp = !self.fetch16() + 1;
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
		let b = self.base.fetch();
		self.base.set_a(b);
		self.set_flags_nz(self.base.get_a16());

		1
	}

	/// Push accumulator register to the stack
	pub fn pha(&mut self) -> u8 {
		self.stack_write(self.base.get_a());
		0
	}

	/// Push state register to the stack
	pub fn php(&mut self) -> u8 {
		self.set_flag(ExStatus::B, true);
		self.base.stack_write(self.get_p_bits());
		self.set_flag(ExStatus::B, false);

		0
	}

	/// Push Z register to the stack
	pub fn phz(&mut self) -> u8 {
		self.base.stack_write(self.get_z());
		0
	}

	/// Pop accumulator register from the stack
	pub fn pla(&mut self) -> u8 {
		let b = self.base.stack_read();
		self.base.set_a(b);
		self.set_flags_nz(self.base.get_a16());

		0
	}

	/// Pop state register from the stack
	pub fn plp(&mut self) -> u8 {
		self.regs.p = ExStatus::from_bits_truncate(self.base.stack_read());
		0
	}

	/// Pop Z register from the stack
	pub fn plz(&mut self) -> u8 {
		let b = self.base.stack_read();
		self.set_z(b);
		self.set_flags_nz(self.get_z16());

		0
	}

	/// Bit rotate left
	pub fn rol(&mut self) -> u8 {
		let tmp = self.base.fetch16().rotate_left(1);
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Bit rotate right
	pub fn ror(&mut self) -> u8 {
		let tmp = self.base.fetch16().rotate_right(1);
		self.set_flag(ExStatus::C, (self.base.get_data() & 1) != 0);
		self.set_flags_nz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Bit rotate left (word)
	pub fn row(&mut self) -> u8 {
		let tmp = self.fetchw().rotate_left(1);
		self.set_flags_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Restores state from interrupt
	pub fn rti(&mut self) -> u8 {
		// restore state flags
		self.regs.p = ExStatus::from_bits_truncate(self.stack_read());
		self.regs.p.toggle(ExStatus::B);

		// and counter
		let addr = self.stack_read_addr();
		self.base.set_counter(addr);

		0
	}

	/// Subtraction with carry
	pub fn sbc(&mut self) -> u8 {
		let value = self.base.fetch16() ^ 255; // invert the value
		let tmp = self.base.get_a16() + value + self.get_carry();

		self.set_flags_cnz(tmp);
		self.set_flag(ExStatus::V, (tmp ^ self.base.get_a16() & (tmp ^ value)) & 128 != 0);
		self.base.set_a((tmp & 255) as u8);

		1
	}

	/// Set carry bit
	pub fn sec(&mut self) -> u8 {
		self.set_flag(ExStatus::C, true);
		0
	}

	/// Set decimal bit
	pub fn sed(&mut self) -> u8 {
		self.set_flag(ExStatus::D, true);
		0
	}

	/// Set stack extend bit
	pub fn see(&mut self) -> u8 {
		self.set_flag(ExStatus::D, true);
		0
	}

	/// Set interrupt disable bit
	pub fn sei(&mut self) -> u8 {
		self.set_flag(ExStatus::I, true);
		0
	}

	/// Store Z at address
	pub fn stz(&mut self) -> u8 {
		self.write_last(self.get_z());
		0
	}

	/// Transfer accumulator to base page
	pub fn tab(&mut self) -> u8 {
		self.set_b(self.base.get_a());
		self.set_flags_nz(self.get_b16());
		0
	}

	/// Transfer accumulator to X
	pub fn tax(&mut self) -> u8 {
		self.base.set_x(self.base.get_a());
		self.set_flags_nz(self.base.get_x16());
		0
	}

	/// Transfer accumulator to Y
	pub fn tay(&mut self) -> u8 {
		self.base.set_y(self.base.get_a());
		self.set_flags_nz(self.base.get_y16());
		0
	}

	/// Transfer accumulator to Z
	pub fn taz(&mut self) -> u8 {
		self.set_z(self.get_a());
		self.set_flags_nz(self.get_z16());
		0
	}

	/// Transfer stack pointer to base page
	pub fn tsb(&mut self) -> u8 {
		self.set_b(self.base.get_sp() as u8);
		self.set_flags_nz(self.get_b16());
		0
	}

	/// Transfer stack pointer to X
	pub fn tsx(&mut self) -> u8 {
		self.base.set_x(self.base.get_sp() as u8);
		self.set_flags_nz(self.base.get_x16());
		0
	}

	/// Transfer X to accumulator
	pub fn txa(&mut self) -> u8 {
		self.base.set_a(self.base.get_x());
		self.set_flags_nz(self.base.get_a16());
		0
	}

	/// Transfer Y to accumulator
	pub fn tya(&mut self) -> u8 {
		self.base.set_a(self.base.get_y());
		self.set_flags_nz(self.base.get_a16());
		0
	}

	/// Transfer Z to accumulator
	pub fn tza(&mut self) -> u8 {
		self.base.set_a(self.base.get_z());
		self.set_flags_nz(self.base.get_a16());
		0
	}
}

impl DeviceMap for CSG65CE02 {
	fn add_region(&mut self, address: usize, region: Region) {
		self.base.add_region(address, region);
	}

	fn generate_regions(&mut self, start: usize, end: usize) {
		let mut offset = start;

		while offset < end {
			let op = self.get_u8(offset) as usize;
			offset += 1;

			match op {
				// relative
				16 | 48 | 80 | 112 | 144 | 176 | 208 | 240 => {
					let addr = ((offset as i32) + (self.bus.get_i8(offset) as i32) + 1) as u16;
					self.add_region(addr as usize, Region::new(0, RegionType::Label,
						format!("L_{:04X}", addr).as_str()));

					offset += 1;
				},
				32 => { // JSR, label is a function
					let addr = self.get_u16_le(offset);
					self.add_region(addr as usize,
						Region::new(0, RegionType::Function, format!("F_{:04X}", addr).as_str()));
					offset += 2;
				},
				76 | 108 => { // JMP absolute or indirect
					let addr = self.get_u16_le(offset);
					self.add_region(addr as usize,
						Region::new(0, RegionType::Label, format!("L_{:04X}", addr).as_str()));
					offset += 2;
				},
				// implied
				_ => todo!(),
			}
		}
	}
}

impl DeviceBase for CSG65CE02 {
	fn read(&self, address: usize, length: usize) -> Vec<u8> {
		self.base.read(address, length)
	}

	fn write(&mut self, address: usize, data: &[u8]) {
		self.base.write(address, data);
	}
}

impl Device for CSG65CE02 {
	fn get_bus(&self) -> Rc<Bus> {
		self.base.get_bus()
	}

	fn get_region(&self, offset: usize) -> Option<&Region> {
		self.base.get_region(offset)
	}

	fn get_region_mut(&mut self, offset: usize) -> Option<&mut Region> {
		self.base.get_region_mut(offset)
	}
}

impl Processor for CSG65CE02 {
	fn clock(&mut self) {
		if self.get_cycles() == 0 {
			// get and increment the counter
			self.cache.opcode = self.get_u8(self.get_counter());
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
					let op_cycles = self.cle();
					self.add_cycles(2 + mode_cycles & op_cycles);
				},
				3 => {
					let mode_cycles = self.imp();
					let op_cycles = self.see();
					self.add_cycles(2 + (mode_cycles & op_cycles));
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
		}

		self.cache.cycles -= 1;
	}

	fn get_ptr_size(&self) -> usize {
		2
	}
}
