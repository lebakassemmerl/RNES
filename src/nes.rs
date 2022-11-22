use crate::cartridge::{self, banked_mem, CartridgeErr, CartridgeInfo, PpuMirror};
use crate::cpu::{Cpu, InterruptSource};
use crate::io::JoyPad;
use crate::mem::MemoryMap;
use crate::ppu::ppu::Ppu;

use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub const SAVE_FILE_ENDING: &str = ".rsav";

#[allow(dead_code)]
pub struct Nes {
	cpu: Cpu<MemoryMap>,
	mem: MemoryMap,
	ppu: Ppu<MemoryMap>,
	rom_info: RomInfo,

	savefile: Option<String>,
}

#[derive(Default)]
struct RomInfo {
	cartr_info: CartridgeInfo,
	trainer: bool,
	vs_unisystem: bool,
	play_choice: bool,
	ines_version: INesVersion,
}

#[derive(Default)]
struct INesV2Info {
	tv_system: bool, // false = NTSC, true = PAL
	tv_system_support: TVSystemSupport,
	bus_conflicts: bool,
}

pub enum RomErr {
	FileNotFound,
	FileInvalid,
	FileCorrupted,
	SavefileWrite,
	CartridgeError(CartridgeErr),
	Unknown,
}

enum INesVersion {
	V1,
	V2(INesV2Info),
}

enum TVSystemSupport {
	Ntsc,
	Pal,
	Both,
}

impl fmt::Debug for RomErr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self {
			Self::FileCorrupted => write!(f, "FileCorrupted"),
			Self::FileInvalid => write!(f, "FileInvalid"),
			Self::FileNotFound => write!(f, "FileNotFound"),
			Self::SavefileWrite => write!(f, "SavefileWrite"),
			Self::Unknown => write!(f, "Unknown"),
			Self::CartridgeError(ce) => write!(f, "{:?}", ce),
		}
	}
}

impl Nes {
	const PLAY_CHOICE_SIZE: usize = 8192;
	pub const FRAME_TIME_NS: Duration = Duration::new(0, 16_666_667);

	pub fn start(&mut self) {
		self.cpu.assert_interrupt(InterruptSource::RESET);
	}

	pub fn run_frame(&mut self) -> bool {
		loop {
			self.ppu.step(&mut self.mem);
			self.ppu.step(&mut self.mem);
			self.ppu.step(&mut self.mem);

			if self.mem.get_nmi() {
				self.cpu.assert_interrupt(InterruptSource::NMI);
			}

			if self.mem.get_irq() {
				self.cpu.assert_interrupt(InterruptSource::IRQ);
			}

			self.cpu.step(&mut self.mem);

			if self.mem.get_dma() {
				self.cpu.dma_transaction_occurred();
			}

			if self.ppu.fb_ready() {
				return true;
			}

			if self.ppu.frame_finished() {
				// self.mem.dump();
				return false;
			}
		}
	}

	pub fn get_fb(&self) -> Arc<RwLock<Vec<u8>>> {
		self.ppu.get_fb()
	}

	pub fn tile_buf(&mut self) -> Arc<RwLock<Vec<u8>>> {
		self.ppu.tile_buf(&mut self.mem)
	}

	pub fn button_update(&mut self, btns: [JoyPad; 2]) {
		self.mem.button_update(btns);
	}

	pub fn save(&mut self) {
		let c = self.mem.cartridge();
		if c.support_savestates() {
			c.save(self.savefile.as_ref().unwrap().as_str()).unwrap();
		}
	}

	// savestates currently not supported!!
	pub fn new(rom_file: &str) -> Result<Nes, RomErr> {
		if !Path::new(rom_file).exists() {
			return Err(RomErr::FileNotFound);
		}

		let rom = fs::read(rom_file).or_else(|_| Err(RomErr::Unknown))?;
		let rom_info = Nes::parse_ines(&rom)?;

		let start_idx = if rom_info.trainer {
			16 + 512
		} else {
			16
		};

		let end_idx = if rom_info.play_choice {
			rom.len() - Nes::PLAY_CHOICE_SIZE
		} else {
			rom.len()
		};

		let mut cartr = cartridge::load(&rom[start_idx..end_idx], &rom_info.cartr_info)
			.or_else(|e| Err(RomErr::CartridgeError(e)))?;

		let savefile = if cartr.support_savestates() {
			let p = Path::new(rom_file);
			let parent = p.parent().unwrap().to_str().unwrap();
			let name = Path::new(p.file_name().unwrap()).file_stem().unwrap().to_str().unwrap();

			let mut savefile = String::from(parent);
			savefile.push('/');
			savefile.push_str(name);
			savefile.push_str(SAVE_FILE_ENDING);

			// if a savefile exists, restore the savestate
			if Path::exists(Path::new(savefile.as_str())) {
				cartr.restore_savestate(savefile.as_str())?;
			}

			Some(savefile)
		} else {
			None
		};

		Ok(Self {
			cpu: Cpu::new(),
			mem: MemoryMap::new(cartr),
			ppu: Ppu::new(),
			rom_info: rom_info,

			savefile,
		})
	}

	fn parse_ines(bytes: &Vec<u8>) -> Result<RomInfo, RomErr> {
		if bytes.len() < 4 {
			// if file is smaller than the header, it's an invalid file
			return Err(RomErr::FileInvalid);
		}

		if !((bytes[0] == 'N' as u8)
			&& (bytes[1] == 'E' as u8)
			&& (bytes[2] == 'S' as u8)
			&& (bytes[3] == 0x1A))
		{
			// if header isn't present it's an invalid file
			return Err(RomErr::FileInvalid);
		}

		if bytes.len() < 16 {
			// if the file has less than 16bytes (=headersize) it's corrupted
			return Err(RomErr::FileCorrupted);
		}

		let mut file_len: usize = 16;
		let mut desc = RomInfo::default();

		desc.cartr_info.prg_rom_cnt = bytes[4] as usize; // nr. of 16KB PRG_ROM banks
		desc.cartr_info.chr_rom_cnt = bytes[5] as usize; // nr. of 8KB CHR_ROM banks
		desc.cartr_info.battery_ram = (bytes[6] & (1 << 1)) > 0; // if batttery-backed save-ram available

		if (bytes[6] & (1 << 2)) > 0 {
			// trainer-block present
			desc.trainer = true;
			file_len += 512;
		}

		if (bytes[6] & (1 << 3)) > 0 {
			desc.cartr_info.ppu_mirror = PpuMirror::FourScreen;
		} else {
			if (bytes[6] & (1 << 0)) > 0 {
				desc.cartr_info.ppu_mirror = PpuMirror::Vertical;
			} else {
				desc.cartr_info.ppu_mirror = PpuMirror::Horizontal;
			}
		}

		// get the actual mapper-ID
		desc.cartr_info.mapper_id = (bytes[6] >> 4) & 0x0F;
		desc.cartr_info.mapper_id += bytes[7] & 0xF0;

		// not sure what this bit is for, just parse it in case of future useage
		desc.vs_unisystem = (bytes[7] & (1 << 0)) > 0;

		// if 8KB useless Hint-Screen data is stored after CHR-data
		// can be used to slice the actual data correctly for creating
		// the Cartridge-instance
		desc.play_choice = (bytes[7] & (1 << 1)) > 0;

		// get version of the iNES header, if the bits equal 2, it is V2
		let v2 = ((bytes[7] >> 2) & 0x03) == 2;

		// nr. 8KB RAM-banks
		if bytes[8] == 0 {
			// if this value is 0, 1 bank should be assumed
			desc.cartr_info.prg_ram_cnt = 1;
		} else {
			desc.cartr_info.prg_ram_cnt = bytes[8] as usize;
		}

		if v2 {
			let mut v2_data = INesV2Info::default();

			// get TV-System, NTSC or PAL
			v2_data.tv_system = (bytes[9] & (1 << 0)) > 0;

			// if the ROM supports NTSC, PAL or both
			match bytes[10] & 0x03 {
				0 => v2_data.tv_system_support = TVSystemSupport::Ntsc,
				2 => v2_data.tv_system_support = TVSystemSupport::Pal,
				1 | 3 => v2_data.tv_system_support = TVSystemSupport::Both,
				_ => (),
			}

			// true if the rom can cause bus-conflicts by writing to PRG_ROM
			// this will be ignored for now
			v2_data.bus_conflicts = (bytes[10] & (1 << 5)) > 0;
			desc.ines_version = INesVersion::V2(v2_data);
		}

		file_len += desc.cartr_info.prg_rom_cnt * banked_mem::PRG_ROM_BANK_SIZE;
		file_len += desc.cartr_info.chr_rom_cnt * banked_mem::CHR_ROM_BANK_SIZE;

		if file_len != bytes.len() {
			return Err(RomErr::FileCorrupted);
		}

		Ok(desc)
	}
}

// default implementations

impl Default for INesVersion {
	fn default() -> Self {
		INesVersion::V1
	}
}

impl Default for TVSystemSupport {
	fn default() -> Self {
		TVSystemSupport::Ntsc
	}
}
