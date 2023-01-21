use rgk_processors_core::Bus;
use rgk_processors_mos::MOS6500;

/// Atari emulator
pub struct Atari {
	cpu: MOS6500,
}

impl Atari {
	pub fn new() -> Self {
		Self {
			cpu: MOS6500::new(Bus::new(65536)),
		}
	}
}
