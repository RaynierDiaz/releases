use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr::null_mut};
use user32::MessageBoxW;
use winapi::um::winuser::MB_OK;



pub fn show_message_box(title: impl AsRef<str>, message: impl AsRef<str>) {
	fn to_wide(string: &str) -> Vec<u16> {
		OsStr::new(string).encode_wide().chain(Some(0).into_iter()).collect()
	}
	let (title, message) = (title.as_ref(), message.as_ref());
	let (title, message) = (to_wide(title), to_wide(message));
	
	unsafe {
		MessageBoxW(
			null_mut(),
			message.as_ptr(),
			title.as_ptr(),
			MB_OK,
		);
	}
}
