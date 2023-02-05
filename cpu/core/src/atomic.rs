use parking_lot::Mutex;

use std::{
	collections::HashMap,
	sync::Arc
};

use crate::{
	DeviceMapBase,
	Region
};

pub type AtomicRegionMap = HashMap<usize, Arc<Mutex<Region>>>;

/// Common atomic device operations
pub trait AtomicDeviceMap: DeviceMapBase {
	fn get_region(&self, offset: usize) -> Option<Arc<Mutex<Region>>>;
}
