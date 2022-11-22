use std::{
	convert::TryFrom,
	ops::{BitAnd, Shl},
};

// If an assertion fails, the following compiler error will be printed:
// 'attempt to compute `0_usize - 1_usize`, which would overflow'
#[macro_export]
macro_rules! mask {
	($type:ty, $bits:expr, $bit_idx:expr, $inv:expr) => {{
		use crate::const_assert;

		const BIT_CNT: usize = std::mem::size_of::<$type>() * 8;

		const_assert!(($bits + $bit_idx) > 0usize);
		const_assert!($bit_idx < BIT_CNT);
		const_assert!(($bits + $bit_idx - 1) < BIT_CNT);

		let one: $type = 1 as $type;
		let zero: $type = 0 as $type;

		let step1 = (!zero) - ((one << $bit_idx) - 1);
		let step2 = (!zero) >> ((BIT_CNT - 1) - ($bits + $bit_idx - 1));

		if $inv {
			!(step1 & step2)
		} else {
			step1 & step2
		}
	}};
}

pub const fn reverse_u8(val: u8) -> u8 {
	let mut b = (val & 0xF0) >> 4 | (val & 0x0F) << 4;
	b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
	b = (b & 0xAA) >> 1 | (b & 0x55) << 1;

	b
}

pub fn test_bit<T: Copy + PartialOrd + BitAnd<Output = T> + Shl<Output = T> + TryFrom<usize>>(
	val: &T,
	bit: usize,
) -> bool
where
	<T as TryFrom<usize>>::Error: std::fmt::Debug,
{
	(*val & (T::try_from(1usize).unwrap() << T::try_from(bit).unwrap()))
		> T::try_from(0usize).unwrap()
}

mod test {
	#[test]
	fn create_mask_noninverted_u8() {
		assert_eq!(0b0000_1111u8, mask!(u8, 4, 0, false));
		assert_eq!(0b0011_1000u8, mask!(u8, 3, 3, false));
		assert_eq!(0b1111_1111u8, mask!(u8, 8, 0, false));
		assert_eq!(0b0000_0001u8, mask!(u8, 1, 0, false));
		assert_eq!(0b1000_0000u8, mask!(u8, 1, 7, false));

		// should fail to compile
		// assert_eq!(0b0000_0000u8, mask!(u8, 0, 0, false));
		// assert_eq!(0b1111_1110u8, mask!(u8, 8, 1, false));
	}

	#[test]
	fn create_mask_inverted_u8() {
		assert_eq!(0b1111_0000u8, mask!(u8, 4, 0, true));
		assert_eq!(0b1100_0111u8, mask!(u8, 3, 3, true));
		assert_eq!(0b0000_0000u8, mask!(u8, 8, 0, true));
		assert_eq!(0b1111_1110u8, mask!(u8, 1, 0, true));
		assert_eq!(0b0111_1111u8, mask!(u8, 1, 7, true));

		// should fail to compile
		// assert_eq!(0b11111111u8, mask!(u8, 0, 0, true));
	}

	#[test]
	fn create_mask_u16() {
		assert_eq!(0xFF00u16, mask!(u16, 8, 8, false));
		assert_eq!(0xF000u16, mask!(u16, 4, 12, false));
		assert_eq!(0x0003u16, mask!(u16, 2, 0, false));
		assert_eq!(0xFFFFu16, mask!(u16, 16, 0, false));
		assert_eq!(0x0001u16, mask!(u16, 1, 0, false));

		// should fail to compile
		// assert_eq!(0x0000u16, mask!(u16,  0, 0, false));
		// assert_eq!(0xFFFEu16, mask!(u16, 16, 1, false));
	}
}
