use super::Segment;

pub struct Ram<const N: usize> {
	data: Vec<u8>,
}

impl<const N: usize> Ram<N> {
	pub fn empty(val: u8) -> Self {
		Self {
			data: vec![val; N],
		}
	}

	pub const fn size(&self) -> usize {
		N
	}
}

impl<const N: usize> Segment for Ram<N> {
	fn read(&self, addr: usize) -> u8 {
		self.data[addr % N]
	}

	fn write(&mut self, addr: usize, val: u8) {
		self.data[addr % N] = val;
	}
}

#[test]
fn test_write_read_mirroring() {
	let mut ram = Ram::<0x800>::empty(0x00);
	ram.write(0x800, 42);

	// Data is mirrored every 0x800 bytes
	assert_eq!(ram.read(0x800 * 2), 42);
}
