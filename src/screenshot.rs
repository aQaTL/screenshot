use std::ptr::null_mut;
use winapi::um::wingdi::*;
use winapi::ctypes::c_void;
use winapi::um::winuser::{GetDC, ReleaseDC};
use winapi::shared::windef::HDC;
use std::mem::size_of;

pub struct Point {
	pub x: i32,
	pub y: i32,
}

pub struct Rect {
	pub x: Point,
	pub y: Point,
}

type WinApiError = &'static str;

pub fn screenshot(rect: Option<Rect>) -> Result<Vec<u8>, WinApiError> { unsafe { screenshot_(rect) } }

unsafe fn screenshot_(rect: Option<Rect>) -> Result<Vec<u8>, WinApiError> {
	let h_screen = GetDC(null_mut());
	let h_mem = CreateCompatibleDC(h_screen);

	if h_mem as i32 == 0 {
		return Err("CreateCompatibleDC failed");
	}

	let (a, b) = match rect {
		Some(r) => (r.x, r.y),
		None => {
			let r = screen_res(h_screen);
			(r.x, r.y)
		}
	};

	let h_bitmap = CreateCompatibleBitmap(h_screen, (b.x - a.x).abs(), (b.y - a.y).abs());
	let old_obj = SelectObject(h_mem, h_bitmap as *mut c_void);
	let result = BitBlt(h_mem, 0, 0, (b.x - a.x).abs(), (b.y - a.y).abs(), h_screen, a.x, a.y, SRCCOPY);
	if result == 0 {
		return Err("BitBlt failed");
	}

	let mut bmp = BITMAP { bmType: 0, bmWidth: 0, bmHeight: 0, bmWidthBytes: 0, bmPlanes: 0, bmBitsPixel: 0, bmBits: null_mut() };
	let result = GetObjectW(h_bitmap as *mut c_void, size_of::<BITMAP>() as i32, &mut bmp as *mut BITMAP as *mut c_void);
	if result == 0 {
		return Err("GetObjectW failed");
	}
	let (bmfh, mut bmi) = create_bitmap_headers(bmp);

	let mut pixels_buffer = vec![0u8; bmfh.bfSize as usize];
	let bmfh_bytes = any_as_u8_slice(&bmfh);
	let bmi_bytes = any_as_u8_slice(&bmi);
	pixels_buffer[0..bmfh_bytes.len()].copy_from_slice(bmfh_bytes);
	pixels_buffer[bmfh_bytes.len()..bmfh_bytes.len() + bmi_bytes.len()].copy_from_slice(bmi_bytes);

	let result = GetDIBits(h_screen, h_bitmap,
						   0, bmi.bmiHeader.biHeight as u32,
						   pixels_buffer.as_mut_ptr().offset(bmfh.bfOffBits as isize) as *mut c_void,
						   &mut bmi as *mut BITMAPINFO,
						   DIB_RGB_COLORS);
	if result == 0 {
		return Err("GetDIBits returned 0");
	}

	SelectObject(h_mem, old_obj);
	DeleteDC(h_mem);
	ReleaseDC(null_mut(), h_screen);
	DeleteObject(h_bitmap as *mut c_void);

	Ok(pixels_buffer)
}

fn create_bitmap_headers(bmp: BITMAP) -> (BITMAPFILEHEADER, BITMAPINFO) {
	let c_clr_bits: u32 = match bmp.bmPlanes * bmp.bmBitsPixel {
		1 => 1,
		n if n <= 4 => 4,
		n if n <= 8 => 8,
		n if n <= 16 => 16,
		n if n <= 24 => 24,
		_ => 32,
	};

	let bmi = BITMAPINFO {
		bmiHeader: BITMAPINFOHEADER {
			biSize: size_of::<BITMAPINFOHEADER>() as u32,
			biWidth: bmp.bmWidth,
			biHeight: bmp.bmHeight,
			biPlanes: bmp.bmPlanes,
			biBitCount: bmp.bmBitsPixel,
			biCompression: BI_RGB,
			biSizeImage: (((bmp.bmWidth as u32 * c_clr_bits + 31u32) & !31) as f64 / 8.0f64 * bmp.bmHeight as f64) as u32,
			biXPelsPerMeter: 0,
			biYPelsPerMeter: 0,
			biClrUsed: if c_clr_bits < 24 { 1 << c_clr_bits } else { 0 },
			biClrImportant: 0,
		},
		bmiColors: [RGBQUAD {
			rgbBlue: 0,
			rgbGreen: 0,
			rgbRed: 0,
			rgbReserved: 0,
		}],
	};
	let bmfh = BITMAPFILEHEADER {
		bfType: 0x4D42, //bitmap
		bfSize: size_of::<BITMAPFILEHEADER>() as u32 + bmi.bmiHeader.biSize +
			bmi.bmiHeader.biClrUsed * size_of::<RGBQUAD>() as u32 + bmi.bmiHeader.biSizeImage,
		bfReserved1: 0,
		bfReserved2: 0,
		bfOffBits: size_of::<BITMAPFILEHEADER>() as u32 + bmi.bmiHeader.biSize +
			bmi.bmiHeader.biClrUsed * size_of::<RGBQUAD>() as u32,
	};
	(bmfh, bmi)
}

unsafe fn screen_res(screen_dc: HDC) -> Rect {
	Rect {
		x: Point { x: 0, y: 0 },
		y: Point { x: GetDeviceCaps(screen_dc, HORZRES), y: GetDeviceCaps(screen_dc, VERTRES) },
	}
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
	std::slice::from_raw_parts(p as *const T as *const u8, std::mem::size_of::<T>())
}
