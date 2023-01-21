use rgk_processors_core::Bus;
use rgk_processors_mos::MOS6500;

/// Commodore emulator
pub struct Commodore {
	cpu: MOS6500,
}

impl Commodore {
	pub fn new() -> Self {
		Self {
			cpu: MOS6500::new(Bus::new(65536)),
		}
	}
}
