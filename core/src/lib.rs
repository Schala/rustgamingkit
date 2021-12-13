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

/// Converts a 3-byte string into a 24-bit big endian integer, stored as a 32-bit padded unsigned integer.
/// Byte strings longer than 3 bytes are truncated.
#[macro_export]
macro_rules! tag3 {
	($b3: literal) => {
		u32::from_be_bytes([$b3[0], $b3[1], $b3[2], 0])
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

