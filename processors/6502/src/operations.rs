/// Absolute address mode
pub fn abs(&mut self) -> u8 {
	let lo = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	let hi = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	self.cache.abs_addr = (hi << 8) | lo;
	0
}

/// Absolute address mode with X register offset
pub fn abx(&mut self) -> u8 {
	let lo = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	let hi = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	self.cache.abs_addr = ((hi << 8) | lo) + self.registers.x;

	if (self.cache.abs_addr & 0xFF00) != (hi << 8) {
		1
	} else {
		0
	}
}

/// Absolute address mode with Y register offset
pub fn aby(&mut self) -> u8 {
	let lo = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	let hi = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	self.cache.abs_addr = ((hi << 8) | lo) + self.registers.y;

	if (self.cache.abs_addr & 0xFF00) != (hi << 8) {
		1
	} else {
		0
	}
}

/// Addition with carry
pub fn adc(&mut self) -> u8 {
	let _ = self.fetch();
	let carry = if self.registers.status.contains(Status::CARRY) { 1 } else { 0 };
	let temp16 = (self.registers.accumulator as u16) + (self.cache.data as u16) + carry;

	self.registers.status.set(Status::CARRY, temp16 > 255);
	self.registers.status.set(Status::ZERO, temp16 & 255 == 0);
	self.registers.status.set(Status::NEGATIVE, temp16 & 128 != 0);

	self.registers.status.set(Status::OVERFLOW,
		!(((self.registers.accumulator as u16) ^
			(self.cache.data as u16)) &
				((self.registers.accumulator as u16) ^ temp16)) & 128);

	self.registers.accumulator = (temp16 & 255) as u8;

	1
}

/// Bitwise and
pub fn and(&mut self) -> u8 {
	let _ = self.fetch();
	self.registers.accumulator &= self.cache.data;
	self.registers.status.set(Status::ZERO, self.registers.accumulator == 0);
	self.registers.status.set(Status::NEGATIVE, self.registers.accumulator & 128 != 0);

	1
}

/// Branching if carry clear
pub fn bcc(&mut self) -> u8 {
	if !self.registers.status.contains(Status::CARRY) {
		self.branch();
	}

	0
}

/// Branching if carry
pub fn bcs(&mut self) -> u8 {
	if self.registers.status.contains(Status::CARRY) {
		self.branch();
	}

	0
}

/// Branching if carry
pub fn beq(&mut self) -> u8 {
	if self.registers.status.contains(Status::ZERO) {
		self.branch();
	}

	0
}

/// Branching if negative
pub fn bmi(&mut self) -> u8 {
	if self.registers.status.contains(Status::NEGATIVE) {
		self.branch();
	}

	0
}

/// Branching if not equal
pub fn bne(&mut self) -> u8 {
	if !self.registers.status.contains(Status::ZERO) {
		self.branch();
	}

	0
}

/// Branching if positive
pub fn bpl(&mut self) -> u8 {
	if !self.registers.status.contains(Status::NEGATIVE) {
		self.branch();
	}

	0
}

/// Branching if overflow
pub fn bvc(&mut self) -> u8 {
	if self.registers.status.contains(Status::OVERFLOW) {
		self.branch();
	}

	0
}

/// Branching if not overflow
pub fn bvs(&mut self) -> u8 {
	if !self.registers.status.contains(Status::OVERFLOW) {
		self.branch();
	}

	0
}

/// Clear carry status bit
pub fn clc(&mut self) -> u8 {
	self.registers.status.set(Status::CARRY, false);
	0
}

/// Clear decimal status bit
pub fn cld(&mut self) -> u8 {
	self.registers.status.set(Status::DECIMAL, false);
	0
}

/// Clear interrupt disable status bit
pub fn cli(&mut self) -> u8 {
	self.registers.status.set(Status::NO_INTERRUPTS, false);
	0
}

/// Clear overflow status bit
pub fn clv(&mut self) -> u8 {
	self.registers.status.set(Status::OVERFLOW, false);
	0
}

/// Immediate address mode
pub fn imm(&mut self) -> u8 {
	self.registers.counter += 1;
	self.cache.abs_addr = self.registers.counter;
	0
}

/// Implied address mode
pub fn imp(&mut self) -> u8 {
	self.cache.data = self.registers.accumulator;
	0
}

/// Indirect address mode (pointer access)
pub fn ind(&mut self) -> u8 {
	let lo = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	let hi = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	let ptr = (hi << 8) | lo;

	if lo == 255 {
		// page boundary hardware bug
		self.cache.abs_addr = (self.read(ptr & 0xFF00) << 8) | self.read(ptr);
	} else {
		// normal behavior
		self.cache.abs_addr = (self.read(ptr + 1) << 8) | self.read(ptr);
	}

	0
}

/// Interrupt request
pub fn irq(&mut self) {
	if !self.registers.status.contains(Status::NO_INTERRUPTS) {
		self.interrupt(IRQ_ADDR, 7);
	}
}

/// Indirect address mode of zero-page with X register offset
pub fn izx(&mut self) -> u8 {
	let t = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	let lo = self.read((t + (self.registers.x as u16)) & 255);
	let hi = self.read((t + (self.registers.x as u16) + 1) & 255);

	self.cache.abs_addr = (hi << 8) | lo;
	0
}

/// Indirect address mode of zero-page with Y register offset
pub fn izy(&mut self) -> u8 {
	let t = self.read(self.registers.counter) as u16;
	self.registers.counter += 1;

	let lo = self.read(t & 255);
	let hi = self.read((t + 1) & 255);

	self.cache.abs_addr = ((hi << 8) | lo) + (self.registers.y as u16);

	if (self.cache.abs_addr & 0xFF00) != (hi << 8) {
		1
	} else {
		0
	}
}

/// Non-maskable interrupt
pub fn nmi(&mut self) {
	self.interrupt(NMI_ADDR, 8);
}

/// Push accumulator to the stack
pub fn pha(&mut self) -> u8 {
	self.stack_write(self.registers.accumulator);
	0
}

/// Pop accumulator from the stack
pub fn pla(&mut self) -> u8 {
	self.registers.accumulator = self.stack_read();
	self.registers.status.set(Status::ZERO, self.registers.accumulator == 0);
	self.registers.status.set(Status::NEGATIVE, self.registers.accumulator & 128 != 0);

	0
}

/// Relative address mode (branching instructions)
pub fn rel(&mut self) -> u8 {
	self.cache.rel_addr = self.read(self.registers.counter);
	self.registers.counter += 1;

	// check for signed bit
	if self.cache.rel_addr & 128 != 0 {
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
	self.registers.counter = self.stack_read() as u16;
	self.registers.counter |= (self.stack_read() as u16) << 8;

	0
}

/// Subtraction with carry
pub fn sdc(&mut self) -> u8 {
	let _ = self.fetch();
	let value = (self.cache.data as u16) ^ 255; // invert the value
	let carry = if self.registers.status.contains(Status::CARRY) { 1 } else { 0 };
	let temp16 = (self.registers.accumulator as u16) + value + carry;

	self.registers.status.set(Status::CARRY, temp16 & 0xFF00 != 0);
	self.registers.status.set(Status::ZERO, temp16 & 255 == 0);
	self.registers.status.set(Status::OVERFLOW,
		(temp16 ^ (self.registers.accumulator as u16)) & (temp16 ^ value) & 128);
	self.registers.status.set(Status::NEGATIVE, temp16 & 128 != 0);

	self.registers.accumulator = (temp16 & 255) as u8;

	1
}

/// Zero-page address mode
pub fn zp0(&mut self) -> u8 {
	self.cache.abs_addr = self.read(self.registers.counter);
	self.registers.counter += 1;
	self.cache.abs_addr &= 255;
	0
}

/// Zero-page address mode with X register offset
pub fn zpx(&mut self) -> u8 {
	self.cache.abs_addr = self.read(self.registers.counter) + (self.registers.x as u16);
	self.registers.counter += 1;
	self.cache.abs_addr &= 255;
	0
}

/// Zero-page address mode with Y register offset
pub fn zpy(&mut self) -> u8 {
	self.cache.abs_addr = self.read(self.registers.counter) + (self.registers.y as u16);
	self.registers.counter += 1;
	self.cache.abs_addr &= 255;
	0
}
