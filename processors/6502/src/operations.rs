/// Absolute address mode
pub fn abs(&mut self) -> u8 {
	self.set_abs(self.read_rom_addr());
	0
}

/// Absolute address mode with X register offset
pub fn abx(&mut self) -> u8 {
	self.set_abs(self.read_rom_addr() + self.get_x());

	if self.abs_hi() != (hi << 8) {
		1
	} else {
		0
	}
}

/// Absolute address mode with Y register offset
pub fn aby(&mut self) -> u8 {
	self.set_abs(self.read_rom_addr() + self.get_y());

	if self.abs_hi() != (hi << 8) {
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

	self.set_pc(self.read_addr());
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
	self.set_abs(self.counter();
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

	if lo == 255 {
		// page boundary hardware bug
		self.set_abs((self.read(ptr & 0xFF00) << 8) | self.read(ptr));
	} else {
		// normal behavior
		self.set_abs((self.read(ptr + 1) << 8) | self.read(ptr));
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
	self.set_abs(self.read(self.counter());
	self.incr();
	self.cache.abs_addr &= 255;
	0
}

/// Zero-page address mode with X register offset
pub fn zpx(&mut self) -> u8 {
	self.set_abs(self.read(self.counter()) + (self.get_x() as u16);
	self.incr();
	self.cache.abs_addr &= 255;
	0
}

/// Zero-page address mode with Y register offset
pub fn zpy(&mut self) -> u8 {
	self.set_abs(self.read(self.counter()) + (self.get_y() as u16);
	self.incr();
	self.cache.abs_addr &= 255;
	0
}
