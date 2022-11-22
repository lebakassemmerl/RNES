use crate::cfn_assert;

pub struct Color {
	r: u8,
	g: u8,
	b: u8,
}

impl Color {
	pub const fn from_hex_24bit(hex: u32) -> Self {
		cfn_assert!(hex <= 0xFFFFFF);

		Self {
			r: ((hex >> 16) & 0xFF) as u8,
			g: ((hex >> 8) & 0xFF) as u8,
			b: (hex & 0xFF) as u8,
		}
	}

	pub const fn from_hex(hex: u32) -> Self {
		Self {
			r: ((hex >> 24) & 0xFF) as u8,
			g: ((hex >> 16) & 0xFF) as u8,
			b: ((hex >> 8) & 0xFF) as u8,
		}
	}

	pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
		Self {
			r,
			g,
			b,
		}
	}

	pub const fn from_tuple(rgb: (u8, u8, u8)) -> Self {
		Self {
			r: rgb.0,
			g: rgb.1,
			b: rgb.2,
		}
	}

	pub const fn r(&self) -> u8 {
		self.r
	}

	pub const fn g(&self) -> u8 {
		self.g
	}

	pub const fn b(&self) -> u8 {
		self.b
	}
}

impl From<&Color> for u32 {
	fn from(col: &Color) -> Self {
		// set the LSB always to 0xFF in order to make the colors bitmap compatible
		let mut ret = 0xFFu32;
		ret |= (col.r as u32) << 24;
		ret |= (col.g as u32) << 16;
		ret |= (col.b as u32) << 8;

		ret
	}
}

impl From<Color> for u32 {
	fn from(col: Color) -> Self {
		// set the LSB always to 0xFF in order to make the colors bitmap compatible
		let mut ret = 0xFFu32;
		ret |= (col.r as u32) << 24;
		ret |= (col.g as u32) << 16;
		ret |= (col.b as u32) << 8;

		ret
	}
}

impl From<u32> for Color {
	fn from(col: u32) -> Self {
		Self {
			r: ((col >> 16) & 0xFF) as u8,
			g: ((col >> 8) & 0xFF) as u8,
			b: (col & 0xFF) as u8,
		}
	}
}

impl Default for Color {
	fn default() -> Self {
		Self::from_hex(0)
	}
}

macro_rules! c {
	($rgb:expr) => {
		Color::from_hex_24bit($rgb)
	};
}

pub const COLOR_PALETTE_SIZE: usize = 64;

#[rustfmt::skip]
pub const COLOR_PALETTE: &[Color; COLOR_PALETTE_SIZE] = &[
	c!(0x666666), c!(0x002A88), c!(0x1412A7), c!(0x3B00A4),
	c!(0x5C007E), c!(0x6E0040), c!(0x6C0600), c!(0x561D00),
	c!(0x333500), c!(0x0B4800), c!(0x005200), c!(0x004F08),
	c!(0x00404D), c!(0x000000), c!(0x000000), c!(0x000000),
	c!(0xADADAD), c!(0x155FD9), c!(0x4240FF), c!(0x7527FE),
	c!(0xA01ACC), c!(0xB71E7B), c!(0xB53120), c!(0x994E00),
	c!(0x6B6D00), c!(0x388700), c!(0x0C9300), c!(0x008F32),
	c!(0x007C8D), c!(0x000000), c!(0x000000), c!(0x000000),
	c!(0xFFFEFF), c!(0x64B0FF), c!(0x9290FF), c!(0xC676FF),
	c!(0xF36AFF), c!(0xFE6ECC), c!(0xFE8170), c!(0xEA9E22),
	c!(0xBCBE00), c!(0x88D800), c!(0x5CE430), c!(0x45E082),
	c!(0x48CDDE), c!(0x4F4F4F), c!(0x000000), c!(0x000000),
	c!(0xFFFEFF), c!(0xC0DFFF), c!(0xD3D2FF), c!(0xE8C8FF),
	c!(0xFBC2FF), c!(0xFEC4EA), c!(0xFECCC5), c!(0xF7D8A5),
	c!(0xE4E594), c!(0xCFEF96), c!(0xBDF4AB), c!(0xB3F3CC),
	c!(0xB5EBf2), c!(0xB8B8B8), c!(0x000000), c!(0x000000),
];
