use std::
	ffi::OsString,
	mem::size_of,
	os::windows::ffi::OsStringExt
};

use winapi::{
	shared::lmcons::{
		CNLEN,
		UNLEN
	},
	um::{
		errhandlingapi::GetLastError,
		winbase::{
			GetComputerNameW,
			GetUserNameW
		}
	}
};

/// Returns the host device name
fn get_device_name() -> Result<String, u32> {
	let mut name: Vec<u16> = vec![0; CNLEN as usize];
	let mut size = (size_of::<u16>() * name.len()) as u32;

	unsafe {
		match GetComputerNameW(name.as_mut_ptr(), &mut size) {
			0 => Err(GetLastError()),
			_ => match OsString::from_wide(&*name).into_string() {
				Ok(s) => Ok(s),
				_ => Err(u32::MAX),
			}
		}
	}
}

/// Returns the device's current user name
fn get_user_name() -> Result<String, u32> {
	let mut name: Vec<u16> = vec![0; UNLEN as usize];
	let mut size = (size_of::<u16>() * name.len()) as u32;

	unsafe {
		match GetUserNameW(name.as_mut_ptr(), &mut size) {
			0 => Err(GetLastError()),
			_ => match OsString::from_wide(&*name).into_string() {
				Ok(s) => Ok(s),
				_ => Err(u32::MAX),
			}
		}
	}
}
