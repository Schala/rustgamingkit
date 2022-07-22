pub mod processor;

use processor::Processor;

pub trait Device {
	fn read(&self, address: u16) -> u8;
	fn write(&mut self, address: u16, data: u8);
}

#[derive(Clone, Debug)]
pub struct Bus<'a> {
	pub cpu: Option<Processor<'a>>,
	pub ram: Vec<u8>,
}

impl<'a> Bus<'a> {
	pub fn new(ramSize: u32) -> Bus<'a> {
		let mut bus = Bus {
			cpu: None,
			ram: vec![0; ramSize],
		};

		bus.cpu = Processor::new(&mut bus);

		bus
	}
}

impl<'a> Device for Bus<'a> {
	fn read(&self, address: u16) -> u8 {
		self.ram[address as usize]
	}

	fn write(&mut self, address: u16, data: u8) {
		self.ram[address as usize] = data;
	}
}

#[test]
fn test_bus() {
	let bus = Bus::new();
}
