use bitflags::bitflags;

use rgk_processors_core::{
	Bus,
	Device
};

pub static OPCODES: Vec<&str> = vec![
	"NOP",
	"AIN",
	"BIN",
	"CIN",
	"LDIA",
	"LDIB",
	"RDEXP",
	"WREXP",
	"STA",
	"STC",
	"ADD",
	"SUB",
	"MULT",
	"DIV",
	"JMP",
	"JMPZ",
	"JMPC",
	"JREG",
	"LDAIN",
	"STAOUT",
	"LDLGE",
	"STLGE",
	"LDW",
	"SWP",
	"SWPC",
	"PCR",
	"BSL",
	"BSR",
	"AND",
	"OR",
	"NOT",
	"BNK",
	"BNKC",
	"LDWB",
];

bitflags! {
	struct Status: u8 {
		const Z = 1; /// Zero
		const C = 2; /// Carry
	}
}

#[derive(Clone, Copy, Debug)]
struct Registers {
	f: Status,
	a: u16, /// general purpose
	b: u16, /// general purpose
	bank: u16,
	c: u16, /// general purpose
	d: u16, /// display
	i: u16, /// instruction
	m: u16, /// address
	pc: u16, /// counter
}

#[derive(Clone, Debug)]
pub struct CPU {
	bus: Box<Bus>,
	regs: Registers,
}

impl CPU {
	pub fn new() -> CPU {
		CPU {
			bus: Box::new(Bus::new(65536)),
			regs: Registers {
				a: 0,
				b: 0,
				c: 0,
				pc: 0,
			},
		}
	}

	// --- MICROINSTRUCTIONS

	fn and_m(&mut self) {
	}

	fn aw(&mut self) {
	}

	fn bnk_m(&mut self) {
	}

	fn ce(&mut self) {
	}

	/// Write counter register to bus
	fn cr(&mut self) {
	}

	/// Enable division in ALU
	fn di(&mut self) {
	}

	/// Read from address to display register
	fn dw(&mut self) {
	}

	/// End instruction
	fn ei(&mut self) {
	}

	fn fl(&mut self) {
	}

	/// Write lower 12 bits of instruction register to bus
	fn ir(&mut self) {
	}

	/// Read from address to instruction register
	fn iw(&mut self) {
	}

	fn j(&mut self) {
	}

	/// Enable multiplication in ALU
	fn mu(&mut self) {
	}

	fn not_m(&mut self) {
	}

	fn or_m(&mut self) {
	}

	/// Write A register to bus
	fn ra(&mut self) {
	}

	/// Write B register to bus
	fn rb(&mut self) {
	}

	/// Write C register to bus
	fn rc(&mut self) {
	}

	/// Write expansion port to bus
	fn re(&mut self) {
	}

	/// Read address to address register
	fn rm(&mut self) {
	}

	fn sl(&mut self) {
	}

	fn sr(&mut self) {
	}

	fn st(&mut self) {
	}

	/// Enable subtraction in ALU
	fn su(&mut self) {
	}

	/// Read from address to A register
	fn wa(&mut self) {
	}

	/// Read from address to B register
	fn wb(&mut self) {
	}

	/// Read from address to C register
	fn wc(&mut self) {
	}

	fn wm(&mut self) {
	}


	// --- INSTRUCTIONS

	/// Add values of registers A and B, store the result in A
	fn add(&mut self) {
	}

	/// Load data from address into A register
	fn ain(&mut self) {
	}

	/// Logical and values of registers A and B, store the result in A
	fn and(&mut self) {
	}

	/// Load data from address into B register
	fn bin(&mut self) {
	}

	/// Change bank register to immediate value
	fn bnk(&mut self) {
	}

	/// Change bank register to C register
	fn bnkc(&mut self) {
	}

	/// Left shift A register value
	fn bsl(&mut self) {
	}

	/// Right shift A register value
	fn bsr(&mut self) {
	}

	/// Load data from address into C register
	fn cin(&mut self) {
	}

	/// Divide values of registers A and B, store the result in A
	fn div(&mut self) {
	}

	/// Fetch
	fn fetch(&mut self) {
	}

	/// Jump to address
	fn jmp(&mut self) {
	}

	/// Jump to address if carry bit set
	fn jmpc(&mut self) {
	}

	/// Jump to address if A register value is 0
	fn jmpz(&mut self) {
	}

	fn jreg(&mut self) {
	}

	/// Load A register value as memory address
	fn ldain(&mut self) {
	}

	/// Load immediate value into A register
	fn ldia(&mut self) {
	}

	/// Load immediate value into B register
	fn ldib(&mut self) {
	}

	/// Load immediate value as address and copy A register value
	fn ldlge(&mut self) {
	}

	/// Load immediate word into A register
	fn ldw(&mut self) {
	}

	/// Load immediate word into B register
	fn ldwb(&mut self) {
	}

	/// Multiply values of registers A and B, store the result in A
	fn mult(&mut self) {
	}

	/// No operation
	fn nop(&self) {
	}

	/// Logical not value of register A, store the result in A
	fn not(&mut self) {
	}

	/// Logical or values of registers A and B, store the result in A
	fn or(&mut self) {
	}

	/// Load counter into A register
	fn pcr(&mut self) {
	}

	/// Load expansion port value into A register
	fn rdexp(&mut self) {
	}

	/// Store value of A register in memory
	fn sta(&mut self) {
	}

	/// Use A register value as memory address
	fn staout(&mut self) {
	}

	/// Store value of C register in memory
	fn stc(&mut self) {
	}

	/// Use immediate value as address and store A register value
	fn stlge(&mut self) {
	}

	/// Subtract values of registers A and B, store the result in A
	fn sub(&mut self) {
	}

	/// Swap A and B register values
	fn swp(&mut self) {
	}

	/// Swap A and C register values
	fn swpc(&mut self) {
	}

	/// Write value of A register to expansion port
	fn wrexp(&mut self) {
	}
}

impl Device for CPU {
	fn read(&self, address: usize, length: usize) -> Vec<u8> {
		self.bus.read(address, length)
	}

	fn write(&mut self, address: usize, data: &[u8]) {
		self.bus.write(address, data);
	}
}

/// Returns the upper 5 bits of a 16 bit operation
pub const fn get_instruction(op: u16) -> u16 {
	(op & 0b1111_1000_0000_0000) >> 11
}

/// Returns the lower 11 bits of a 16 bit operatioon
pub const fn get_operation(op: u16) -> u16 {
	op & 0b111_1111_1111
}
