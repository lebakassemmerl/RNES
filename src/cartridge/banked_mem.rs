use crate::mem::BankedSegment;

pub const PRG_ROM_BANK_SIZE: usize = 16384;
pub const PRG_RAM_BANK_SIZE: usize = 8192;
pub const CHR_ROM_BANK_SIZE: usize = 8192;
pub const CHR_RAM_BANK_SIZE: usize = 8192;

pub const CI_RAM_BANK_SIZE: usize = 1024;
pub const CI_RAM_BANK_CNT: usize = 2;

pub(crate) struct BankedMemory {
	data: Vec<u8>,
	bank_size: usize,
	bank_cnt: usize,
}

impl BankedMemory {
	pub fn load(data: &[u8], bank_size: usize, bank_cnt: usize) -> Self {
		assert!(data.len() == bank_size * bank_cnt, "BankedMemory: size of data array is invalid");

		Self {
			bank_size: bank_size,
			bank_cnt: bank_cnt,
			data: Vec::from(data),
		}
	}

	pub fn empty(bank_size: usize, bank_cnt: usize) -> Self {
		Self {
			bank_size: bank_size,
			bank_cnt: bank_cnt,
			data: vec![0xFF; bank_size * bank_cnt],
		}
	}

	pub fn bank_cnt(&self) -> usize {
		self.bank_cnt
	}

	pub fn bank_size(&self) -> usize {
		self.bank_size
	}

	pub fn size(&self) -> usize {
		self.data.len()
	}

	pub fn data(&self) -> &Vec<u8> {
		&self.data
	}

	pub fn reload(&mut self, data: &[u8]) {
		self.data.copy_from_slice(data);
	}

	fn idx_in_data(&self, bank_idx: usize, addr: usize) -> usize {
		if bank_idx >= self.bank_cnt {
			panic!("BankedMemory: bank index out of range, idx: {}", bank_idx);
		}
		bank_idx * self.bank_size + (addr % self.bank_size)
	}
}

impl BankedSegment for BankedMemory {
	fn read(&self, bank_idx: usize, addr: usize) -> u8 {
		if self.size() > 0 {
			self.data[self.idx_in_data(bank_idx, addr)]
		} else {
			0
		}
	}

	fn write(&mut self, bank_idx: usize, addr: usize, val: u8) {
		if self.size() > 0 {
			let idx = self.idx_in_data(bank_idx, addr);
			self.data[idx] = val;
		}
	}
}
