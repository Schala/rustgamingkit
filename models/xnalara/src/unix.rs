use libc::gethostname;
use std::ffi::OsString;
use users::get_current_username;

/// Returns the device name
fn get_device_name() -> Result<String, i32> {
	let mut name: Vec<i8> = vec![0; 64];

	unsafe {
		let res = gethostname(name.as_mut_ptr(), 64);
		match res{
			0 => Ok(String::from_utf8_lossy(name.iter()
					.map(|&c| (c as u8))
					.collect::<Vec<_>>().as_slice())
					.to_owned()
					.to_string()),
			_ => Err(res),
		}
	}
}

/// Returns the device's current user name
fn get_user_name() -> Result<String, OsString> {
	match get_current_username() {
		Some(name) => name.into_string(),
		None => Ok("".to_string()),
	}
}
