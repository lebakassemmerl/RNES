use std::sync::{Arc, RwLock};

use crate::ppu::color::Color;

pub struct FrameBuffer {
	width: usize,
	height: usize,
	fb: Arc<RwLock<Vec<u8>>>,
}

impl FrameBuffer {
	pub fn new(width_px: usize, height_px: usize) -> Self {
		Self {
			width: width_px,
			height: height_px,
			// RGB, every pixel needs 3 bytes
			fb: Arc::new(RwLock::new(vec![0x00; width_px * height_px * 3])),
		}
	}

	pub fn render_pixel(&mut self, idx_x: usize, idx_y: usize, color: &Color) {
		assert!(idx_x < self.width, "FB: idx_x out of range: {}", idx_x);
		assert!(idx_y < self.height, "FB: idx_y out of range: {}", idx_y);

		let idx = 3 * (idx_y * self.width + idx_x);

		{
			let mut fb = self.fb.write().unwrap();
			fb[idx + 0] = color.r();
			fb[idx + 1] = color.g();
			fb[idx + 2] = color.b();
		}
	}

	pub fn fb(&self) -> Arc<RwLock<Vec<u8>>> {
		self.fb.clone()
	}
}
