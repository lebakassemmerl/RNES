use crate::mask;
use crate::mem::PpuRegisterAccess;

trait BasicRegisterOps {
	fn set(&mut self, val: u8);
	fn get(&self) -> u8;
	fn set_bit(&mut self, bit: u8);
	fn clear_bit(&mut self, bit: u8);
	fn test_bit(&self, bit: u8) -> bool;
}

#[derive(Default, Copy, Clone)]
pub(crate) struct PpuCtrl(u8);

#[derive(Default, Copy, Clone)]
pub(crate) struct PpuMask(u8);

#[derive(Default, Copy, Clone)]
pub(crate) struct PpuStatus(u8);

#[derive(Copy, Clone)]
pub(crate) struct LoopyRegister(u16);

#[derive(Default, Copy, Clone)]
pub(crate) struct Oam {
	pub(crate) addr: u8,
	pub(crate) data: u8,
	pub(crate) write_stb: bool,
}

pub struct PpuRegisters {
	pub(crate) ppu_ctrl: PpuCtrl,
	pub(crate) ppu_mask: PpuMask,
	pub(crate) ppu_status: PpuStatus,
	pub(crate) oam: Oam,

	w: bool,
	pub(crate) t: LoopyRegister,
	pub(crate) v: LoopyRegister,
	pub(crate) x: u8,

	ppudata_buf: u8,
}

macro_rules! ppu_basic_regops {
	($ppureg:ident) => {
		impl BasicRegisterOps for $ppureg {
			fn set(&mut self, val: u8) {
				self.0 = val;
			}
			fn get(&self) -> u8 {
				self.0
			}
			fn set_bit(&mut self, bit: u8) {
				assert!(
					bit < 8,
					concat!(stringify!($ppureg), "::set_bit(): bit index out of range: {}"),
					bit
				);

				self.0 |= 1 << bit;
			}
			fn clear_bit(&mut self, bit: u8) {
				assert!(
					bit < 8,
					concat!(stringify!($ppureg), "::clear_bit(): bit index out of range: {}"),
					bit
				);

				self.0 &= !(1 << bit);
			}
			fn test_bit(&self, bit: u8) -> bool {
				assert!(
					bit < 8,
					concat!(stringify!($ppureg), "::test_bit(): bit index out of range: {}"),
					bit
				);

				(self.0 & (1 << bit)) > 0
			}
		}
	};
}

ppu_basic_regops!(PpuCtrl);
ppu_basic_regops!(PpuMask);
ppu_basic_regops!(PpuStatus);

use LoopyRegister as LR;

impl LoopyRegister {
	const COARSE_BITS: usize = 5;
	const FINE_Y_BITS: usize = 3;
	const NT_BITS: usize = 2;

	const COARSE_X_IDX: usize = 0;
	const COARSE_Y_IDX: usize = 5;
	const NT_IDX: usize = 10;
	const FINE_Y_IDX: usize = 12;
	const NT_X_IDX: usize = LR::NT_IDX;
	const NT_Y_IDX: usize = LR::NT_IDX + 1;

	const NT_MASK: u16 = mask!(u16, LR::NT_BITS, LR::NT_IDX, false);
	const NT_X_MASK: u16 = mask!(u16, 1, LR::NT_X_IDX, false);
	const NT_Y_MASK: u16 = mask!(u16, 1, LR::NT_Y_IDX, false);
	const COARSE_X_MASK: u16 = mask!(u16, LR::COARSE_BITS, LR::COARSE_X_IDX, false);
	const COARSE_Y_MASK: u16 = mask!(u16, LR::COARSE_BITS, LR::COARSE_Y_IDX, false);
	const FINE_Y_MASK: u16 = mask!(u16, LR::FINE_Y_BITS, LR::FINE_Y_IDX, false);

	const fn new() -> Self {
		Self(0)
	}

	pub(crate) fn get(&self) -> u16 {
		self.0
	}

	pub(crate) fn transfer_x(&mut self, other: &LoopyRegister) {
		self.set_coarse_x(other.get_coarse_x());

		let nt = other.get_nt_x();
		self.0 &= !LR::NT_X_MASK;
		self.0 |= nt << LR::NT_X_IDX;
	}

	pub(crate) fn transfer_y(&mut self, other: &LoopyRegister) {
		self.set_coarse_y(other.get_coarse_y());
		self.set_fine_y(other.get_fine_y());

		let nt = other.get_nt_y();
		self.0 &= !LR::NT_Y_MASK;
		self.0 |= nt << LR::NT_Y_IDX;
	}

	pub(crate) fn inc_x(&mut self) {
		if (self.0 & LR::COARSE_X_MASK) == LR::COARSE_X_MASK {
			self.0 ^= LR::NT_X_MASK; // toggle horizontal nametable
			self.0 &= !LR::COARSE_X_MASK; // set coarse x to 0
		} else {
			self.0 += 1;
		}
	}

	pub(crate) fn inc_y(&mut self) {
		const FINE_Y_INC: u16 = mask!(u16, 1, LR::FINE_Y_IDX, false);
		const COARSE_Y_MAX: u8 = mask!(u8, 1, LR::COARSE_Y_IDX, false);
		const COARSE_Y_SWITCH_NT: u8 = 29;

		if (self.0 & LR::FINE_Y_MASK) != LR::FINE_Y_MASK {
			self.0 += FINE_Y_INC; // inc fine-y by 1
		} else {
			self.0 &= !LR::FINE_Y_MASK; // set fine-y to 0

			let mut coarse_y = self.get_coarse_y();
			if coarse_y == COARSE_Y_SWITCH_NT {
				coarse_y = 0; // set coarse-y to 0
				self.0 ^= LR::NT_Y_MASK; // toggle vertical nametable
			} else if coarse_y == COARSE_Y_MAX {
				coarse_y = 0;
			} else {
				coarse_y += 1;
			}

			self.set_coarse_y(coarse_y);
		}
	}

	pub(crate) fn get_nametable_addr(&self) -> u16 {
		const NT_ADDR_BASE: u16 = 0x2000;
		const NT_ADDR_MASK: u16 = mask!(u16, 12, 0, false);

		NT_ADDR_BASE | (self.0 & NT_ADDR_MASK)
	}

	pub(crate) fn get_attribute_addr(&self) -> u16 {
		const ATTR_ADDR_BASE: u16 = 0x23C0;
		const ADDR_X_IDX: usize = 0;
		const ADDR_Y_IDX: usize = 3;

		let x = (self.get_coarse_x() >> 2) as u16;
		let y = (self.get_coarse_y() >> 2) as u16;
		let nt = self.get_nt() as u16;

		let mut addr = ATTR_ADDR_BASE | (nt << LR::NT_IDX); // choose the correct nametable
		addr |= y << ADDR_Y_IDX; // add (coarse-y / 4) to the address
		addr |= x << ADDR_X_IDX; // add (coarse-x / 4) to the address

		addr
	}

	fn set_nt(&mut self, nt: u8) {
		const SRC_MASK: u8 = mask!(u8, LR::NT_BITS, 0, false);

		self.0 &= !LR::NT_MASK;
		self.0 |= ((nt & SRC_MASK) as u16) << LR::NT_IDX;
	}

	pub(crate) fn get_nt(&self) -> u8 {
		((self.0 & LR::NT_MASK) >> LR::NT_IDX) as u8
	}

	fn get_nt_x(&self) -> u16 {
		((self.0 & LR::NT_X_MASK) > 0) as u16
	}

	fn get_nt_y(&self) -> u16 {
		((self.0 & LR::NT_Y_MASK) > 0) as u16
	}

	pub(crate) fn set_coarse_x(&mut self, x: u8) {
		const SRC_MASK: u8 = mask!(u8, LR::COARSE_BITS, 0, false);

		self.0 &= !LR::COARSE_X_MASK;
		self.0 |= ((x & SRC_MASK) as u16) << LR::COARSE_X_IDX;
	}

	pub(crate) fn get_coarse_x(&self) -> u8 {
		((self.0 & LR::COARSE_X_MASK) >> LR::COARSE_X_IDX) as u8
	}

	fn set_coarse_y(&mut self, y: u8) {
		const SRC_MASK: u8 = mask!(u8, LR::COARSE_BITS, 0, false);

		self.0 &= !LR::COARSE_Y_MASK;
		self.0 |= ((y & SRC_MASK) as u16) << LR::COARSE_Y_IDX;
	}

	pub(crate) fn get_coarse_y(&self) -> u8 {
		((self.0 & LR::COARSE_Y_MASK) >> LR::COARSE_Y_IDX) as u8
	}

	fn set_fine_y(&mut self, y: u8) {
		const SRC_MASK: u8 = mask!(u8, LR::FINE_Y_BITS, 0, false);

		self.0 &= !LR::FINE_Y_MASK;
		self.0 |= ((y & SRC_MASK) as u16) << LR::FINE_Y_IDX;
	}

	pub(crate) fn get_fine_y(&self) -> u8 {
		((self.0 & LR::FINE_Y_MASK) >> LR::FINE_Y_IDX) as u8
	}

	fn set_addr_w0(&mut self, addr: u8) {
		const W0_BITS: usize = 6;
		const W0_IDX: usize = 8;
		const W0_MASK_DST: u16 = mask!(u16, W0_BITS + 1, W0_IDX, true);
		const W0_MASK_SRC: u8 = mask!(u8, W0_BITS, 0, false);

		self.0 &= W0_MASK_DST;
		self.0 |= ((addr & W0_MASK_SRC) as u16) << W0_IDX;
	}

	fn set_addr_w1(&mut self, addr: u8) {
		const W1_BITS: usize = 8;
		const W1_IDX: usize = 0;
		const W1_MASK: u16 = mask!(u16, W1_BITS, W1_IDX, true);

		self.0 &= W1_MASK;
		self.0 |= addr as u16;
	}

	fn inc(&mut self, inc: u16) {
		// stupid incrementing, used when CPU accesses the PPUPDATA register
		self.0 = self.0.wrapping_add(inc);
	}
}

impl PpuCtrl {
	const INC_IDX: u8 = 2;
	const SPRITES_TABLE_IDX: u8 = 3;
	const BACKGROUND_TABLE_IDX: u8 = 4;
	const SPRITE_SIZE: u8 = 5;
	const ENABLE_NMI_IDX: u8 = 7;

	const PATTERN_TABLE_SIZE: u16 = 0x1000;

	fn get_inc(&self) -> u16 {
		if self.test_bit(Self::INC_IDX) {
			32
		} else {
			1
		}
	}

	pub(crate) fn background_tile_base(&self) -> u16 {
		(self.test_bit(Self::BACKGROUND_TABLE_IDX)) as u16 * Self::PATTERN_TABLE_SIZE
	}

	pub(crate) fn sprites_tile_base(&self) -> u16 {
		(self.test_bit(Self::SPRITES_TABLE_IDX)) as u16 * Self::PATTERN_TABLE_SIZE
	}

	pub(crate) fn nmi_enabled(&self) -> bool {
		self.test_bit(Self::ENABLE_NMI_IDX)
	}

	pub(crate) fn sprite_size(&self) -> u8 {
		if self.test_bit(Self::SPRITE_SIZE) {
			16
		} else {
			8
		}
	}
}

impl PpuMask {
	const GREY_SCALE_IDX: u8 = 0;
	const RENDER_LEFT_BACKGROUND_IDX: u8 = 1;
	const RENDER_LEFT_SPRITES_IDX: u8 = 2;
	const RENDER_BACKGROUND_IDX: u8 = 3;
	const RENDER_SPRITES_IDX: u8 = 4;

	pub(crate) fn greyscale(&self) -> bool {
		self.test_bit(Self::GREY_SCALE_IDX)
	}

	pub(crate) fn render_background(&self) -> bool {
		self.test_bit(Self::RENDER_BACKGROUND_IDX)
	}

	pub(crate) fn render_sprites(&self) -> bool {
		self.test_bit(Self::RENDER_SPRITES_IDX)
	}

	pub(crate) fn render_leftmost_background(&self) -> bool {
		self.test_bit(Self::RENDER_LEFT_BACKGROUND_IDX)
	}

	pub(crate) fn render_leftmost_sprites(&self) -> bool {
		self.test_bit(Self::RENDER_LEFT_SPRITES_IDX)
	}
}

impl PpuStatus {
	const SPRITE_OVFL: u8 = 5;
	const SPRITE0_HIT: u8 = 6;
	const VBLANK_IDX: u8 = 7;

	pub(crate) fn clear_sprite_overflow(&mut self) {
		self.clear_bit(Self::SPRITE_OVFL);
	}

	pub(crate) fn set_sprite_overflow(&mut self) {
		self.set_bit(Self::SPRITE_OVFL);
	}

	pub(crate) fn clear_sprite0_hit(&mut self) {
		self.clear_bit(Self::SPRITE0_HIT);
	}

	pub(crate) fn set_sprite0_hit(&mut self) {
		self.set_bit(Self::SPRITE0_HIT);
	}

	pub(crate) fn get_sprite0_hit(&self) -> bool {
		self.test_bit(Self::SPRITE0_HIT)
	}

	pub(crate) fn clear_vblank(&mut self) {
		self.clear_bit(Self::VBLANK_IDX);
	}
	pub(crate) fn set_vblank(&mut self) {
		self.set_bit(Self::VBLANK_IDX);
	}
}

impl PpuRegisters {
	pub fn new() -> Self {
		Self {
			ppu_ctrl: Default::default(),
			ppu_mask: Default::default(),
			ppu_status: Default::default(),
			oam: Default::default(),

			t: LoopyRegister::new(),
			v: LoopyRegister::new(),
			x: 0,
			w: false,
			ppudata_buf: 0,
		}
	}

	fn inc_ppu_addr(&mut self) {
		self.v.inc(self.ppu_ctrl.get_inc());
	}
}

impl PpuRegisterAccess for PpuRegisters {
	fn ppu_ctrl_write(&mut self, val: u8) {
		self.ppu_ctrl.set(val);
		self.t.set_nt(val & 0x03);
	}

	fn ppu_mask_write(&mut self, val: u8) {
		self.ppu_mask.set(val);
	}

	fn ppu_scroll_write(&mut self, val: u8) {
		const FINE_BITS: u8 = 3;
		const FINE_MASK: u8 = (1 << FINE_BITS) - 1;

		if !self.w {
			self.x = val & FINE_MASK;
			self.t.set_coarse_x(val >> FINE_BITS);
		} else {
			self.t.set_coarse_y(val >> FINE_BITS);
			self.t.set_fine_y(val & FINE_MASK);
		}

		self.w = !self.w;
	}

	fn ppu_addr_write(&mut self, val: u8) {
		if !self.w {
			self.t.set_addr_w0(val);
		} else {
			self.t.set_addr_w1(val);
			self.v = self.t;
		}

		self.w = !self.w
	}

	fn ppu_data_write(&mut self) {
		self.inc_ppu_addr();
	}

	fn ppu_stat_read(&mut self) -> u8 {
		let ret = self.ppu_status.get();

		self.ppu_status.clear_vblank();
		self.w = false;

		ret
	}

	fn ppu_addr_get(&self) -> u16 {
		self.v.get()
	}

	fn ppu_data_read(&mut self, new_val: u8) -> u8 {
		let ret = if self.v.get() < 0x3F00 {
			let r = self.ppudata_buf;
			self.ppudata_buf = new_val;

			r
		} else {
			new_val
		};

		self.inc_ppu_addr();

		ret
	}

	fn oam_data_read(&self) -> u8 {
		self.oam.data
	}

	fn oam_addr_write(&mut self, val: u8) {
		self.oam.addr = val;
	}

	fn oam_data_write(&mut self, val: u8) {
		self.oam.data = val;
		self.oam.write_stb = true;
	}
}
