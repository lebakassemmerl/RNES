use super::banked_mem::*;
use super::mem::{BankedSegment, PpuSegment, Segment};
use super::{Cartridge, CartridgeInfo, LoadRom};
use crate::mask;

const MMC1_CHR_ROM_BANK_SIZE: usize = 4 * 1024;
const CHR_RAM_BANK_CNT: usize = (128 * 1024) / MMC1_CHR_ROM_BANK_SIZE;
const SHIFT_REG_RESET_MASK: u8 = mask!(u8, 1, 7, false);
const PRG_RAM_ENABLE_MASK: u8 = mask!(u8, 1, 4, false);
const PRG_REG_RESET: u8 = 0x0C;

struct ShiftRegister(u8);

impl ShiftRegister {
	const INIT_VAL: u8 = mask!(u8, 1, 7, false); // use one bit to detect the 'fullness'
	const FULL_MASK: u8 = mask!(u8, 1, 2, false);
	const ENQUEUE_SHIFT: u8 = 7;
	const FINAL_VALUE_SHIFT: u8 = 3;

	fn new() -> Self {
		Self(Self::INIT_VAL)
	}

	fn enqueue(&mut self, val: u8) {
		if (val & SHIFT_REG_RESET_MASK) > 0 {
			self.0 = Self::INIT_VAL;
		} else {
			if self.value_ready() {
				self.0 = Self::INIT_VAL;
			}

			self.0 >>= 1;
			self.0 |= (val & 0x01) << Self::ENQUEUE_SHIFT;
		}
	}

	fn value_ready(&self) -> bool {
		(self.0 & Self::FULL_MASK) > 0
	}

	fn get_value(&self) -> u8 {
		(self.0 >> Self::FINAL_VALUE_SHIFT) & 0x1F
	}
}

pub(crate) struct Mmc1 {
	prg_rom: BankedMemory,
	chr_romram: BankedMemory,
	prg_ram: Option<BankedMemory>,
	ci_ram: BankedMemory,
	use_chr_ram: bool,

	prg_sel: [usize; 2],
	chr_sel: [usize; 2],
	ci_sel: [usize; 4],

	shiftreg: ShiftRegister,
	ctrl_reg: u8,
	chr0_reg: u8,
	chr1_reg: u8,
	prg_reg: u8,
}

impl Mmc1 {
	fn update_banks(&mut self) {
		const CHR_8K_MASK: u8 = mask!(u8, 1, 4, false);
		const MIRROR_MASK: u8 = mask!(u8, 2, 0, false);
		const DOUBLE_BANK_MASK: u8 = mask!(u8, 1, 0, true);
		const PRG_ROM_SEL_MASK: u8 = mask!(u8, 4, 0, false);
		const PRG_ROM_MODE_MASK: u8 = mask!(u8, 2, 2, false);
		const PRG_ROM_MODE_IDX: u8 = 2;

		match self.ctrl_reg & MIRROR_MASK {
			0 => {
				// one-screen, lower bank
				self.ci_sel[0] = 0;
				self.ci_sel[1] = 0;
				self.ci_sel[2] = 0;
				self.ci_sel[3] = 0;
			}
			1 => {
				// one-screen, upper bank
				self.ci_sel[0] = 1;
				self.ci_sel[1] = 1;
				self.ci_sel[2] = 1;
				self.ci_sel[3] = 1;
			}
			2 => {
				// vertical
				self.ci_sel[0] = 0;
				self.ci_sel[1] = 1;
				self.ci_sel[2] = 0;
				self.ci_sel[3] = 1;
			}
			3 => {
				// horizontal
				self.ci_sel[0] = 0;
				self.ci_sel[1] = 0;
				self.ci_sel[2] = 1;
				self.ci_sel[3] = 1;
			}
			_ => {}
		}

		if (self.ctrl_reg & CHR_8K_MASK) == 0 {
			self.chr_sel[0] = (self.chr0_reg & DOUBLE_BANK_MASK) as usize;
			self.chr_sel[1] = (self.chr0_reg | 0x01) as usize;
		} else {
			self.chr_sel[0] = self.chr0_reg as usize;
			self.chr_sel[1] = self.chr1_reg as usize;
		}

		let prg_mode = (self.ctrl_reg & PRG_ROM_MODE_MASK) >> PRG_ROM_MODE_IDX;
		let bank = (self.prg_reg & PRG_ROM_SEL_MASK) as usize;
		match prg_mode {
			0 | 1 => {
				// 32KB mode
				self.prg_sel[0] = bank & (DOUBLE_BANK_MASK as usize);
				self.prg_sel[1] = bank | 0x01;
			}
			2 => {
				self.prg_sel[0] = 0;
				self.prg_sel[1] = bank;
			}
			3 => {
				self.prg_sel[0] = bank;
				self.prg_sel[1] = self.prg_rom.bank_cnt() - 1;
			}
			_ => {}
		}
	}
}

impl Segment for Mmc1 {
	fn read(&self, addr: usize) -> u8 {
		match addr {
			0x6000..=0x7FFF => {
				if self.prg_ram.is_none() {
					0
				} else if (self.prg_reg & PRG_RAM_ENABLE_MASK) > 0 {
					0
				} else {
					self.prg_ram.as_ref().unwrap().read(0, addr)
				}
			}
			0x8000..=0xBFFF => self.prg_rom.read(self.prg_sel[0], addr),
			0xC000..=0xFFFF => self.prg_rom.read(self.prg_sel[1], addr),
			_ => panic!("MMC1 segment read(): address out of memory range: 0x{:x}", addr),
		}
	}
	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x6000..=0x7FFF => {
				if let Some(ram) = self.prg_ram.as_mut() {
					if (self.prg_reg & PRG_RAM_ENABLE_MASK) == 0 {
						ram.write(0, addr, val);
					}
				}
			}
			0x8000..=0xFFFF => {
				self.shiftreg.enqueue(val);
				if (val & SHIFT_REG_RESET_MASK) > 0 {
					self.ctrl_reg |= PRG_REG_RESET;

					self.update_banks();
				}

				if self.shiftreg.value_ready() {
					let a = (addr >> 13) & 0x03;
					let v = self.shiftreg.get_value();

					match a {
						0 => self.ctrl_reg = v,
						1 => self.chr0_reg = v,
						2 => self.chr1_reg = v,
						3 => self.prg_reg = v,
						_ => {}
					}

					self.update_banks();
				}
			}
			_ => panic!("MMC1 segment write(): address out of memory range: 0x{:x}", addr),
		}
	}
}

impl PpuSegment for Mmc1 {
	fn read(&mut self, addr: usize) -> u8 {
		match addr {
			0x0000..=0x0FFF => self.chr_romram.read(self.chr_sel[0], addr),
			0x1000..=0x1FFF => self.chr_romram.read(self.chr_sel[1], addr),

			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.read(self.ci_sel[0], addr),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.read(self.ci_sel[1], addr),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.read(self.ci_sel[2], addr),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.read(self.ci_sel[3], addr),
			_ => panic!("MMC1 PPU segment read(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x0000..=0xFFF => {
				if self.use_chr_ram {
					self.chr_romram.write(self.chr_sel[0], addr, val);
				}
			}
			0x1000..=0x1FFF => {
				if self.use_chr_ram {
					self.chr_romram.write(self.chr_sel[1], addr, val);
				}
			}
			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.write(self.ci_sel[0], addr, val),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.write(self.ci_sel[1], addr, val),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.write(self.ci_sel[2], addr, val),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.write(self.ci_sel[3], addr, val),
			_ => panic!("MMC1 PPU segment write(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn irq(&mut self) -> bool {
		false
	}
}

impl LoadRom for Mmc1 {
	fn load(data: &[u8], info: &CartridgeInfo) -> Box<dyn Cartridge> {
		println!("Load MMC1 ROM");

		let prg_rom_bytes = info.prg_rom_cnt * PRG_ROM_BANK_SIZE;
		let prg_rom =
			BankedMemory::load(&data[..prg_rom_bytes], PRG_ROM_BANK_SIZE, info.prg_rom_cnt);

		let chr_rom_cnt = info.chr_rom_cnt * CHR_ROM_BANK_SIZE / MMC1_CHR_ROM_BANK_SIZE;
		let (chr_mem, ram) = if chr_rom_cnt == 0 {
			// if chr_rom_cnt is zero, we only have CHR RAM!
			(BankedMemory::empty(MMC1_CHR_ROM_BANK_SIZE, CHR_RAM_BANK_CNT), true)
		} else {
			(BankedMemory::load(&data[prg_rom_bytes..], MMC1_CHR_ROM_BANK_SIZE, chr_rom_cnt), false)
		};

		let prg_ram = if info.prg_ram_cnt == 0 {
			None
		} else {
			Some(BankedMemory::empty(PRG_RAM_BANK_SIZE, 1))
		};

		let mut ret = Box::new(Self {
			prg_rom,
			chr_romram: chr_mem,
			prg_ram,
			ci_ram: BankedMemory::empty(CI_RAM_BANK_SIZE, CI_RAM_BANK_CNT),
			use_chr_ram: ram,

			prg_sel: [0; 2],
			chr_sel: [0; 2],
			ci_sel: [0; 4],

			shiftreg: ShiftRegister::new(),
			ctrl_reg: PRG_REG_RESET,
			chr0_reg: 0,
			chr1_reg: 0,
			prg_reg: 0,
		});

		ret.update_banks();

		ret
	}
}

impl Cartridge for Mmc1 {
	fn support_savestates(&self) -> bool {
		self.prg_ram.is_some()
	}

	fn get_battery_ram<'a>(&'a self) -> &'a [u8] {
		self.prg_ram.as_ref().unwrap().data().as_slice()
	}

	fn set_battery_ram(&mut self, ram: &[u8]) {
		self.prg_ram.as_mut().unwrap().reload(ram);
	}
}
