pub mod opcodes;
pub mod processor;

use processor::Processor;

pub const RAM_SIZE: usize = 65536;

pub trait Device {
	fn read(&self, address: u16) -> u8;
	fn write(&mut self, address: u16; data: u8);
}

#[derive(Clone, Debug)]
pub struct Bus {
	pub cpu: Option<Processor>,
	pub ram: [u8; RAM_SIZE],
}

impl Bus {
	pub fn new() -> Bus {
		let mut bus = Bus {
			cpu: None,
			ram: [0; RAM_SIZE],
		};

		bus.cpu = Processor::new(&mut bus);
		bus
	}
}

impl Device for Bus {
	fn read(&self, address: u16) -> u8 {
		self.ram[address as usize]
	}

	fn write(&mut self, address: u16; data: u8) {
		self.ram[address as usize] = data;
	}
}
