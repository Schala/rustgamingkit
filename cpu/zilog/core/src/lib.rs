use bitflags::bitflags;

use std::{
	default::Default,
	fmt::{
		Display,
		Formatter,
		self
	}
};

use rgk_processors_core::{
	Bus,
	Device
};

bitflags! {
	/// Z80 state flags
	struct Status: u8 {
		const S = 1;
		const Z = 2;
		const H = 8;
		const PV = 32;
		const N = 64;
		const C = 128;
	}
}

impl Display for Status {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if self.contains(Status::S) {
			write!(f, "S")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::Z) {
			write!(f, "Z")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::H) {
			write!(f, "H")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::PV) {
			write!(f, "P")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::N) {
			write!(f, "N")?;
		} else {
			write!(f, "x")?;
		}

		if self.contains(Status::C) {
			write!(f, "C")?;
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

/// Z80 registers
#[derive(Clone, Copy, Debug)]
pub struct Registers {
	a: u8,
	f: Status,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	h: u8,
	l: u8,
	x: u16,
	y: u16,
	s: u16,
	pc: u16,
}

pub struct Z80 {
	bus: Box<Bus>
	regs: Registers,
}

impl Z80 {
	/// Initialises a new Z80, given a bus pointer
	pub fn new(bus: Box<Bus>) -> Z80 {
		Z80 {
			bus,
		}
	}

	/// Gets the accumulator register
	const fn get_a(&self) -> u8 {
		self.regs.a
	}

	/// Gets the flags register bits
	const fn get_f_bits(&self) -> u8 {
		self.regs.f.bits()
	}
}
