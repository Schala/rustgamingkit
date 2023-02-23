/// Zilog instruction set
pub trait Z80ISA {
	/// Add with carry
	fn adc(&mut self) -> u8;

	/// Addition
	fn add(&mut self) -> u8;

	/// Bitwise AND
	fn and(&mut self) -> u8;

	fn call(&mut self) -> u8;

	fn ccf(&mut self) -> u8;

	/// Comparison
	fn cp(&mut self) -> u8;

	fn cpl(&mut self) -> u8;

	fn daa(&mut self) -> u8;

	/// Decrement
	fn dec(&mut self) -> u8;

	fn di(&mut self) -> u8;

	fn ei(&mut self) -> u8;

	fn halt(&mut self) -> u8;

	/// Increment
	fn inc(&mut self) -> u8;

	/// Jump to address
	fn jp(&mut self) -> u8;

	fn jr(&mut self) -> u8;

	/// Load
	fn ld(&mut self) -> u8;

	/// Load into H
	fn ldh(&mut self) -> u8;

	/// No operation
	fn nop(&self) -> u8;

	/// Bitwise OR
	fn or(&mut self) -> u8;

	/// Pop from stack
	fn pop(&mut self) -> u8;

	/// Push to stack
	fn push(&mut self) -> u8;

	fn ret(&mut self) -> u8;

	fn reti(&mut self) -> u8;

	fn rla(&mut self) -> u8;

	fn rlca(&mut self) -> u8;

	fn rra(&mut self) -> u8;

	fn rrca(&mut self) -> u8;

	fn rst(&mut self) -> u8;

	/// Subtract with carry
	fn sbc(&mut self) -> u8;

	fn stop(&mut self) -> u8;

	/// Subtraction
	fn sub(&mut self) -> u8;

	/// Exclusive OR
	fn xor(&mut self) -> u8;
}
