use crate::Cart;

/// Maps addresses on devices to addresses in RAM
pub trait Mapper {
	fn map_chr(rom: &Cart, src_addr: usize, dest_addr: &mut usize) -> bool;
	fn map_prg(rom: &Cart, src_addr: usize, dest_addr: &mut usize) -> bool;
}

/// Generic designation NES ROM mapper
pub struct NROM;

impl Mapper for NROM {
	fn map_chr(rom: &Cart, src_addr: usize, dest_addr: &mut usize) -> bool {
		if src_addr >= 0 && src_addr <= 8191 {
			// Mirror the ROM read based on the ROM file offset
			*dest_addr = src_addr & 8191;
			return true;
		}

		false
	}

	fn map_prg(rom: &Cart, src_addr: usize, dest_addr: &mut usize) -> bool {
		if src_addr >= 32768 && src_addr <= 65535 {
			// Mirror the ROM read based on the ROM file offset, pending 16kb or 32kb
			*dest_addr = src_addr & if rom.get_prg_pages() > 1 { 32767 } else { 16383 };
			return true;
		}

		false
	}
}
