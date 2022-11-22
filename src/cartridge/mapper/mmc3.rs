use super::banked_mem::*;
use super::mem::{BankedSegment, PpuSegment, Segment};
use super::{Cartridge, CartridgeInfo, LoadRom, PpuMirror};
use crate::mask;

const MMC3_PRG_ROM_BANK_SIZE: usize = 8192;
const MMC3_CHR_ROM_BANK_SIZE: usize = 1024;

pub(crate) struct Mmc3 {
	prg_rom: BankedMemory,
	chr_rom: BankedMemory,
	prg_ram: Option<BankedMemory>,
	ci_ram: BankedMemory,

	prg_sel: [usize; 3],
	chr_sel: [usize; 8],
	bank_regs: [u8; 8],
	bank_sel: u8,
	ci_sel: [usize; 4],
	ci_4screen: bool,

	prg_ram_enable: bool,
	prg_ram_wp: bool,

	irq_counter: u8,
	irq_load: u8,
	irq_reload: bool,
	irq_enable: bool,
	prev_a12: bool,
	irq_thrown: bool,
	irq_asserted: bool,
}

impl Mmc3 {
	fn clock_irq_counter(&mut self, addr: usize) {
		const A12_MASK: usize = mask!(usize, 1, 12, false);
		const ADDR_MAX: usize = 0x2000;

		let a12 = addr < ADDR_MAX && (addr & A12_MASK) > 0;
		let rising_edge = self.prev_a12 == false && a12 == true;
		self.prev_a12 = a12;

		// we have to clock the counter on the rising edge
		if !rising_edge {
			return;
		}

		if (self.irq_counter == 0) || self.irq_reload {
			self.irq_reload = false;
			self.irq_counter = self.irq_load;
		} else {
			self.irq_counter -= 1;
		}

		if self.irq_enable && self.irq_counter == 0 {
			if self.irq_load > 0 || (self.irq_load == 0 && !self.irq_thrown) {
				self.irq_thrown = true;
				self.irq_asserted = true;
			}
		}
	}

	fn update_banks(&mut self) {
		const DOUBLE_BANK_MASK: u8 = mask!(u8, 1, 0, true);
		const A12_INV_MASK: u8 = mask!(u8, 1, 7, false);
		const ROM_MODE_MASK: u8 = mask!(u8, 1, 6, false);

		let a12 = (self.bank_sel & A12_INV_MASK) > 0;
		let rom_mode = (self.bank_sel & ROM_MODE_MASK) > 0;

		let chr_offs = (a12 as usize) * 4;
		let prg_sec_last_idx = ((!rom_mode) as usize) * 2;
		let prg_r6_idx = (rom_mode as usize) * 2;

		self.chr_sel[0 + chr_offs] = (self.bank_regs[0] & DOUBLE_BANK_MASK) as usize;
		self.chr_sel[1 + chr_offs] = (self.bank_regs[0] | 0x01) as usize;
		self.chr_sel[2 + chr_offs] = (self.bank_regs[1] & DOUBLE_BANK_MASK) as usize;
		self.chr_sel[3 + chr_offs] = (self.bank_regs[1] | 0x01) as usize;

		self.chr_sel[4 - chr_offs] = self.bank_regs[2] as usize;
		self.chr_sel[5 - chr_offs] = self.bank_regs[3] as usize;
		self.chr_sel[6 - chr_offs] = self.bank_regs[4] as usize;
		self.chr_sel[7 - chr_offs] = self.bank_regs[5] as usize;

		self.prg_sel[prg_r6_idx] = self.bank_regs[6] as usize;
		self.prg_sel[1] = self.bank_regs[7] as usize;
		self.prg_sel[prg_sec_last_idx] = self.prg_rom.bank_cnt() - 2;
	}
}

impl Segment for Mmc3 {
	fn read(&self, addr: usize) -> u8 {
		match addr {
			0x6000..=0x7FFF => {
				if self.prg_ram.is_none() {
					0
				} else if !self.prg_ram_enable {
					0
				} else {
					self.prg_ram.as_ref().unwrap().read(0, addr)
				}
			}
			0x8000..=0x9FFF => self.prg_rom.read(self.prg_sel[0], addr),
			0xA000..=0xBFFF => self.prg_rom.read(self.prg_sel[1], addr),
			0xC000..=0xDFFF => self.prg_rom.read(self.prg_sel[2], addr),
			0xE000..=0xFFFF => self.prg_rom.read(self.prg_rom.bank_cnt() - 1, addr),
			_ => panic!("MMC3 segment read(): address out of memory range: 0x{:x}", addr),
		}
	}
	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x6000..=0x7FFF => {
				if let Some(ram) = self.prg_ram.as_mut() {
					if !self.prg_ram_wp && self.prg_ram_enable {
						ram.write(0, addr, val);
					}
				}
			}
			0x8000..=0x9FFF => {
				if (addr & 0x01) > 0 {
					// odd, bank data register
					self.bank_regs[(self.bank_sel as usize) & 0x07] = val;
				} else {
					// even bank selection register
					self.bank_sel = val;
				}
				self.update_banks();
			}
			0xA000..=0xBFFF => {
				if (addr & 0x01) > 0 {
					// odd, PRG RAM protect
					self.prg_ram_enable = (val & 0x80) > 0;
					self.prg_ram_wp = (val & 0x40) > 0;
				} else {
					// even, mirroring register
					if !self.ci_4screen {
						if (val & 0x01) > 0 {
							// horizontal mirroring is desired
							self.ci_sel[1] = 0;
							self.ci_sel[2] = 1;
						} else {
							// vertical mirroring is desired
							self.ci_sel[1] = 1;
							self.ci_sel[2] = 0;
						}
					}
				}
			}
			0xC000..=0xDFFF => {
				if (addr & 0x01) > 0 {
					// odd, reload IRQ counter
					self.irq_reload = true;
					self.irq_thrown = false;
				} else {
					// even, update IRQ latch
					self.irq_load = val;
				}
			}
			0xE000..=0xFFFF => {
				if (addr & 0x01) > 0 {
					// odd, IRQ enable register
					self.irq_enable = true;
				} else {
					// even, IRQ disable register
					self.irq_enable = false;
					self.irq_asserted = false;
					self.irq_counter = self.irq_load;
				}
			}
			_ => panic!("MMC3 segment write(): address out of memory range: 0x{:x}", addr),
		}
	}
}

impl PpuSegment for Mmc3 {
	fn read(&mut self, addr: usize) -> u8 {
		self.clock_irq_counter(addr);

		match addr {
			0x0000..=0x03FF => self.chr_rom.read(self.chr_sel[0], addr),
			0x0400..=0x07FF => self.chr_rom.read(self.chr_sel[1], addr),
			0x0800..=0x0BFF => self.chr_rom.read(self.chr_sel[2], addr),
			0x0C00..=0x0FFF => self.chr_rom.read(self.chr_sel[3], addr),
			0x1000..=0x13FF => self.chr_rom.read(self.chr_sel[4], addr),
			0x1400..=0x17FF => self.chr_rom.read(self.chr_sel[5], addr),
			0x1800..=0x1BFF => self.chr_rom.read(self.chr_sel[6], addr),
			0x1C00..=0x1FFF => self.chr_rom.read(self.chr_sel[7], addr),

			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.read(self.ci_sel[0], addr),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.read(self.ci_sel[1], addr),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.read(self.ci_sel[2], addr),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.read(self.ci_sel[3], addr),
			_ => panic!("MMC3 PPU segment read(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.write(self.ci_sel[0], addr, val),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.write(self.ci_sel[1], addr, val),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.write(self.ci_sel[2], addr, val),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.write(self.ci_sel[3], addr, val),
			_ => panic!("MMC3 PPU segment write(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn irq(&mut self) -> bool {
		let ret = self.irq_asserted;
		self.irq_asserted = false;

		ret
	}
}

impl LoadRom for Mmc3 {
	fn load(data: &[u8], info: &CartridgeInfo) -> Box<dyn Cartridge> {
		println!("Load MMC3 ROM");

		let prg_rom_bytes = info.prg_rom_cnt * PRG_ROM_BANK_SIZE;

		let prg_rom_cnt = info.prg_rom_cnt * PRG_ROM_BANK_SIZE / MMC3_PRG_ROM_BANK_SIZE;
		let prg_rom =
			BankedMemory::load(&data[..prg_rom_bytes], MMC3_PRG_ROM_BANK_SIZE, prg_rom_cnt);

		let chr_rom_cnt = info.chr_rom_cnt * CHR_ROM_BANK_SIZE / MMC3_CHR_ROM_BANK_SIZE;
		let chr_rom =
			BankedMemory::load(&data[prg_rom_bytes..], MMC3_CHR_ROM_BANK_SIZE, chr_rom_cnt);

		let prg_ram = if info.prg_ram_cnt == 0 {
			None
		} else {
			Some(BankedMemory::empty(PRG_RAM_BANK_SIZE, 1))
		};

		let (ci_ram, ci_sel, ci_4screen) = match info.ppu_mirror {
			PpuMirror::Horizontal => {
				(BankedMemory::empty(CI_RAM_BANK_SIZE, CI_RAM_BANK_CNT), [0, 0, 1, 1], false)
			}
			PpuMirror::Vertical => {
				(BankedMemory::empty(CI_RAM_BANK_SIZE, CI_RAM_BANK_CNT), [0, 1, 0, 1], false)
			}
			PpuMirror::FourScreen => {
				(BankedMemory::empty(CI_RAM_BANK_SIZE, CI_RAM_BANK_CNT * 2), [0, 1, 2, 3], true)
			}
			_ => panic!("MMC3: mirroring-mode not supported: {}", info.ppu_mirror),
		};

		Box::new(Self {
			prg_rom,
			chr_rom,
			prg_ram,
			ci_ram,

			prg_sel: [0; 3],
			chr_sel: [0; 8],
			bank_regs: [0; 8],
			bank_sel: 0,
			ci_sel,
			ci_4screen,

			prg_ram_enable: true,
			prg_ram_wp: false,

			irq_counter: 0,
			irq_load: 0,
			irq_reload: false,
			irq_enable: false,
			prev_a12: false,
			irq_thrown: false,
			irq_asserted: false,
		})
	}
}

impl Cartridge for Mmc3 {
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
