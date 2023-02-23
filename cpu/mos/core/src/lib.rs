#[cfg(feature = "assembler")]
pub mod asm;

#[cfg(feature = "disassembler")]
pub mod disasm6502;

#[cfg(feature = "mos6502")]
pub mod mos6502;

#[cfg(feature = "csg65ce02")]
pub mod csg65ce02;

#[cfg(feature = "assembler")]
pub use asm::*;

#[cfg(feature = "disassembler")]
pub use disasm6502::*;

#[cfg(feature = "mos6502")]
pub use mos6502::*;

use rgk_processors_core::{
	DeviceBase,
	Processor
};

/// Offset of interrupt request vector
pub const IRQ_ADDR: usize = 65534;

/// Offset of non-maskable interrupt vector
pub const NMI_ADDR: usize = 65530;

/// Offset of reset vector
pub const RES_ADDR: usize = 65532;

/// 6502 helper functions
pub trait Helper6502: DeviceBase + Processor {
	/// Add additional cycles to the current operation
	fn add_cycles(&mut self, value: u8);

	/// Checks the given value and do something depending on the address mode
	fn check_mode(&mut self, value: u16);

	/// Fetch byte from an operation
	fn fetch(&mut self) -> u8;

	/// Gets the zero flag
	fn get_0(&self) -> bool;

	/// Gets the accumulator register value
	fn get_a(&self) -> u8;

	/// Gets cached absolute address
	fn get_abs_addr(&self) -> usize;

	/// Gets the carry flag
	fn get_carry(&self) -> bool;

	/// Gets the carry bit
	fn get_carry_bit(&self) -> u16;

	/// Gets the program counter register value
	fn get_counter(&self) -> usize;

	/// Gets the remaining cycle count
	fn get_cycles(&self) -> u8;

	/// Gets the currently cached data byte
	fn get_data(&self) -> u8;

	/// Gets the negative flag
	fn get_neg(&self) -> bool;

	/// Gets the currently cached opcode
	fn get_opcode(&self) -> usize;

	/// Gets the overflow flag
	fn get_overflow(&self) -> bool;

	/// Retrieve the registry state flag bits
	fn get_p_bits(&self) -> u8;

	/// Gets cached relative address
	fn get_rel_addr(&self) -> usize;

	/// Gets the stack pointer register value
	fn get_sp(&self) -> usize;

	/// Gets the X register value
	fn get_x(&self) -> u8;

	/// Gets the Y register value
	fn get_y(&self) -> u8;

	/// Interrupts the execution state
	fn interrupt(&mut self, new_abs_addr: usize, new_cycles: u8);

	/// Set the flag if the value is zero
	fn set_0_if(&mut self, value: u16);

	/// Sets accumulator register value
	fn set_a(&mut self, value: u8);

	/// Sets cached absolute address
	fn set_abs_addr(&mut self, value: usize);

	/// Sets the break flag
	fn set_brk(&mut self, condition: bool);

	/// Sets the carry flag
	fn set_carry_if(&mut self, condition: bool);

	/// Sets program counter register value
	fn set_counter(&mut self, value: usize);

	/// Sets cycle count
	fn set_cycles(&mut self, value: u8);

	/// Sets cached data
	fn set_data(&mut self, value: u8);

	/// Set the flag if the value is negative
	fn set_neg_if(&mut self, value: u16);

	/// Set the flag if the condition overflows
	fn set_overflow_if(&mut self, condition: bool);

	/// Sets the interrupt flag
	fn set_int(&mut self, condition: bool);

	/// Sets the relative address
	fn set_rel_addr(&mut self, value: usize);

	/// Sets stack pointer
	fn set_sp(&mut self, value: usize);

	/// Sets X register value
	fn set_x(&mut self, value: u8);

	/// Sets Y register value
	fn set_y(&mut self, value: u8);

	/// Read from stack
	fn stack_read(&mut self) -> u8;

	/// Write to stack
	fn stack_write(&mut self, data: u8);

	/// Returns a hexdump string of the stackdump
	fn stackdump(&self) -> String;

	/// Branch execution
	fn branch(&mut self) {
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

	/// Check for page change
	fn check_page(&self, addr: usize) -> u8 {
		if self.get_abs_hi() != (addr & 0xFF00) {
			1
		} else {
			0
		}
	}

	/// Fetch byte from an operation as 16-bit
	fn fetch16(&mut self) -> u16 {
		self.fetch().into()
	}

	/// Fetch address
	fn fetch_ptr(&mut self) -> usize {
		u16::from_le_bytes([self.fetch(), self.fetch()]).into()
	}

	/// Gets the accumulator register value as a 16-bit value
	fn get_a16(&self) -> u16 {
		self.get_a().into()
	}

	/// Gets cached absolute address' high byte
	fn get_abs_hi(&self) -> usize {
		self.get_abs_addr() & 0xFF00
	}

	/// Gets the currently cached data byte as 16-bit
	fn get_data16(&self) -> u16 {
		self.get_data().into()
	}

	/// Gets the X register value as 16-bit
	fn get_x16(&self) -> u16 {
		self.get_x().into()
	}

	/// Gets X register value as a zero page address
	fn get_x_zp_addr(&self) -> usize {
		self.get_x().into()
	}

	/// Gets the Y register value as 16-bit
	fn get_y16(&self) -> u16 {
		self.get_y().into()
	}

	/// Gets X register value as a zero page address
	fn get_y_zp_addr(&self) -> usize {
		self.get_y().into()
	}

	/// Get zero-page address
	fn get_zp_addr(&self, address: usize) -> usize {
		self.get_u8(address).into()
	}

	/// Increment program counter registry by 1
	fn incr(&mut self) {
		self.set_counter(self.get_counter() + 1);
	}

	/// Reads a byte from the ROM
	fn read_rom(&mut self) -> u8 {
		let data = self.get_u8(self.get_counter());
		self.incr();

		data
	}

	/// Reads an address from the ROM
	fn read_rom_addr(&mut self) -> usize {
		u16::from_le_bytes([self.read_rom(), self.read_rom()]).into()
	}

	/// Reads an 8-bit address from the ROM
	fn read_rom_zp_addr(&mut self) -> usize {
		self.read_rom().into()
	}

	/// Set carry, negative, and/or zero bits of state flags register, given a value
	fn set_cnz(&mut self, value: u16) {
		self.set_carry_if(value > 255);
		self.set_nz(value);
	}

	/// Set negative and/or zero bits of state flags register, given a value
	fn set_nz(&mut self, value: u16) {
		self.set_0_if(value);
		self.set_neg_if(value);
	}

	/// Reads an address from stack
	fn stack_get_ptr(&mut self) -> usize {
		u16::from_le_bytes([self.stack_read(), self.stack_read()]).into()
	}

	/// Writes an address to stack
	fn stack_write_ptr(&mut self, addr: usize) {
		self.stack_write(((addr & 0xFF00) >> 8) as u8);
		self.stack_write((addr & 255) as u8);
	}

	/// Writes to the last absolute address
	fn write_last(&mut self, data: u8) {
		self.write(self.get_abs_addr(), &[data]);
	}
}

/// 6502 instruction set architecture
pub trait ISA6502: Helper6502 {
	/// Interrupt request
	fn irq(&mut self);

	/// Absolute address mode
	fn abs(&mut self) -> u8;

	/// Absolute address mode with X offset
	fn abx(&mut self) -> u8;

	/// Absolute address mode with Y offset
	fn aby(&mut self) -> u8;

	/// Immediate address mode
	fn imm(&mut self) -> u8;

	/// Implied address mode
	fn imp(&mut self) -> u8;

	/// Indirect address mode (pointer access)
	fn ind(&mut self) -> u8;

	/// Indirect address mode with X offset
	fn izx(&mut self) -> u8;

	/// Indirect address mode with Y offset
	fn izy(&mut self) -> u8;

	/// Relative address mode (branching)
	fn rel(&mut self) -> u8;

	/// Zero page address mode
	fn zpg(&mut self) -> u8;

	/// Zero page address mode with X offset
	fn zpx(&mut self) -> u8;

	/// Zero page address mode with Y offset
	fn zpy(&mut self) -> u8;

	/// Clear decimal
	fn cld(&mut self) -> u8;

	/// No operation
	fn nop(&self) -> u8;

	/// Push processor state to stack
	fn php(&mut self) -> u8;

	/// Pop processor state from stack
	fn plp(&mut self) -> u8;

	/// Return from interrupt
	fn rti(&mut self) -> u8;

	/// Return from subroutine
	fn rts(&mut self) -> u8;

	/// Set decimal
	fn sed(&mut self) -> u8;

	/// Addition with carry
	fn adc(&mut self) -> u8 {
		let fetch = self.fetch16();
		let tmp = self.get_a16() + (fetch + self.get_carry_bit());

		self.set_cnz(tmp);

		self.set_overflow_if(!(((self.get_a16() ^ self.get_data16()) &
			(self.get_a16() ^ tmp)) & 128) == 0);

		self.set_a((tmp & 255) as u8);

		1
	}

	/// Bitwise AND
	fn and(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_a(self.get_a() & fetch);
		self.set_nz(self.get_a16());

		1
	}

	/// Arithmetical left shift
	fn asl(&mut self) -> u8 {
		let tmp = self.fetch16() << 1;
		self.set_cnz(tmp);
		self.check_mode(tmp);

		0
	}

	/// Branch if carry clear
	fn bcc(&mut self) -> u8 {
		if !self.get_carry() {
			self.branch();
		}
		0
	}

	/// Branch if carry set
	fn bcs(&mut self) -> u8 {
		if self.get_carry() {
			self.branch();
		}
		0
	}

	/// Branch if equal (zero)
	fn beq(&mut self) -> u8 {
		if self.get_0() {
			self.branch();
		}
		0
	}

	/// Bit test
	fn bit(&mut self) -> u8 {
		let fetch = self.fetch16();
		self.set_0_if(self.get_a16() & fetch);
		self.set_neg_if(self.get_data16());
		self.set_overflow_if(self.get_data() & 64 != 0);

		0
	}

	/// Branch if negative
	fn bmi(&mut self) -> u8 {
		if self.get_neg() {
			self.branch();
		}
		0
	}

	/// Branch if not equal (non-zero)
	fn bne(&mut self) -> u8 {
		if !self.get_0() {
			self.branch();
		}
		0
	}

	/// Branch if positive
	fn bpl(&mut self) -> u8 {
		if !self.get_neg() {
			self.branch();
		}
		0
	}

	/// Program-sourced interrupt
	fn brk(&mut self) -> u8 {
		// This differs slightly from self.interrupt()

		self.incr();
		self.set_int(true);
		self.stack_write_ptr(self.get_counter());
		self.set_brk(true);
		self.stack_write(self.get_p_bits());
		self.set_brk(false);
		self.set_counter(self.get_ptr(IRQ_ADDR));

		0
	}

	/// Branch if not overflow
	fn bvc(&mut self) -> u8 {
		if !self.get_overflow() {
			self.branch();
		}
		0
	}

	/// Branch if overflow
	fn bvs(&mut self) -> u8 {
		if self.get_overflow() {
			self.branch();
		}
		0
	}

	/// Clear carry
	fn clc(&mut self) -> u8 {
		self.set_carry_if(false);
		0
	}

	/// Clear interrupt disable
	fn cli(&mut self) -> u8 {
		self.set_int(false);
		0
	}

	/// Clear overflow
	fn clv(&mut self) -> u8 {
		self.set_overflow_if(false);
		0
	}

	/// Compare with accumulator
	fn cmp(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_carry_if(self.get_a() >= fetch);
		self.set_nz(self.get_a16() - self.get_data16());
		1
	}

	/// Compare with X
	fn cpx(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_carry_if(self.get_x() >= fetch);
		self.set_nz(self.get_x16() - self.get_data16());
		1
	}

	/// Compare with Y
	fn cpy(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_carry_if(self.get_y() >= fetch);
		self.set_nz(self.get_y16() - self.get_data16());
		1
	}

	/// Decrement
	fn dec(&mut self) -> u8 {
		let fetch = self.fetch() - 1;
		self.write_last(fetch);
		self.set_nz(fetch as u16);
		0
	}

	/// Decrement X
	fn dex(&mut self) -> u8 {
		self.set_x(self.get_x() - 1);
		self.set_nz(self.get_x16());
		0
	}

	/// Decrement Y
	fn dey(&mut self) -> u8 {
		self.set_y(self.get_y() - 1);
		self.set_nz(self.get_y16());
		0
	}

	/// Exclusive OR
	fn eor(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_a(self.get_a() ^ fetch);
		self.set_nz(self.get_a16());
		1
	}

	/// Increment
	fn inc(&mut self) -> u8 {
		let fetch = self.fetch() + 1;
		self.write_last(fetch);
		self.set_nz(fetch as u16);
		0
	}

	/// Increment X
	fn inx(&mut self) -> u8 {
		self.set_x(self.get_x() + 1);
		self.set_nz(self.get_x16());
		0
	}

	/// Increment Y
	fn iny(&mut self) -> u8 {
		self.set_y(self.get_y() + 1);
		self.set_nz(self.get_y16());
		0
	}

	/// Jump to address
	fn jmp(&mut self) -> u8 {
		self.set_counter(self.get_abs_addr());
		0
	}

	/// Jump to subroutine
	fn jsr(&mut self) -> u8 {
		self.stack_write_ptr(self.get_counter());
		self.set_counter(self.get_abs_addr());
		0
	}

	/// Load into accumulator
	fn lda(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_a(fetch);
		self.set_nz(self.get_a16());
		1
	}

	/// Load into X
	fn ldx(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_x(fetch);
		self.set_nz(self.get_x16());
		1
	}

	/// Load into Y
	fn ldy(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_y(fetch);
		self.set_nz(self.get_y16());
		1
	}

	/// Logical right shift
	fn lsr(&mut self) -> u8 {
		let tmp = self.fetch16() >> 1;
		self.set_cnz(tmp);
		self.check_mode(tmp);
		0
	}

	/// Non-maskable interrupt
	fn nmi(&mut self) {
		self.interrupt(NMI_ADDR, 8);
	}

	/// Bitwise OR
	fn ora(&mut self) -> u8 {
		let fetch = self.fetch();
		self.set_a(self.get_a() | fetch);
		self.set_nz(self.get_a16());
		1
	}

	/// Push accumulator to stack
	fn pha(&mut self) -> u8 {
		self.stack_write(self.get_a());
		0
	}

	/// Pop accumulator from stack
	fn pla(&mut self) -> u8 {
		let b = self.stack_read();
		self.set_a(b);
		self.set_nz(self.get_a16());
		0
	}

	/// Bit rotate left
	fn rol(&mut self) -> u8 {
		let tmp = self.fetch16().rotate_left(1);
		self.set_cnz(tmp);
		self.check_mode(tmp);
		0
	}

	/// Bit rotate right
	fn ror(&mut self) -> u8 {
		let tmp = self.fetch16().rotate_right(1);
		self.set_carry_if((self.get_data() & 1) != 0);
		self.set_nz(tmp);
		self.check_mode(tmp);
		0
	}

	/// Subtraction with carry
	fn sbc(&mut self) -> u8 {
		let value = self.fetch16() ^ 255; // invert the value
		let tmp = self.get_a16() + value + self.get_carry_bit();

		self.set_cnz(tmp);
		self.set_overflow_if((tmp ^ self.get_a16() & (tmp ^ value)) & 128 != 0);
		self.set_a((tmp & 255) as u8);

		1
	}

	/// Set carry
	fn sec(&mut self) -> u8 {
		self.set_carry_if(true);
		0
	}

	/// Set disable interrupt
	fn sei(&mut self) -> u8 {
		self.set_int(true);
		0
	}

	/// Store accumulator
	fn sta(&mut self) -> u8 {
		self.write_last(self.get_a());
		0
	}

	/// Store X
	fn stx(&mut self) -> u8 {
		self.write_last(self.get_x());
		0
	}

	/// Store Y
	fn sty(&mut self) -> u8 {
		self.write_last(self.get_y());
		0
	}

	/// Transfer accumulator to X
	fn tax(&mut self) -> u8 {
		self.set_x(self.get_a());
		self.set_nz(self.get_x16());
		0
	}

	/// Transfer accumulator to Y
	fn tay(&mut self) -> u8 {
		self.set_y(self.get_a());
		self.set_nz(self.get_y16());
		0
	}

	/// Transfer stack pointer to X
	fn tsx(&mut self) -> u8 {
		self.set_x((self.get_sp() % 256) as u8);
		self.set_nz(self.get_x16());
		0
	}

	/// Transfer X to accumulator
	fn txa(&mut self) -> u8 {
		self.set_a(self.get_x());
		self.set_nz(self.get_a16());
		0
	}

	/// Transfer X to stack pointer
	fn txs(&mut self) -> u8 {
		self.set_sp(self.get_x_zp_addr());
		0
	}

	/// Transfer Y to accumulator
	fn tya(&mut self) -> u8 {
		self.set_a(self.get_y());
		self.set_nz(self.get_a16());
		0
	}
}
