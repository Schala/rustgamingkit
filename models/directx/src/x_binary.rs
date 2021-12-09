use rgk_core::tag4;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Compression {
	LZW = tag4!(b"lzw "),
	Zip = tag4!(b"zip "),
}
