use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use super::color::{Color, COLOR_PALETTE};
use super::framebuffer::FrameBuffer;
use super::oam_buffer::OamBuffer;
use super::ppu_regs::{LoopyRegister, PpuCtrl, PpuMask};
use super::shiftreg::ShiftReg16;
use crate::mask;
use crate::mem::{PpuBus, Segment};

const MAX_SPRITE_CNT: usize = 64;

pub struct Ppu<B: PpuBus> {
	cycle: usize,
	scanline: usize,

	bg_next_tile_addr: u8,
	bg_next_attr: u8,
	bg_next_tile_msb: u8,
	bg_next_tile_lsb: u8,
	bg_tile_lsb: ShiftReg16,
	bg_tile_msb: ShiftReg16,
	bg_attr_lsb: ShiftReg16,
	bg_attr_msb: ShiftReg16,

	oam_buf: OamBuffer,

	fb: FrameBuffer,
	tile_fb: FrameBuffer,
	frame_finished: bool,
	fb_ready: bool,
	cycle_cnt: usize, // for statistics
	_phantom: PhantomData<B>,
}

impl<B: PpuBus> Ppu<B> {
	pub fn new() -> Self {
		// TODO: handle NTSC and PAL differently, for now we default to NTSC
		Self {
			cycle: 0,
			scanline: 0,

			bg_next_tile_addr: 0,
			bg_next_attr: 0,
			bg_next_tile_msb: 0,
			bg_next_tile_lsb: 0,
			bg_tile_lsb: ShiftReg16::new(),
			bg_tile_msb: ShiftReg16::new(),
			bg_attr_lsb: ShiftReg16::new(),
			bg_attr_msb: ShiftReg16::new(),

			oam_buf: OamBuffer::new(),

			fb: FrameBuffer::new(256, 240),
			tile_fb: FrameBuffer::new(256, 128),
			fb_ready: true,
			frame_finished: false,
			cycle_cnt: 0,

			_phantom: PhantomData,
		}
	}

	fn bg_shift(&mut self) {
		self.bg_tile_lsb.shl();
		self.bg_tile_msb.shl();
		self.bg_attr_lsb.shl();
		self.bg_attr_msb.shl();
	}

	fn bg_shifter_reload(&mut self) {
		self.bg_tile_lsb.reload(self.bg_next_tile_lsb);
		self.bg_tile_msb.reload(self.bg_next_tile_msb);
		self.bg_attr_lsb.reload(if (self.bg_next_attr & 0x01) > 0 {
			0xFF
		} else {
			0x00
		});
		self.bg_attr_msb.reload(if (self.bg_next_attr & 0x02) > 0 {
			0xFF
		} else {
			0x00
		});
	}

	fn get_background_address(&self, ctrl: &PpuCtrl, v: &LoopyRegister) -> u16 {
		// multiply addr by 16 since 1 tile needs 16 bytes
		let mut pt_addr = (self.bg_next_tile_addr as u16) << 4;
		pt_addr += ctrl.background_tile_base(); // table 0 or 1
		pt_addr += v.get_fine_y() as u16; // choose the correct byte out of all 16
		pt_addr
	}

	fn get_color_from_palette(
		palette: u8,
		pixel: u8,
		mask: &PpuMask,
		mem: &mut B,
	) -> &'static Color {
		const PALETTE_BASE: usize = 0x3F00;
		const GREYSCALE_MASK: u8 = 0x30;

		// 1 palette needs 4 bytes -> 2x leftshift
		let addr = PALETTE_BASE | (((palette << 2) | pixel) as usize);
		let mut color = mem.read(addr);

		if mask.greyscale() {
			color &= GREYSCALE_MASK;
		}

		&COLOR_PALETTE[color as usize]
	}

	pub fn step(&mut self, mem: &mut B) {
		let f_x = mem.ppu_reg().x;
		let t = mem.ppu_reg().t;
		let mut v = mem.ppu_reg().v;
		let ctrl = mem.ppu_reg().ppu_ctrl;
		let mask = mem.ppu_reg().ppu_mask;
		let mut status = mem.ppu_reg().ppu_status;
		let mut oam_reg = mem.ppu_reg().oam;

		// first, handle the OAM accesses
		if oam_reg.write_stb {
			// data is transferred to the OAM memory
			mem.oam().write(oam_reg.addr as usize, oam_reg.data);
			oam_reg.write_stb = false;
		} else {
			// data is read from the OAM memory
			if self.cycle >= 1 && self.cycle <= 64 {
				// during this time reading from 0x2004 (OAMDATA) returns always 0xFF (hardware)
				oam_reg.data = 0xFF;
			} else {
				oam_reg.data = mem.oam().read(oam_reg.addr as usize);

				if (oam_reg.addr & 0x03) == 0x02 {
					// attribute register
					const UNIMPLEMENTED_MASK: u8 = mask!(u8, 3, 2, true);
					oam_reg.data &= UNIMPLEMENTED_MASK;
				}
			}
		}

		// visible scan-lines -> actual rendering happens here
		if self.scanline == 261 || self.scanline <= 239 {
			if self.cycle == 0 {
				// dummy read of BG LSB always at cycle = 0
				let _ = mem.read(self.get_background_address(&ctrl, &v) as usize);
			} else if self.cycle <= 257 || (self.cycle >= 321 && self.cycle <= 338) {
				if mask.render_background() {
					self.bg_shift();
				}
				// background stuff, for some reason all the stuff starts happening at frame 9

				match (self.cycle - 1) & 0x07 {
					0 => {
						self.bg_shifter_reload();
						let nt = v.get_nametable_addr();
						self.bg_next_tile_addr = mem.read(nt as usize);
					}
					2 => {
						self.bg_next_attr = mem.read(v.get_attribute_addr() as usize);

						if (v.get_coarse_y() & 0x02) > 0 {
							self.bg_next_attr >>= 4; // we are in the bottom half of the tile
						}
						if (v.get_coarse_x() & 0x02) > 0 {
							self.bg_next_attr >>= 2; // we are on the right half of the tile
						}
						self.bg_next_attr &= 0x03; // mask out the unused bits
					}
					4 => {
						let pt_addr = self.get_background_address(&ctrl, &v);
						self.bg_next_tile_lsb = mem.read(pt_addr as usize);
					}
					6 => {
						// 8 byte offset to the lowbyte
						let pt_addr = self.get_background_address(&ctrl, &v).wrapping_add(8);
						self.bg_next_tile_msb = mem.read(pt_addr as usize);
					}
					7 => {
						if mask.render_background() || mask.render_sprites() {
							v.inc_x();
						}
					}
					_ => {}
				}
			}

			if self.cycle == 256 {
				if mask.render_background() || mask.render_sprites() {
					v.inc_y();
				}
			}

			if mask.render_sprites() && self.cycle > 0 && self.cycle <= 257 {
				self.oam_buf.shift();
			}

			if self.cycle == 257 && self.scanline != 261 {
				self.bg_shifter_reload();

				if mask.render_background() || mask.render_sprites() {
					v.transfer_x(&t);
				}

				// evaluate sprites, TODO: do this correctly later
				self.oam_buf.reset();

				for s in 0..MAX_SPRITE_CNT {
					let sprite_height = ctrl.sprite_size() as usize;
					let y = mem.oam().read(s * 4);

					if self.scanline.wrapping_sub(y as usize) < sprite_height {
						let sf = self.oam_buf.sprites_found;
						if sf < 8 {
							self.oam_buf.spr[sf].set_y(y);
							self.oam_buf.spr[sf].set_idx(mem.oam().read(s * 4 + 1));
							self.oam_buf.spr[sf].set_attr(mem.oam().read(s * 4 + 2));
							self.oam_buf.spr[sf].set_x(mem.oam().read(s * 4 + 3));
							if s == 0 {
								self.oam_buf.spr[sf].set_sprite0();
							}

							self.oam_buf.sprites_found += 1;
						} else {
							status.set_sprite_overflow();
							break;
						}
					}
				}
			}

			if self.cycle == 340 {
				// TODO: this is probably the wrong place to do this, but for simplicity do it here
				self.oam_buf.load_sprite_shifter::<B>(
					mem,
					self.scanline,
					ctrl.sprites_tile_base(),
					ctrl.sprite_size() > 8,
				);
			}

			if self.scanline == 261 && (self.cycle >= 280 && self.cycle <= 304) {
				if mask.render_background() || mask.render_sprites() {
					v.transfer_y(&t);
				}
			}

			// obviously some mappers depend on this read
			if self.cycle == 338 || self.cycle == 340 {
				self.bg_next_tile_addr = mem.read(v.get_nametable_addr() as usize);
			}
		}

		if self.scanline == 241 && self.cycle == 1 {
			status.set_vblank();

			if ctrl.nmi_enabled() {
				mem.assert_nmi();
			}

			self.fb_ready = true;
		}

		if self.scanline == 261 && self.cycle == 1 {
			status.clear_vblank();
			status.clear_sprite_overflow();
			status.clear_sprite0_hit();
		}

		// now that we collected all information, we start the actual rendering
		if self.scanline <= 239 && (self.cycle > 0 && self.cycle <= 256) {
			let mut pix: u8 = 0x00;
			let mut pal: u8 = 0x00;

			if mask.render_background() && (mask.render_leftmost_background() || self.cycle > 8) {
				// render the background pixel
				pix = self.bg_tile_lsb.get_bit(f_x) | (self.bg_tile_msb.get_bit(f_x) << 1);
				pal = self.bg_attr_lsb.get_bit(f_x) | (self.bg_attr_msb.get_bit(f_x) << 1);

				// if the pixel value is 0, we fall back to the background color
				if pix == 0 {
					pal = 0;
				}
			}

			if mask.render_sprites() {
				// render sprite pixel

				if let Some(spr) = self.oam_buf.evaluate_sprite() {
					// we found a sprite which we maybe have to render

					if spr.sprite0 && mask.render_background() {
						// check for sprit0 hit, background rendering must be enabled
						if spr.pix > 0 && pix > 0 && !status.get_sprite0_hit() {
							let mut left_border = 0usize;

							// if 1 of these bits is zero, we have to ignore the 1st 8 pixels
							left_border += (!(mask.render_leftmost_background()
								|| mask.render_leftmost_sprites()) as usize)
								* 8;

							if self.cycle > left_border && self.cycle < 255 {
								status.set_sprite0_hit();
							}
						}
					}

					if (!spr.bg_prio || pix == 0)
						&& (mask.render_leftmost_sprites() || self.cycle > 8)
					{
						// The sprite has priority, either due to the priority bit or due to the
						// transparent background. Also it's pix value is greater than 0, otherwise
						// evaluate_sprite() wouldn't had returned it.
						pix = spr.pix;
						pal = spr.pal;
					}
				}
			}

			// if any of these bits is activated, we have to render the pixel
			if mask.render_background() || mask.render_sprites() {
				self.fb.render_pixel(
					self.cycle - 1,
					self.scanline,
					Self::get_color_from_palette(pal, pix, &mask, mem),
				);
			}
		}

		self.cycle += 1;
		self.cycle_cnt += 1;

		if self.cycle == 340 && self.scanline == 261 {
			self.cycle = 1; // skip "odd" cycle
			self.scanline = 0;

			self.frame_finished = true;
		} else if self.cycle > 340 {
			self.cycle = 0;
			self.scanline += 1;
		}

		if self.scanline > 261 {
			self.scanline = 0;
		}

		mem.ppu_reg().v = v;
		mem.ppu_reg().ppu_status = status;
		mem.ppu_reg().oam = oam_reg;
	}

	pub fn frame_finished(&mut self) -> bool {
		let ret = self.frame_finished;
		if ret {
			self.frame_finished = false;
		}

		ret
	}

	pub fn fb_ready(&mut self) -> bool {
		let ret = self.fb_ready;
		if ret {
			self.fb_ready = false;
		}

		ret
	}

	pub fn get_fb(&self) -> Arc<RwLock<Vec<u8>>> {
		self.fb.fb()
	}

	pub fn tile_buf(&mut self, mem: &mut B) -> Arc<RwLock<Vec<u8>>> {
		const COLOR_LOOKUP: [usize; 4] = [0x20, 0x16, 0x2A, 0x11];

		let mut tile_raw: [u8; 16] = [0u8; 16];

		for m in 0..2 {
			for t in 0..256usize {
				for i in 0..16usize {
					tile_raw[i] = PpuBus::read(mem, m * 0x1000 + t * 16 + i);
				}

				for y in 0..8usize {
					for x in 0..8usize {
						let real_x = 7 - x;
						let b0 = (tile_raw[y] & (1 << real_x)) >> real_x;
						let b1 = (tile_raw[y + 8] & (1 << real_x)) >> real_x;
						let col_idx = ((b1 as usize) << 1) | (b0 as usize);

						let idx_x = m * 128 + (t & 0x0F) * 8 + x;
						let idx_y = (t >> 4) * 8 + y;
						self.tile_fb.render_pixel(
							idx_x,
							idx_y,
							&COLOR_PALETTE[COLOR_LOOKUP[col_idx]],
						);
					}
				}
			}
		}

		self.tile_fb.fb()
	}
}
