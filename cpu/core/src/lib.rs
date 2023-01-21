use std::fmt::{
	Display,
	Formatter,
	self
};

/// Common device operations
pub trait Device {
	/// Reads data from an address on the device
	fn read(&self, address: usize, length: usize) -> Vec<u8>;

	/// Writes data to an address on the device
	fn write(&mut self, address: usize, data: &[u8]);

	/// Retrieves a single signed byte from the address on the device
	#[inline]
	fn get_i8(&self, address: usize) -> i8 {
		i8::from_ne_bytes(self.read(address, 1).try_into().unwrap())
	}

	/// Retrieves a single unsigned byte from the address on the device
	#[inline]
	fn get_u8(&self, address: usize) -> u8 {
		u8::from_ne_bytes(self.read(address, 1).try_into().unwrap())
	}

	/// Writes a single byte to the address on the device
	#[inline]
	fn put_u8(&mut self, address: usize, data: u8) {
		self.write(address, data.to_ne_bytes().as_slice());
	}

	/// Retrieves a little endian 16-bit unsigned value from the address on the device
	#[inline]
	fn get_u16_le(&self, address: usize) -> u16 {
		u16::from_le_bytes(self.read(address, 2).try_into().unwrap())
	}

	/// Retrieves a big endian 16-bit unsigned value from the address on the device
	#[inline]
	fn get_u16_be(&self, address: usize) -> u16 {
		u16::from_be_bytes(self.read(address, 2).try_into().unwrap())
	}

	/// Writes a big endian 16-bit unsigned value to the address on the device
	#[inline]
	fn put_u16_be(&mut self, address: usize, data: u16) {
		self.write(address, data.to_be_bytes().as_slice());
	}

	/// Writes a little endian 16-bit unsigned value to the address on the device
	#[inline]
	fn put_u16_le(&mut self, address: usize, data: u16) {
		self.write(address, data.to_le_bytes().as_slice());
	}

	/// Retrieves a little endian 32-bit unsigned value from the address on the device
	#[inline]
	fn get_u32_le(&self, address: usize) -> u32 {
		u32::from_le_bytes(self.read(address, 4).try_into().unwrap())
	}

	/// Retrieves a big endian 32-bit unsigned value from the address on the device
	#[inline]
	fn get_u32_be(&self, address: usize) -> u32 {
		u32::from_be_bytes(self.read(address, 4).try_into().unwrap())
	}

	/// Writes a big endian 32-bit unsigned value to the address on the device
	#[inline]
	fn put_u32_be(&mut self, address: usize, data: u32) {
		self.write(address, data.to_be_bytes().as_slice());
	}

	/// Writes a little endian 32-bit unsigned value to the address on the device
	#[inline]
	fn put_u32_le(&mut self, address: usize, data: u32) {
		self.write(address, data.to_le_bytes().as_slice());
	}

	/// Retrieves a little endian 64-bit unsigned value from the address on the device
	#[inline]
	fn get_u64_le(&self, address: usize) -> u64 {
		u64::from_le_bytes(self.read(address, 8).try_into().unwrap())
	}

	/// Retrieves a big endian 64-bit unsigned value from the address on the device
	#[inline]
	fn get_u64_be(&self, address: usize) -> u64 {
		u64::from_be_bytes(self.read(address, 8).try_into().unwrap())
	}

	/// Writes a big endian 64-bit unsigned value to the address on the device
	#[inline]
	fn put_u64_be(&mut self, address: usize, data: u64) {
		self.write(address, data.to_be_bytes().as_slice());
	}

	/// Writes a little endian 64-bit unsigned value to the address on the device
	#[inline]
	fn put_u64_le(&mut self, address: usize, data: u64) {
		self.write(address, data.to_le_bytes().as_slice());
	}
}

/// Generic memory bus
#[derive(Clone, Debug)]
pub struct Bus {
	pub ram: Vec<u8>,
}

impl Bus {
	pub fn new(ram_size: usize) -> Bus {
		Bus {
			ram: vec![0; ram_size],
		}
	}
}

impl Device for Bus {
	fn read(&self, address: usize, length: usize) -> Vec<u8> {
		self.ram.iter().skip(address).take(length).map(|b| *b).collect()
	}

	fn write(&mut self, address: usize, data: &[u8]) {
		data.iter().enumerate().for_each(|(i, b)| {
			self.ram[address + i] = *b;
		});
	}
}

impl Display for Bus {
	/// Writes the RAM as a 16-column hexdump with ASCII view
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		for i in (0..self.ram.len()).step_by(16) {
			write!(f, "{:04X}:\t", i)?;

			for x in 0..16 {
				write!(f, " {:02X}", self.ram[i + x])?;
			}
			write!(f, "\t")?;

			for x in 0..16 {
				let c = self.ram[i + x];

				// check for ASCII and Latin-1 supplement
				match c {
					(32..=126) | (161..=255) => write!(f, "{}", c as char)?,
					_ => write!(f, ".")?,
				}
			}
			writeln!(f, "")?;
		}

		Ok(())
	}
}

#[test]
fn test_bus() {
	let mut bus = Bus::new(65536);
	let filler = b"Hello";

	for i in (0..65536).step_by(5) {
		for (x, j) in filler.iter().enumerate() {
			if (i + x) < 65536 {
				bus.write(i + x, &[*j]);
			}
		}
	}

	println!("{}", &bus);
}
