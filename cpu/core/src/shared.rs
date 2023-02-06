use parking_lot::Mutex;

use std::{
	collections::HashMap,
	sync::Arc
};

use crate::{
	DeviceMapBase,
	Region
};

pub type SharedRegionMap = HashMap<usize, Arc<Mutex<Region>>>;

/// Common atomic device operations
pub trait SharedDeviceMap: DeviceMapBase {
	fn get_region(&self, offset: usize) -> Option<Arc<Mutex<Region>>>;
}
