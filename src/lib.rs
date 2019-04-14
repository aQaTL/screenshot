mod screenshot;

use image::{GenericImageView, ImageFormat};

#[repr(C)]
pub struct RawVec {
	pub ptr: *mut u8,
	pub len: usize,
	pub cap: usize,
}

#[no_mangle]
pub extern fn screenshot_bitmap() -> RawVec {
	return match screenshot::screenshot(None) {
		Ok(mut bmp) => {
			let slice = RawVec {
				ptr: bmp.as_mut_ptr(),
				len: bmp.len(),
				cap: bmp.capacity(),
			};
			std::mem::forget(bmp);
			slice
		}
		Err(msg) => {
			panic!(msg);
		}
	};
}

#[no_mangle]
pub extern fn free_vec(vec: RawVec) {
	let _ = unsafe { Vec::from_raw_parts(vec.ptr, vec.len, vec.cap) };
}

#[no_mangle]
pub extern fn encode_img_as_jpg(img: &[u8]) -> RawVec {
	let img = image::load_from_memory_with_format(img, ImageFormat::BMP).unwrap();
	let mut buf = Vec::new();
	let mut jpeg_enc = image::jpeg::JPEGEncoder::new_with_quality(&mut buf, 90);
	jpeg_enc.encode(img.raw_pixels().as_slice(), img.width(), img.height(), img.color()).unwrap();

	let vec = RawVec {
		ptr: buf.as_mut_ptr(),
		len: buf.len(),
		cap: buf.capacity(),
	};
	std::mem::forget(buf);
	vec
}
