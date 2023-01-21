use std::io::{
	Read,
	Result
};

use ultraviolet::vec::{
	Vec2,
	Vec3,
	Vec4
};

pub trait ReadBinExt: Read {
	/// Reads a null-terminated string
	#[inline]
	fn read_cstr(&mut self) -> Result<String> {
		let mut s = String::new();
		let mut buf = [1; 1];

		while buf[0] != 0 {
			self.read_exact(&mut buf)?;
			if buf[0] != 0 {
				s.push(buf[0] as char);
			}
		}

		Ok(s)
	}

	/// Reads a Pascal style string
	#[inline]
	fn read_pstr(&mut self) -> Result<String> {
		let mut s = String::new();
		let mut buf = [0; 1];

		self.read_exact(&mut buf)?;
		let length = buf[0] as usize;

		for _ in 0..length {
			self.read_exact(&mut buf)?;
			s.push(buf[0] as char);
		}

		Ok(s)
	}

	/// Reads a little endian 2D vector
	#[inline]
	fn read_vec2_le(&mut self) -> Result<Vec2> {
		let mut x = [0; 4];
		let mut y = x;

		self.read_exact(&mut x)?;
		self.read_exact(&mut y)?;

		Ok(Vec2::new(f32::from_le_bytes(x), f32::from_le_bytes(y)))
	}

	/// Reads a little endian 3D vector
	#[inline]
	fn read_vec3_le(&mut self) -> Result<Vec3> {
		let mut x = [0; 4];
		let mut y = x;
		let mut z = y;

		self.read_exact(&mut x)?;
		self.read_exact(&mut y)?;
		self.read_exact(&mut z)?;

		Ok(Vec3::new(f32::from_le_bytes(x), f32::from_le_bytes(y), f32::from_le_bytes(z)))
	}

	/// Reads a little endian 4D vector
	#[inline]
	fn read_vec4_le(&mut self) -> Result<Vec4> {
		let mut x = [0; 4];
		let mut y = x;
		let mut z = y;
		let mut w = z;

		self.read_exact(&mut x)?;
		self.read_exact(&mut y)?;
		self.read_exact(&mut z)?;
		self.read_exact(&mut w)?;

		Ok(Vec4::new(f32::from_le_bytes(x), f32::from_le_bytes(y), f32::from_le_bytes(z),
			f32::from_le_bytes(w)))
	}
}

impl<R> ReadBinExt for R
where
	R: Read + ?Sized,
{
}

#[cfg(test)]
mod tests {
	use std::io::Read;

	use ultraviolet::vec::{
		Vec2,
		Vec3,
		Vec4
	};

	use super::*;

	#[test]
	fn test_read_cstr() {
		let mut data = &b"test\x00123454321"[..];
		assert_eq!("test".to_string(), data.read_cstr().unwrap());
	}

	#[test]
	fn test_read_pstr() {
		let mut data = &b"\x04test123454321"[..];
		assert_eq!("test".to_string(), data.read_pstr().unwrap());
	}

	#[test]
	fn test_read_vecs() {
		let mut vec2: &[u8] = &[0x5c, 0x1f, 0x7f, 0x3c, 0xa4, 0xfb, 0xf0, 0x3d][..];
		let mut vec3: &[u8] = &[0x5c, 0x1f, 0x7f, 0x3c, 0xa4, 0xfb, 0xf0, 0x3d, 0xd4, 0xf1, 0xb6, 0x3d][..];
		let mut vec4: &[u8] = &[0x5c, 0x1f, 0x7f, 0x3c, 0xa4, 0xfb, 0xf0, 0x3d, 0xd4, 0xf1, 0xb6, 0x3d,
			0, 0xa0, 0xd9, 0xbd][..];
		assert_eq!(Vec2::new(0.0155714415, 0.117667466), vec2.read_vec2_le().unwrap());
		assert_eq!(Vec3::new(0.0155714415, 0.117667466, 0.089328438), vec3.read_vec3_le().unwrap());
		assert_eq!(Vec4::new(0.0155714415, 0.117667466, 0.089328438, -0.106262207), vec4.read_vec4_le().unwrap());
	}
}
