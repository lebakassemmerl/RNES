use super::catridge_mem::*;
use super::mem::{BankedSegment, PpuSegment, Segment};
use super::Cartridge;

pub(crate) struct Unmapped {
	prg_rom: BankedMemory,
	prg_ram: BankedMemory,
	chr_rom: BankedMemory,
}

impl Cartridge for Unmapped {}

impl Segment for Unmapped {
	fn read(&self, addr: usize) -> u8 {
		match addr {
			0x6000..=0x7FFF => self.prg_ram.read(0, addr),
			0x8000..=0xBFFF => self.prg_rom.read(0, addr),
			_ => panic!("Unmapped segment read(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x6000..=0x7FFF => self.prg_ram.write(0, addr, val),
			0x8000..=0xBFFF => (),
			_ => panic!("Unmapped segment write(): address out of memory range: 0x{:x}", addr),
		}
	}
}

impl PpuSegment for Unmapped {
	fn read(&mut self, addr: usize) -> u8 {
		match addr {
			0..=0x1FFF => self.chr_rom.read(0, addr),
			_ => panic!("Unmapped PPU segment read(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn write(&mut self, addr: usize, _val: u8) {
		match addr {
			0..=0x1FFF => (),
			_ => panic!("Unmapped PPU segment write(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn irq(&mut self) -> bool {
		false
	}
}

impl Unmapped {
	// this mapper is only for testing reasons, so make a constant number
	// of banks and do also fixed mapping
	pub fn empty() -> Box<dyn Cartridge> {
		Box::new(Self {
			prg_rom: BankedMemory::empty(2 * PRG_ROM_BANK_SIZE, 1),
			prg_ram: BankedMemory::empty(PRG_RAM_BANK_SIZE, 1),
			chr_rom: BankedMemory::empty(CHR_ROM_BANK_SIZE, 1),
		})
	}
}
