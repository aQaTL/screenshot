#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod screenshot;

use crate::screenshot::*;

use winapi::um::winuser::{MessageBoxW, MB_ICONERROR};
use image::{GenericImageView, ImageFormat};

fn main() {
	let bmp = match screenshot(None) {
		Ok(bmp) => bmp,
		Err(msg) => {
			message_box_error("Windows error", msg);
			panic!(msg);
		}
	};
	let img = image::load_from_memory_with_format(bmp.as_slice(), ImageFormat::BMP).unwrap();
	let mut buf = Vec::new();
	let mut jpeg_enc = image::jpeg::JPEGEncoder::new_with_quality(&mut buf, 90);
	jpeg_enc.encode(img.raw_pixels().as_slice(), img.width(), img.height(), img.color()).unwrap();

	img.save("screenshot.jpg").unwrap();
}

pub fn message_box_error(title: &str, msg: &str) {
	unsafe {
		MessageBoxW(std::ptr::null_mut(),
					msg.encode_utf16().chain(vec![0u16]).collect::<Vec<_>>().as_ptr(),
					title.encode_utf16().chain(vec![0u16]).collect::<Vec<_>>().as_ptr(),
					MB_ICONERROR);
	}
}
