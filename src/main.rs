mod screenshot;

use crate::screenshot::*;
use winapi::um::winuser::{MessageBoxW, MB_ICONERROR};

fn main() {
	let bmp = match screenshot(None) {
		Ok(bmp) => bmp,
		Err(msg) => {
			message_box_error("Windows error", msg);
			panic!(msg);
		}
	};

	let img = image::load_from_memory_with_format(&*bmp, image::BMP).unwrap();
	img.save("screenshot.png").unwrap();
}

pub fn message_box_error(title: &str, msg: &str) {
	unsafe {
		MessageBoxW(std::ptr::null_mut(),
					msg.encode_utf16().chain(vec![0u16]).collect::<Vec<_>>().as_ptr(),
					title.encode_utf16().chain(vec![0u16]).collect::<Vec<_>>().as_ptr(),
					MB_ICONERROR);
	}
}
