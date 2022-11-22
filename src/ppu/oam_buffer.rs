use crate::mem::PpuBus;
use crate::util::bit::{reverse_u8, test_bit};

use super::shiftreg::ShiftReg8;

pub(crate) struct ComposedSprite {
	pub(crate) sprite0: bool,
	pub(crate) pix: u8,
	pub(crate) pal: u8,
	pub(crate) bg_prio: bool,
}

pub(crate) struct Sprite {
	y: u8,
	x: u8,
	idx: u8,
	attr: u8,

	tile_msb: ShiftReg8,
	tile_lsb: ShiftReg8,

	sprite0: bool,
}

pub(crate) struct OamBuffer {
	pub(crate) sprites_found: usize,

	pub(crate) spr: [Sprite; 8],
}

impl Sprite {
	pub(crate) const fn new() -> Self {
		Self {
			y: 0xFF,
			x: 0xFF,
			idx: 0xFF,
			attr: 0xFF,

			tile_lsb: ShiftReg8::new(0),
			tile_msb: ShiftReg8::new(0),

			sprite0: false,
		}
	}

	pub(crate) fn reset(&mut self) {
		self.y = 0xFF;
		self.x = 0xFF;
		self.idx = 0xFF;
		self.attr = 0xFF;

		self.tile_lsb.reload(0);
		self.tile_msb.reload(0);

		self.sprite0 = false;
	}

	pub(crate) fn set_x(&mut self, val: u8) {
		self.x = val;
	}

	pub(crate) fn set_y(&mut self, val: u8) {
		self.y = val;
	}

	pub(crate) fn set_attr(&mut self, val: u8) {
		self.attr = val;
	}

	pub(crate) fn set_idx(&mut self, val: u8) {
		self.idx = val;
	}

	pub(crate) fn set_sprite0(&mut self) {
		self.sprite0 = true;
	}

	pub(crate) fn flip_verticaly(&self) -> bool {
		test_bit::<u8>(&self.attr, 7)
	}

	pub(crate) fn flip_horizontally(&self) -> bool {
		test_bit::<u8>(&self.attr, 6)
	}

	pub(crate) fn bg_priority(&self) -> bool {
		test_bit::<u8>(&self.attr, 5)
	}

	pub(crate) fn palette(&self) -> u8 {
		const SPRITE_TABLE_BASE: u8 = 0x04;
		(self.attr & 0x03) | SPRITE_TABLE_BASE
	}
}

impl OamBuffer {
	pub(crate) const fn new() -> Self {
		Self {
			sprites_found: 0,

			spr: [
				Sprite::new(),
				Sprite::new(),
				Sprite::new(),
				Sprite::new(),
				Sprite::new(),
				Sprite::new(),
				Sprite::new(),
				Sprite::new(),
			],
		}
	}

	pub(crate) fn reset(&mut self) {
		self.sprites_found = 0;

		for s in self.spr.iter_mut() {
			s.reset();
		}
	}

	// NOTE: has to be called before shift() was called
	pub(crate) fn load_sprite_shifter<B: PpuBus>(
		&mut self,
		bus: &mut B,
		scanline: usize,
		pt_tbl: u16,
		spr_8x16: bool,
	) {
		for s in self.spr[..self.sprites_found].iter_mut() {
			let mut y_idx = (scanline as u16) - (s.y as u16);

			let pt = if spr_8x16 {
				((s.idx & 0x01) as u16) * 0x1000
			} else {
				pt_tbl
			};

			let tile = if spr_8x16 {
				if s.flip_verticaly() {
					y_idx = 15 - y_idx;
				}

				// if y_idx is greater than 8, we have to add 1 to the tile and subtract 8 from the
				// y-index
				let offset = if y_idx >= 8 {
					y_idx -= 8;
					1
				} else {
					0
				};

				(((s.idx & 0xFE) as u16) + offset) << 4
			} else {
				if s.flip_verticaly() {
					y_idx = 7 - y_idx;
				}

				(s.idx as u16) << 4
			};

			let tile_addr = pt | tile | y_idx;

			let mut spr_lsb = bus.read(tile_addr as usize);
			let mut spr_msb = bus.read(tile_addr.wrapping_add(8) as usize);

			if s.flip_horizontally() {
				spr_lsb = reverse_u8(spr_lsb);
				spr_msb = reverse_u8(spr_msb);
			}

			s.tile_lsb.reload(spr_lsb);
			s.tile_msb.reload(spr_msb);
		}
	}

	pub(crate) fn shift(&mut self) {
		for s in self.spr[..self.sprites_found].iter_mut() {
			if s.x > 0 {
				s.x -= 1;
			} else {
				s.tile_lsb.shl();
				s.tile_msb.shl();
			}
		}
	}

	pub(crate) fn evaluate_sprite(&self) -> Option<ComposedSprite> {
		for s in self.spr.iter() {
			if s.x > 0 {
				continue;
			}

			let pix = (s.tile_msb.get_bit() << 1) | s.tile_lsb.get_bit();
			if pix > 0 {
				// we got a pixel which is not transparent
				return Some(ComposedSprite {
					sprite0: s.sprite0,
					pix,
					pal: s.palette(),
					bg_prio: s.bg_priority(),
				});
			}
		}

		None
	}
}
