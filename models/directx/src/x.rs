use rgk_core::tag4;

pub const MAGIC: u32 = tag4!(b"xof ");
pub const VERSION: u16 = tag4!(b"0303");

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Format {
	Binary = tag4!(b"bin "),
	Compressed = tag4!(b"com "),
	Text = tag4!(b"txt "),
	Unknown = 0xFFFFFFFF,
}
