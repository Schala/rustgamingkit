use bitflags::bitflags;
use indexmap::IndexMap;

use std::{
	cell::RefCell,
	collections::HashMap,
	fmt::{
		Display,
		Formatter,
		self
	},
	rc::Rc
};

#[cfg(feature = "shared")]
pub mod shared;

bitflags! {
	#[derive(Default)]
	pub struct RegionFlags: u8 {
		/// Region is a pointer
		const PTR = 1;

		/// Region is an array
		const ARRAY = 2;
	}
}

/// Region type
#[derive(Clone, Debug, Default, PartialEq)]
pub enum RegionType {
	/// Simple label
	#[default]
	Label,

	/// Function
	Function,

	/// General section of memory
	Section,

	/// Signed byte
	Signed8,

	/// Unsigned byte
	Unsigned8,

	/// Signed word
	Signed16,

	/// Unsigned word
	Unsigned16,

	/// Signed 32-bit integer
	Signed32,

	/// Unsigned 32-bit integer
	Unsigned32,

	/// Single-precision floating point
	Float32,

	/// Signed 64-bit integer
	Signed64,

	/// Unsigned 64-bit integer
	Unsigned64,

	/// Double-precision floating point
	Float64,

	/// Null-terminated string
	CString,

	/// Pointer
	Pointer,

	/// Byte length-prefixed string
	PString,

	/// Structure (struct, class, record, object, etc.)
	Structure(RawRegionMap),

	/// Union/variant
	Union(Vec<Region>),
}

/// A region of code
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
	flags: RegionFlags,
	kind: RegionType,
	label: String,
	refs: Vec<usize>,
	size: usize,
}

pub type RawRegionMap = HashMap<usize, Region>;
pub type RegionMap = IndexMap<usize, Rc<RefCell<Region>>>;

impl Region {
	/// Creates a new memory region with the specified type, size and label
	pub fn new(size: usize, kind: RegionType, flags: RegionFlags, label: &str) -> Region {
		Region {
			flags,
			size,
			kind,
			label: label.to_owned(),
			refs: vec![],
		}
	}

	/// Adds an address that references the region
	pub fn add_ref(&mut self, addr: usize) {
		self.refs.push(addr);
	}

	/// Gets the size of the region if the type is an array, otherwise the singular size is given
	pub fn get_array_size(&self) -> usize {
		if self.is_array() {
			self.size / self.get_size()
		} else {
			self.get_size()
		}
	}

	/// Gets the region label
	pub fn get_label(&self) -> &str {
		self.label.as_str()
	}

	/// Gets the region's referencing addresses
	pub fn get_refs(&self) -> &[usize] {
		self.refs.as_ref()
	}

	/// Gets the size of the region. This currently does not take alignment
	/// into account.
	pub fn get_size(&self) -> usize {
		match &self.kind {
			RegionType::Signed8 | RegionType::Unsigned8 => 1,
			RegionType::Signed16 | RegionType::Unsigned16 => 2,
			RegionType::Signed32 | RegionType::Unsigned32 | RegionType::Float32 => 4,
			RegionType::Signed64 | RegionType::Unsigned64 | RegionType::Float64 => 8,
			RegionType::Structure(s) => {
				let mut sz = 0;
				for (_, r) in s.iter() {
					sz += r.get_size();
				}
				sz
			},
			RegionType::Union(u) => {
				let mut sz = 0;
				for r in u.iter() {
					let rsz = r.get_size();
					if rsz > sz {
						sz = rsz;
					}
				}
				sz
			}
			_ => self.size,
		}
	}

	/// Gets the region type
	pub fn get_type(&self) -> &RegionType {
		&self.kind
	}

	/// Is the region flagged as an array?
	pub const fn is_array(&self) -> bool {
		self.flags.contains(RegionFlags::ARRAY)
	}

	/// Is the region flagged as a pointer?
	pub const fn is_ptr(&self) -> bool {
		self.flags.contains(RegionFlags::PTR)
	}

	/// Changes a label to a function
	pub fn label_to_fn(&mut self, new_label: Option<&str>) {
		if self.kind == RegionType::Label {
			self.kind = RegionType::Function;

			if let Some(label) = new_label {
				self.label = label.to_owned();
			}
		}
	}
}

/// Common processor operations
pub trait Processor {
	/// Execute one clock cycle
	fn clock(&mut self);

	/// Gets the pointer size of the processor
	fn get_ptr_size(&self) -> usize;
}

/// Common memory region mapping operations
pub trait DeviceMap {
	/// Registers a region in memory
	fn add_region(&mut self, address: usize, region: Region);

	/// Generates regions between the given start and end offsets
	fn generate_regions(&mut self, start: usize, end: usize);

	/// Does the region exist at the specified offset?
	fn region_exists(&self, offset: usize) -> bool;

	/// Sorts region offsets in order
	fn sort_regions(&mut self);

	/// Registers multiple regions in memory
	fn add_regions(&mut self, mut map: RawRegionMap) {
		for (a, r) in map.drain() {
			self.add_region(a, r);
		}
	}
}

/// Common disassembler operations
pub trait Disassembler {
	/// Analyses one region
	fn analyze(&mut self, offset: &mut usize);

	/// Analyses a range of binary
	fn analyze_range(&mut self, start: usize, end: usize) {
		let mut offset = start;

		while offset < end {
			self.analyze(&mut offset);
		}
	}

	/// Returns the code at the given offset, if any
	fn get_code_at_offset(&self, offset: usize) -> Option<String>;

	/// Returns the label at the given offset, if any
	fn get_label_at_offset(&self, offset: usize) -> Option<String>;
}

/// Common device operations
pub trait DeviceBase {
	/// Reads data from an address on the device
	fn read(&self, address: usize, length: usize) -> Vec<u8>;

	/// Writes data to an address on the device
	fn write(&mut self, address: usize, data: &[u8]);

	/// Retrieves a single signed byte from the address on the device
	fn get_i8(&self, address: usize) -> i8 {
		i8::from_ne_bytes(self.read(address, 1).try_into().unwrap())
	}

	/// Retrieves a single unsigned byte from the address on the device
	fn get_u8(&self, address: usize) -> u8 {
		u8::from_ne_bytes(self.read(address, 1).try_into().unwrap())
	}

	/// Writes a single byte to the address on the device
	fn put_u8(&mut self, address: usize, data: u8) {
		self.write(address, data.to_ne_bytes().as_slice());
	}

	/// Retrieves a little endian 16-bit signed value from the address on the device
	fn get_i16_le(&self, address: usize) -> i16 {
		i16::from_le_bytes(self.read(address, 2).try_into().unwrap())
	}

	/// Retrieves a big endian 16-bit signed value from the address on the device
	fn get_i16_be(&self, address: usize) -> i16 {
		i16::from_be_bytes(self.read(address, 2).try_into().unwrap())
	}

	/// Retrieves a little endian 16-bit unsigned value from the address on the device
	fn get_u16_le(&self, address: usize) -> u16 {
		u16::from_le_bytes(self.read(address, 2).try_into().unwrap())
	}

	/// Retrieves a big endian 16-bit unsigned value from the address on the device
	fn get_u16_be(&self, address: usize) -> u16 {
		u16::from_be_bytes(self.read(address, 2).try_into().unwrap())
	}

	/// Writes a big endian 16-bit unsigned value to the address on the device
	fn put_u16_be(&mut self, address: usize, data: u16) {
		self.write(address, data.to_be_bytes().as_slice());
	}

	/// Writes a little endian 16-bit unsigned value to the address on the device
	fn put_u16_le(&mut self, address: usize, data: u16) {
		self.write(address, data.to_le_bytes().as_slice());
	}

	/// Retrieves a little endian 32-bit unsigned value from the address on the device
	fn get_u32_le(&self, address: usize) -> u32 {
		u32::from_le_bytes(self.read(address, 4).try_into().unwrap())
	}

	/// Retrieves a big endian 32-bit unsigned value from the address on the device
	fn get_u32_be(&self, address: usize) -> u32 {
		u32::from_be_bytes(self.read(address, 4).try_into().unwrap())
	}

	/// Writes a big endian 32-bit unsigned value to the address on the device
	fn put_u32_be(&mut self, address: usize, data: u32) {
		self.write(address, data.to_be_bytes().as_slice());
	}

	/// Writes a little endian 32-bit unsigned value to the address on the device
	fn put_u32_le(&mut self, address: usize, data: u32) {
		self.write(address, data.to_le_bytes().as_slice());
	}

	/// Retrieves a little endian 64-bit unsigned value from the address on the device
	fn get_u64_le(&self, address: usize) -> u64 {
		u64::from_le_bytes(self.read(address, 8).try_into().unwrap())
	}

	/// Retrieves a big endian 64-bit unsigned value from the address on the device
	fn get_u64_be(&self, address: usize) -> u64 {
		u64::from_be_bytes(self.read(address, 8).try_into().unwrap())
	}

	/// Writes a big endian 64-bit unsigned value to the address on the device
	fn put_u64_be(&mut self, address: usize, data: u64) {
		self.write(address, data.to_be_bytes().as_slice());
	}

	/// Writes a little endian 64-bit unsigned value to the address on the device
	fn put_u64_le(&mut self, address: usize, data: u64) {
		self.write(address, data.to_le_bytes().as_slice());
	}
}

/// Single-threaded device operations
pub trait Device: DeviceBase {
	/// Gets a reference to the device's bus
	fn get_bus(&self) -> Rc<RefCell<Bus>>;

	/// Gets a region on this device
	fn get_region(&self, offset: usize) -> Option<Rc<RefCell<Region>>>;

	/// Gets a mutable region on this device
	fn get_region_mut(&mut self, offset: usize) -> Option<Rc<RefCell<Region>>>;

	/// Gets a vector of a copy of all regions, useful for debugging
	fn get_all_regions(&self) -> Vec<(usize, Region)>;
}

/// Single-threaded memory bus
#[derive(Clone, Debug)]
pub struct Bus {
	ram: Vec<u8>,
	rgns: RegionMap,
}

impl Bus {
	pub fn new(ram_size: usize) -> Bus {
		Bus {
			ram: vec![0; ram_size],
			rgns: RegionMap::new(),
		}
	}
}

impl DeviceMap for Bus {
	fn add_region(&mut self, address: usize, region: Region) {
		if !self.region_exists(address) {
			self.rgns.insert(address, Rc::new(RefCell::new(region)));
		}
	}

	fn generate_regions(&mut self, _start: usize, _end: usize) {
		unimplemented!("Region generation called on ambiguous memory bus");
	}

	fn region_exists(&self, offset: usize) -> bool {
		self.rgns.contains_key(&offset)
	}

	fn sort_regions(&mut self) {
		self.rgns.sort_keys();
	}
}

impl DeviceBase for Bus {
	fn read(&self, address: usize, length: usize) -> Vec<u8> {
		self.ram.iter().skip(address).take(length).map(|b| *b).collect()
	}

	fn write(&mut self, address: usize, data: &[u8]) {
		data.iter().enumerate().for_each(|(i, b)| {
			self.ram[address + i] = *b;
		});
	}
}

impl Device for Bus {
	fn get_bus(&self) -> Rc<RefCell<Bus>> {
		unimplemented!("Bus attempted to get a reference counted pointer of itself");
	}

	fn get_region(&self, offset: usize) -> Option<Rc<RefCell<Region>>> {
		self.rgns.get(&offset).cloned()
	}

	fn get_region_mut(&mut self, offset: usize) -> Option<Rc<RefCell<Region>>> {
		self.rgns.get_mut(&offset).cloned()
	}

	fn get_all_regions(&self) -> Vec<(usize, Region)> {
		self.rgns.iter().map(|(o, r)| (*o, r.borrow().clone())).collect()
	}
}

impl Display for Bus {
	/// Writes the RAM as a 16-column hexdump with ASCII view
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self.ram.len() {
			0..=255 => write!(f, "{}", hexdump(self.ram.as_ref(), 1)),
			256..=65535 => write!(f, "{}", hexdump(self.ram.as_ref(), 2)),
			65536..=4294967295 => write!(f, "{}", hexdump(self.ram.as_ref(), 4)),
			_ => unreachable!(),
		}
	}
}

/// Outputs the provided data as a 16-column hexdump with ASCII view
pub fn hexdump(data: &[u8], ptr_size: u8) -> String {
	let mut s = String::new();

	for i in (0..data.len()).step_by(16) {
		let addr = match ptr_size {
			1 => format!("{:02X}:\t", i),
			2 => format!("{:04X}:\t", i),
			4 => format!("{:08X}:\t", i),
			8 => format!("{:016X}:\t", i),
			_ => panic!("Invalid pointer size given"),
		};

		let mut bytes = String::new();
		for x in 0..16 {
			bytes = format!("{} {:02X}", bytes, data[i + x]);
		}
		bytes.push('\t');

		s.push_str(addr.as_str());
		s.push_str(bytes.as_str());

		for x in 0..16 {
			let c = data[i + x];

			// check for ASCII and Latin-1 supplement
			s.push(match c {
				(32..=126) | (161..=255) => c as char,
				_ => '.',
			});
		}
		s.push('\n');
	}

	s
}

#[cfg(test)]
mod tests {
	use super::*;

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
}
