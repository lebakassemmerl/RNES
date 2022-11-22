use super::banked_mem::*;
use super::mem::{BankedSegment, PpuSegment, Segment};
use super::{Cartridge, CartridgeInfo, LoadRom, PpuMirror};

pub(crate) struct CNRom {
	prg_rom: BankedMemory,
	chr_rom: BankedMemory,
	ci_ram: BankedMemory,
	nt1_idx: usize,
	nt2_idx: usize,
	chr_rom_bank: u8,
}

impl Segment for CNRom {
	fn read(&self, addr: usize) -> u8 {
		match addr {
			0x8000..=0xBFFF => self.prg_rom.read(0, addr),
			0xC000..=0xFFFF => self.prg_rom.read(1, addr),
			_ => panic!("CNRom segment read(): address out of memory range: 0x{:x}", addr),
		}
	}
	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x8000..=0xFFFF => self.chr_rom_bank = val,
			_ => panic!("CNRom segment write(): address out of memory range: 0x{:x}", addr),
		}
	}
}

impl PpuSegment for CNRom {
	fn read(&mut self, addr: usize) -> u8 {
		let bank_addr = addr % self.ci_ram.bank_size();

		match addr {
			0..=0x1FFF => self.chr_rom.read(self.chr_rom_bank as usize, addr),
			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.read(0, bank_addr),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.read(self.nt1_idx, bank_addr),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.read(self.nt2_idx, bank_addr),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.read(1, bank_addr),
			_ => panic!("CNRom PPU segment read(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn write(&mut self, addr: usize, val: u8) {
		let bank_addr = addr % self.ci_ram.bank_size();

		match addr {
			0..=0x1FFF => (),
			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.write(0, bank_addr, val),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.write(self.nt1_idx, bank_addr, val),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.write(self.nt2_idx, bank_addr, val),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.write(1, bank_addr, val),
			_ => panic!("CNRom PPU segment write(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn irq(&mut self) -> bool {
		false
	}
}

impl LoadRom for CNRom {
	fn load(data: &[u8], info: &CartridgeInfo) -> Box<dyn Cartridge> {
		println!("Load CNROM ROM");

		let (nt1, nt2) = if let PpuMirror::Horizontal = info.ppu_mirror {
			(0, 1)
		} else {
			(1, 0)
		};

		let prg_rom_bytes = info.prg_rom_cnt * PRG_ROM_BANK_SIZE;
		Box::new(Self {
			prg_rom: BankedMemory::load(
				&data[..prg_rom_bytes],
				PRG_ROM_BANK_SIZE,
				info.prg_rom_cnt,
			),
			chr_rom: BankedMemory::load(
				&data[prg_rom_bytes..(prg_rom_bytes + info.chr_rom_cnt * CHR_ROM_BANK_SIZE)],
				CHR_ROM_BANK_SIZE,
				info.chr_rom_cnt,
			),
			ci_ram: BankedMemory::empty(CI_RAM_BANK_SIZE, CI_RAM_BANK_CNT),
			nt1_idx: nt1,
			nt2_idx: nt2,
			chr_rom_bank: 0,
		})
	}
}

impl Cartridge for CNRom {
	fn support_savestates(&self) -> bool {
		false
	}

	fn get_battery_ram<'a>(&'a self) -> &'a [u8] {
		panic!("CNROM: savestates are not supported");
	}

	fn set_battery_ram(&mut self, ram: &[u8]) {
		panic!("CNROM: savestates are not supported");
	}
}
