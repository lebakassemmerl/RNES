use core::ops::{
	Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, ShlAssign, Shr, ShrAssign,
	Sub,
};

pub trait Integer:
	Add<Output = Self>
	+ AddAssign
	+ BitAnd<Output = Self>
	+ BitAndAssign
	+ BitOr<Output = Self>
	+ BitOrAssign
	+ Not<Output = Self>
	+ Shr<usize, Output = Self>
	+ Shl<usize, Output = Self>
	+ ShlAssign
	+ ShrAssign
	+ Sub<Output = Self>
	+ Eq
	+ PartialOrd
	+ Copy
	+ Clone
{
	fn zero() -> Self;
	fn one() -> Self;
}

macro_rules! integer_impl {
	($type:ty) => {
		impl Integer for $type {
			fn zero() -> Self {
				0
			}

			fn one() -> Self {
				1
			}
		}
	};
}

integer_impl!(u8);
integer_impl!(u16);
integer_impl!(u32);
integer_impl!(u64);

pub trait RegisterOps<T: Integer + Into<T>> {
	fn empty() -> Self;
	fn set(&mut self, val: &T);
	fn get(&self) -> T;
	fn set_bit(&mut self, bit: usize);
	fn clear_bit(&mut self, bit: usize);
	fn test_bit(&self, bit: usize) -> bool;
}

pub struct Register<T: Integer + Into<T>>(T);

impl<T: Integer + From<T>> RegisterOps<T> for Register<T> {
	fn empty() -> Self {
		Self {
			0: T::zero(),
		}
	}
	fn set(&mut self, val: &T) {
		self.0 = *val;
	}

	fn get(&self) -> T {
		self.0
	}

	fn set_bit(&mut self, bit: usize) {
		assert!(
			(std::mem::size_of::<T>() * 8 - 1) >= bit as usize,
			"bit-idx out of range: {}",
			bit
		);

		self.0 |= T::one() << bit;
	}

	fn clear_bit(&mut self, bit: usize) {
		assert!(
			(std::mem::size_of::<T>() * 8 - 1) >= bit as usize,
			"bit-idx out of range: {}",
			bit
		);

		self.0 &= !(T::one() << bit);
	}

	fn test_bit(&self, bit: usize) -> bool {
		assert!(
			(std::mem::size_of::<T>() * 8 - 1) >= bit as usize,
			"bit-idx out of range: {}",
			bit
		);

		(self.0 & (T::one() << bit)) > T::zero()
	}
}

impl<T: Integer + From<T>> Default for Register<T> {
	fn default() -> Self {
		Self::empty()
	}
}
