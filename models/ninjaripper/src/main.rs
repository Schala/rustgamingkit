use std::{
	env,
	fs::read,
	io::Cursor
};

use rgk_models_ninjaripper::rip::{
	RipModel,
	RipImportError
};

fn main() -> Result<(), RipImportError> {
	let args: Vec<String> = env::args().collect();
	let mut data = Cursor::new(read(&args[1])?);
	let model = RipModel::read(&mut data)?;

	println!("{:#?}", model);

	Ok(())
}
