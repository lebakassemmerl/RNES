use super::banked_mem::*;
use super::mem::{BankedSegment, PpuSegment, Segment};
use super::{Cartridge, CartridgeInfo, LoadRom, PpuMirror};

pub(crate) struct UxRom {
	prg_rom: BankedMemory,
	chr_ram: BankedMemory,
	ci_ram: BankedMemory,
	bank_idx: usize,
	bank_cnt: usize,
	nt1_idx: usize,
	nt2_idx: usize,
}

impl Segment for UxRom {
	fn read(&self, addr: usize) -> u8 {
		match addr {
			0x8000..=0xBFFF => self.prg_rom.read(self.bank_idx, addr),
			0xC000..=0xFFFF => self.prg_rom.read(self.bank_cnt - 1, addr),
			_ => panic!("UxRom segment read(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x8000..=0xFFFF => self.bank_idx = (val & 0x0F) as usize,
			_ => panic!("UxRom segment write(): address out of memory range: 0x{:x}", addr),
		}
	}
}

impl PpuSegment for UxRom {
	fn read(&mut self, addr: usize) -> u8 {
		let bank_addr = addr % self.ci_ram.bank_size();

		match addr {
			0..=0x1FFF => self.chr_ram.read(0, addr),
			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.read(0, bank_addr),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.read(self.nt1_idx, bank_addr),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.read(self.nt2_idx, bank_addr),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.read(1, bank_addr),
			_ => panic!("UxRom PPU segment read(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn write(&mut self, addr: usize, val: u8) {
		let bank_addr = addr % self.ci_ram.bank_size();

		match addr {
			0..=0x1FFF => self.chr_ram.write(0, addr, val),
			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.write(0, bank_addr, val),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.write(self.nt1_idx, bank_addr, val),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.write(self.nt2_idx, bank_addr, val),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.write(1, bank_addr, val),
			_ => panic!("UxRom PPU segment write(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn irq(&mut self) -> bool {
		false
	}
}

impl LoadRom for UxRom {
	fn load(data: &[u8], info: &CartridgeInfo) -> Box<dyn Cartridge> {
		println!("Load UxROM ROM");

		assert_eq!(data.len(), info.prg_rom_cnt * PRG_ROM_BANK_SIZE);

		let (nt1, nt2) = if let PpuMirror::Horizontal = info.ppu_mirror {
			(0, 1)
		} else {
			(1, 0)
		};

		Box::new(Self {
			// always only 1 CHR_RAM bank since no CHR_ROM is available
			chr_ram: BankedMemory::empty(CHR_RAM_BANK_SIZE, 1),
			prg_rom: BankedMemory::load(data, PRG_ROM_BANK_SIZE, info.prg_rom_cnt),
			ci_ram: BankedMemory::empty(CI_RAM_BANK_SIZE, CI_RAM_BANK_CNT),
			bank_idx: 0,
			bank_cnt: info.prg_rom_cnt,
			nt1_idx: nt1,
			nt2_idx: nt2,
		})
	}
}

impl Cartridge for UxRom {
	fn support_savestates(&self) -> bool {
		false
	}

	fn get_battery_ram<'a>(&'a self) -> &'a [u8] {
		panic!("UxROM: savestates are not supported");
	}

	fn set_battery_ram(&mut self, ram: &[u8]) {
		panic!("UxROM: savestates are not supported");
	}
}
