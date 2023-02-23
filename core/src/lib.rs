#[cfg(feature = "bit_ext")]
pub mod bit_ext;

#[cfg(feature = "io_ext")]
pub mod io_ext;

#[cfg(feature = "nom_ext")]
pub mod nom_ext;

pub mod scene;
pub mod texture;

/// Converts a 4-byte string into a 32-bit little endian integer.
/// Byte strings longer than 4 bytes are truncated.
#[macro_export]
macro_rules! rtag4 {
	($b4: literal) => {
		u32::from_le_bytes([$b4[0], $b4[1], $b4[2], $b4[3]])
	}
}

/// Converts a 2-byte string into a 16-bit big endian integer.
/// Byte strings longer than 2 bytes are truncated.
#[macro_export]
macro_rules! tag2 {
	($b2: literal) => {
		u16::from_be_bytes([$b2[0], $b2[1]])
	}
}

/// Converts a 4-byte string into a 32-bit big endian integer.
/// Byte strings longer than 4 bytes are truncated.
#[macro_export]
macro_rules! tag4 {
	($b4: literal) => {
		u32::from_be_bytes([$b4[0], $b4[1], $b4[2], $b4[3]])
	}
}

/// Scales a 5 bit value to 8 bits
pub const fn scale5to8(b: u8) -> u8 {
	b << 3 | b >> 2
}

/// Scales an 8 bit value to 5 bits
pub const fn scale8to5(b: u8) -> u8 {
	(b & 0xF8) >> 3
}
