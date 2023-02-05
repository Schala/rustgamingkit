use std::rc::Rc;

use rgk_processors_core::{
	Bus,
	Device,
	DeviceBase,
	DeviceMap,
	Processor,
	Region,
	RegionMap,
	RegionType
};

use rgk_processors_zilog::Z80;

pub struct LR35902 {
	base: Z80,
}

impl LR35902 {
	/// Initialises a new LR35902, given a bus pointer
	pub pub fn new(bus: Rc<Bus>) -> LR35902 {
		LR35902 {
			base: Z80::new(bus),
		}
	}
}
