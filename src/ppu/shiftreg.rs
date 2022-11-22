use crate::mask;

pub(crate) struct ShiftReg8(u8);
pub(crate) struct ShiftReg16(u16);

impl ShiftReg16 {
	const BITS: usize = std::mem::size_of::<u16>() * 8;

	pub(crate) const fn new() -> Self {
		Self(0)
	}

	pub(crate) fn shl(&mut self) -> bool {
		let ret = self.test_bit(Self::BITS - 1);

		self.0 <<= 1;
		ret
	}

	pub(crate) fn reload(&mut self, val: u8) {
		const MASK: u16 = mask!(u16, 8, 0, true);

		self.0 &= MASK;
		self.0 |= val as u16;
	}

	pub(crate) fn get_bit(&self, fine_x: u8) -> u8 {
		const MASK: u16 = mask!(u16, 1, 15, false);

		((self.0 & (MASK >> (fine_x as u16))) > 0) as u8
	}

	fn test_bit(&self, bit: usize) -> bool {
		(self.0 & (1 << bit)) > 0
	}
}

impl ShiftReg8 {
	const BITS: usize = std::mem::size_of::<u8>() * 8;

	pub(crate) const fn new(val: u8) -> Self {
		Self(val)
	}

	pub(crate) fn shl(&mut self) -> bool {
		let ret = self.test_bit(Self::BITS - 1);

		self.0 <<= 1;
		ret
	}

	pub(crate) fn reload(&mut self, val: u8) {
		self.0 = val;
	}

	pub(crate) fn get_bit(&self) -> u8 {
		self.test_bit(Self::BITS - 1) as u8
	}

	fn test_bit(&self, bit: usize) -> bool {
		(self.0 & (1 << bit)) > 0
	}
}
