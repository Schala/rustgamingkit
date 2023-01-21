use std::{
	fmt::{
		self,
		Display,
		Formatter
	},
	mem::transmute
};

#[derive(Clone, Copy)]
union Mantissa {
	value: u32,
	bytes: [u8; 3],
}

/// Commodore 64's 40-bit floating-point type
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub union f40 {
	fp: f32,
	exp: u8,
	man: Mantissa
}

impl f40 {
	pub fn floor(self) -> Self {
		let mut fp = self;

		unsafe {
			if self.exp == 0 || self.exp == 128 || self.exp == 129 {
				return fp;
			}

			let frac = 32 - ((self.exp as i32) - 128);
			let mask = 0xFFFF_FFFF << (frac as u32);
			fp.man.value = self.man.value & mask;
		}

		fp
	}
}

impl Display for f40 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
	}
}

impl From<f32> for f40 {
	fn from(small: f32) -> Self {
		let raw = unsafe { transmute::<f32, u32>(small) };
		let exp = ((raw & 0x7F8_0000) >> 19) + 128;
		let m = (raw & 0x7F_FFFF).to_le_bytes();
		let val = 128 + ((m[0] as u32) << 8) | ((m[1] as u32) << 16) |
			((m[2] as u32) << 24);

		Self {
			man: Mantissa {
				value: val,
			}
		}
	}
}

/*impl From<f64> for f40 {
	fn from(large: f64) -> Self {
	}
}*/

#[cfg(test)]
mod tests {
	use super::*;

	fn test_from_to_f32() {
		let fp40 = f40::from(1.2345);
		let fp32: f32 = fp40.into();

		println!("{}", fp32);
	}

	fn test_floor() {

	}
}
