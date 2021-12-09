use std::{
	env,
	fs::read
};

use meshio_ninjaripper::rip::import::{
	rip,
	RipImportError
};

fn main() -> Result<(), RipImportError> {
	let args: Vec<String> = env::args().collect();
	let data = read(&args[1]);
	if let Ok(input) = data {
		if let Ok(mdl) = rip(&mut input.as_slice()) {
			println!("{:#?}", mdl);
		}
	}

	Ok(())
}
