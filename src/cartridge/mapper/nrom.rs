use super::banked_mem::*;
use super::mem::{BankedSegment, PpuSegment, Segment};
use super::{Cartridge, CartridgeInfo, LoadRom, PpuMirror};

pub(crate) struct NRom {
	prg_ram: BankedMemory,
	prg_rom: BankedMemory,
	chr_rom: BankedMemory,
	ci_ram: BankedMemory,
	nt1_idx: usize,
	nt2_idx: usize,
}

impl Segment for NRom {
	fn read(&self, addr: usize) -> u8 {
		match addr {
			0x4020..=0x5FFF => 0x00, // no expansion ROM supported
			0x6000..=0x7FFF => self.prg_ram.read(0, addr),
			0x8000..=0xBFFF => self.prg_rom.read(0, addr),
			0xC000..=0xFFFF => {
				if self.prg_rom.bank_cnt() == 2 {
					self.prg_rom.read(1, addr)
				} else if self.prg_rom.bank_cnt() == 1 {
					// if only one bank is present, mirror it
					self.prg_rom.read(0, addr)
				} else {
					panic!(
						"NRom segment read(): invalid PRG_ROM length: 0x{:x}",
						self.prg_rom.size()
					);
				}
			}
			_ => panic!("NRom segment read(): address out of memory range: 0x{:x}", addr),
		}
	}
	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x4020..=0x5FFF => {} // no expansion ROM supported
			0x6000..=0x7FFF => self.prg_ram.write(0, addr, val),
			0x8000..=0xFFFF => {} // has no RAM to write to
			_ => panic!("NRom segment write(): address out of memory range: 0x{:x}", addr),
		}
	}
}

impl PpuSegment for NRom {
	fn read(&mut self, addr: usize) -> u8 {
		let bank_addr = addr % self.ci_ram.bank_size();

		match addr {
			0..=0x1FFF => self.chr_rom.read(0, addr),
			0x2000..=0x23FF | 0x3000..=0x33FF => self.ci_ram.read(0, bank_addr),
			0x2400..=0x27FF | 0x3400..=0x37FF => self.ci_ram.read(self.nt1_idx, bank_addr),
			0x2800..=0x2BFF | 0x3800..=0x3BFF => self.ci_ram.read(self.nt2_idx, bank_addr),
			0x2C00..=0x2FFF | 0x3C00..=0x3EFF => self.ci_ram.read(1, bank_addr),
			_ => panic!("NRom PPU segment read(): address out of memory range: 0x{:x}", addr),
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
			_ => panic!("NRom PPU segment write(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn irq(&mut self) -> bool {
		false
	}
}

impl LoadRom for NRom {
	fn load(data: &[u8], info: &CartridgeInfo) -> Box<dyn Cartridge> {
		println!("Load NROM ROM");

		assert_input(data, info);

		let (nt1, nt2) = if let PpuMirror::Horizontal = info.ppu_mirror {
			(0, 1)
		} else {
			(1, 0)
		};

		let prg_rom_bytes = info.prg_rom_cnt * PRG_ROM_BANK_SIZE;
		Box::new(Self {
			// always only 1 RAM bank, actually the size SHOULD be 2KB or
			// with the 'Family Basic' edition 4KB but most emulators
			// just use a 1 8KB bank which obviously seems to work
			prg_ram: BankedMemory::empty(PRG_RAM_BANK_SIZE, 1),
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
		})
	}
}

impl Cartridge for NRom {
	fn support_savestates(&self) -> bool {
		true
	}

	fn set_battery_ram(&mut self, ram: &[u8]) {
		assert!(
			ram.len() == self.prg_ram.size(),
			"NROM: invalid savestate-length: expected {}, got {}",
			self.prg_ram.size(),
			ram.len()
		);

		self.prg_ram.reload(ram);
	}

	fn get_battery_ram<'a>(&'a self) -> &'a [u8] {
		self.prg_ram.data().as_slice()
	}
}

fn assert_input(data: &[u8], info: &CartridgeInfo) {
	assert_eq!(
		data.len(),
		info.prg_rom_cnt * PRG_ROM_BANK_SIZE + info.chr_rom_cnt * CHR_ROM_BANK_SIZE,
		"NRom: length of data array does not match with bank_cnt and bank_size: 0x{:x}",
		data.len()
	);

	assert_eq!(info.chr_rom_cnt, 1, "NROM supports only 1 CHR_ROM bank");

	match info.ppu_mirror {
		PpuMirror::Horizontal | PpuMirror::Vertical => (),
		_ => panic!("NRom: unsupported ppu mirroring: {}", info.ppu_mirror),
	}
}
