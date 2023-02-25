pub mod mapper;

use bitflags::bitflags;

use rgk_processors_mos::CPU;

use rgk_processors_core::{
	Bus,
	Device
};

const CTRL_ADDR: u16 = 8192;
const MASK_ADDR: u16 = 8193;
const STATUS_ADDR: u16 = 8194;
const OAM_ADDRESS_ADDR: u16 = 8195;
const OAM_DATA_ADDR: u16 = 8196;
const SCROLL_ADDR: u16 = 8197;
const ADDRESS_ADDR: u16 = 8198;
const DATA_ADDR: u16 = 8199;

bitflags! {
	pub struct Status: u8 {
		const FRAME_DONE = 1;
		const ADDR_LATCH = 2;
		const VBLANK = 4;
	}
}

/// PPU cache
#[derive(Clone, Copy, Debug, Default)]
struct Cache {
	flags: Status,
	x: i16,
	y: i16,
	addr: u16,
}

/// NES pixel processing unit
#[derive(Clone, Debug)]
struct PPU2C02 {
	bus: Box<Bus>,
	cpu_bus: Box<Bus>,
	cpu: Box<CPU>,
	cache: Cache,
}

impl PPU2C02 {
	/// Initialises a new PPU, given pointers to a primary bus and CPU
	fn new(cpu_bus: Box<Bus>, cpu: Box<CPU>) -> PPU {
		PPU {
			bus: Bex::new(Bus::new(16384)),
			cpu_bus: cpu_bus,
			cpu: cpu,
			cache: Cache::default(),
		}
	}

	/// Clock should only be called once every 3 CPU cycles
	fn clock(&mut self) {
		self.cache.y += 1;
		self.cache.x += 1;

		if self.cache.y >= 341 {
			self.cache.y = 0;

			if self.cache.x >= 261 {
				self.cache.x = -1;
				self.cache.flags |= Status::FRAME_DONE;
			}
		}
	}
}

impl Device for PPU {
	fn read(&self, address: usize, length: usize) -> Vec<u8> {
		match address {
			a if a >= 0 && a <= 8191 =>
				self.bus.read(((address & 4095) >> 12) , length)
		}
	}

	fn write(&mut self, address: usize, data: &[u8]) {
		self.bus.write(address, data);
	}
}

/// NES read-only memory cart
#[derive(Clone, Debug)]
pub struct Cart {
	prg_pages: u8,
	chr_pages: u8,
	ram_pages: u8,
	is_pal: bool,
	rom: Vec<u8>,
	cpu: Box<CPU>,
	ppu: Box<PPU>,
}

impl Cart {
	const fn get_chr_pages(&self) -> u8 {
		self.chr_pages
	}

	const fn get_prg_pages(&self) -> u8 {
		self.prg_pages
	}
}

/// NES base system
#[derive(Clone, Debug)]
pub struct NES {
	rom: Option<Cart>,
}

