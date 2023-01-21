#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum ImageType {
	Icon = 1,
	Cursor,
	Unknown = 65535,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
	reserved: u16,
	pub kind: ImageType,
	pub num_images: u16,
}
