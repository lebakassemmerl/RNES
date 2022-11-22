pub(crate) mod banked_mem;
pub(crate) mod mapper;

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::mem;
use crate::nes::RomErr;
use mapper::*;

pub trait Cartridge: mem::Segment + mem::PpuSegment {
	fn support_savestates(&self) -> bool;
	fn get_battery_ram<'a>(&'a self) -> &'a [u8];
	fn set_battery_ram(&mut self, ram: &[u8]);

	fn restore_savestate(&mut self, savefile: &str) -> Result<(), RomErr> {
		if !self.support_savestates() {
			return Ok(());
		}

		if !Path::new(savefile).exists() {
			return Err(RomErr::FileNotFound);
		}

		let save = fs::read(savefile).or_else(|_| Err(RomErr::Unknown))?;
		if !(save[0] == ('R' as u8)
			&& save[1] == ('N' as u8)
			&& save[2] == ('E' as u8)
			&& save[3] == ('S' as u8))
		{
			return Err(RomErr::FileInvalid);
		}

		self.set_battery_ram(&save[4..]);

		Ok(())
	}
	fn save(&self, savefile: &str) -> Result<(), RomErr> {
		const HEADER: &'static [u8; 4] = &['R' as u8, 'N' as u8, 'E' as u8, 'S' as u8];

		if !self.support_savestates() {
			return Ok(());
		}

		let mut f =
			fs::OpenOptions::new().write(true).truncate(true).create(true).open(savefile).unwrap();

		f.write(HEADER).or_else(|_| Err(RomErr::SavefileWrite))?;
		f.write(self.get_battery_ram()).or_else(|_| Err(RomErr::SavefileWrite))?;

		Ok(())
	}
}

pub trait LoadRom {
	fn load(data: &[u8], info: &CartridgeInfo) -> Box<dyn Cartridge>;
}

#[derive(Default)]
pub struct CartridgeInfo {
	pub mapper_id: u8,
	pub prg_rom_cnt: usize,
	pub prg_ram_cnt: usize,
	pub chr_rom_cnt: usize,
	pub battery_ram: bool,
	pub ppu_mirror: PpuMirror,
}

#[derive(Debug, Clone)]
pub enum PpuMirror {
	Horizontal,
	Vertical,
	OneScreen,
	FourScreen,
	Other,
}

pub enum CartridgeErr {
	NotImplemented(u8),
	Unknown,
}

impl std::fmt::Debug for CartridgeErr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::NotImplemented(id) => write!(f, "NotImplemented, mapper ID: {}", id),
			Self::Unknown => write!(f, "Unknown"),
		}
	}
}

pub fn load(data: &[u8], info: &CartridgeInfo) -> Result<Box<dyn Cartridge>, CartridgeErr> {
	println!("Mapper ID: {}", info.mapper_id);

	match info.mapper_id {
		// add new mappers here
		0 => Ok(nrom::NRom::load(data, info)),
		1 => Ok(mmc1::Mmc1::load(data, info)),
		2 => Ok(uxrom::UxRom::load(data, info)),
		3 => Ok(cnrom::CNRom::load(data, info)),
		4 => Ok(mmc3::Mmc3::load(data, info)),
		_ => Err(CartridgeErr::NotImplemented(info.mapper_id)),
	}
}

impl Default for PpuMirror {
	fn default() -> Self {
		PpuMirror::Horizontal
	}
}

impl std::fmt::Display for PpuMirror {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PpuMirror::Horizontal => write!(f, "Horizontal"),
			PpuMirror::Vertical => write!(f, "Vertical"),
			PpuMirror::OneScreen => write!(f, "OneScreen"),
			PpuMirror::FourScreen => write!(f, "FourScreen"),
			PpuMirror::Other => write!(f, "Other"),
		}
	}
}
