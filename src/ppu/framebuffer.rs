use std::sync::Arc;

use crate::ppu::color::Color;

// TODO: get rid of unsafe but do not use locks to keep the performance

pub struct FrameBuffer {
	width: usize,
	height: usize,
	// fb: Arc<RwLock<Vec<u8>>>,
	fb: Arc<Vec<u8>>,
}

impl FrameBuffer {
	pub fn new(width_px: usize, height_px: usize) -> Self {
		Self {
			width: width_px,
			height: height_px,
			// RGB, every pixel needs 3 bytes
			// fb: Arc::new(RwLock::new(vec![0x00; width_px * height_px * 3])),
			fb: Arc::new(vec![0x00; width_px * height_px * 3]),
		}
	}

	pub fn render_pixel(&mut self, idx_x: usize, idx_y: usize, color: &Color) {
		assert!(idx_x < self.width, "FB: idx_x out of range: {}", idx_x);
		assert!(idx_y < self.height, "FB: idx_y out of range: {}", idx_y);

		let idx = 3 * (idx_y * self.width + idx_x);

		let fb_ptr = self.fb.as_ptr() as *mut u8;
		let fb_raw = unsafe { std::slice::from_raw_parts_mut(fb_ptr, self.fb.len()) };

		{
			fb_raw[idx + 0] = color.r();
			fb_raw[idx + 1] = color.g();
			fb_raw[idx + 2] = color.b();
		}
	}

	pub fn fb(&self) -> Arc<Vec<u8>> {
		self.fb.clone()
	}
}
