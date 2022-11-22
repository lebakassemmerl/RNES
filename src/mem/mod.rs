mod ram;

use self::ram::Ram;
use crate::cartridge::Cartridge;
use crate::io::{iocontrol::IOControl, JoyPad};
use crate::mask;
use crate::ppu::ppu_regs::PpuRegisters;

const CPU_RAM_SIZE: usize = 0x800;
const OAM_SIZE: usize = 0x100;
const PALETTE_SIZE: usize = 0x20;
const PALETTE_MIRROR: usize = mask!(usize, 1, 4, true);

pub trait Segment {
	fn read(&self, addr: usize) -> u8;
	fn write(&mut self, addr: usize, val: u8);
}

pub trait PpuSegment {
	fn read(&mut self, addr: usize) -> u8;
	fn write(&mut self, addr: usize, val: u8);
	fn irq(&mut self) -> bool;
}

pub trait BankedSegment {
	fn read(&self, bank_idx: usize, addr: usize) -> u8;
	fn write(&mut self, bank_idx: usize, addr: usize, val: u8);
}

pub trait PpuBus {
	fn read(&mut self, addr: usize) -> u8;
	fn write(&mut self, addr: usize, val: u8);
	fn assert_nmi(&mut self);
	fn ppu_reg(&mut self) -> &mut PpuRegisters;
	fn oam(&mut self) -> &mut Ram<OAM_SIZE>;
}

pub trait CpuBus: PpuBus {
	fn read(&mut self, addr: usize) -> u8;
	fn write(&mut self, addr: usize, val: u8);
	fn dma_transfer_occurred(&mut self);
}

pub trait PpuRegisterAccess {
	fn ppu_ctrl_write(&mut self, val: u8);
	fn ppu_mask_write(&mut self, val: u8);
	fn ppu_scroll_write(&mut self, val: u8);
	fn ppu_addr_write(&mut self, val: u8);
	fn ppu_data_write(&mut self);
	fn oam_addr_write(&mut self, val: u8);
	fn oam_data_write(&mut self, val: u8);

	fn ppu_stat_read(&mut self) -> u8;
	fn ppu_addr_get(&self) -> u16;
	fn ppu_data_read(&mut self, new_val: u8) -> u8;
	fn oam_data_read(&self) -> u8;
}

// We do not include the VRAM of the PPU in this map, we include it in the individual mappers.
// This makes the whole mirroring way easier and the mappers are allocated on the heap anyway.
pub struct MemoryMap {
	cpu_ram: Ram<CPU_RAM_SIZE>,
	oam: Ram<OAM_SIZE>,
	palette_ram: Ram<PALETTE_SIZE>,
	ppu_regs: PpuRegisters,
	cartridge: Box<dyn Cartridge>,

	ioctrl: IOControl,

	dma_happened: bool,
	nmi_asserted: bool,
	irq_asserted: bool,
}

impl MemoryMap {
	pub fn new(cartridge: Box<dyn Cartridge>) -> Self {
		MemoryMap {
			cpu_ram: Ram::empty(0x00),
			oam: Ram::empty(0xFF),
			palette_ram: Ram::empty(0x00),
			ppu_regs: PpuRegisters::new(),
			cartridge: cartridge,
			ioctrl: IOControl::new(true, true),
			dma_happened: false,
			nmi_asserted: false,
			irq_asserted: false,
		}
	}

	pub fn button_update(&mut self, btns: [JoyPad; 2]) {
		self.ioctrl.refresh_controller(&btns[0], &btns[1]);
	}

	pub fn cartridge(&mut self) -> &mut Box<dyn Cartridge> {
		&mut self.cartridge
	}

	pub fn get_dma(&mut self) -> bool {
		let ret = self.dma_happened;
		self.dma_happened = false;

		ret
	}

	pub fn get_nmi(&mut self) -> bool {
		let ret = self.nmi_asserted;
		self.nmi_asserted = false;

		ret
	}

	pub fn get_irq(&mut self) -> bool {
		let ret = self.irq_asserted;
		self.irq_asserted = false;

		ret
	}

	#[allow(dead_code)]
	fn dump_name_table(&mut self, idx: usize) {
		const COLUMNS: usize = 32;
		const ROWS: usize = 30;
		const ATTR_COLUMS: usize = COLUMNS / 4;
		const ATTR_ROWS: usize = COLUMNS / 4;

		print!("  ");
		for i in 0..COLUMNS {
			print!("|{:02x}", i);
		}
		print!("|\n");

		print!("--");
		for _ in 0..COLUMNS {
			print!("+--");
		}
		print!("+\n");

		for y in 0..ROWS {
			print!("{:02x}|", y);
			for x in 0..COLUMNS {
				let v = PpuBus::read(self, 0x2000 + idx * 0x400 + y * COLUMNS + x);
				if x < (COLUMNS - 1) {
					print!("{:02x} ", v)
				} else {
					print!("{:02x}", v)
				}
			}
			print!("|\n");
		}

		print!("--+");
		for _ in 0..COLUMNS {
			print!("--+");
		}
		print!("\n  ");

		for i in 0..ATTR_COLUMS {
			print!("|     {:02x}    ", i);
		}
		print!("|\n==");
		for _ in 0..ATTR_COLUMS {
			print!("+===========");
		}
		print!("+\n");

		for y in 0..ATTR_ROWS {
			print!("{:02X}", y);
			for x in 0..ATTR_COLUMS {
				let v = PpuBus::read(self, 0x23C0 + idx * 0x400 + y * ATTR_COLUMS + x);
				print!("|    x{:02x}    ", v);
			}
			print!("|\n");

			print!("--");
			for _ in 0..ATTR_COLUMS {
				print!("+-----------");
			}
			print!("+\n");
		}
	}

	#[allow(dead_code)]
	fn dump_palette(&self) {
		println!("    | 00 | 01 | 02 | 03 |");
		println!("----+----+----+----+----+");

		for p in 0..8 {
			if p < 4 {
				print!("bg {:01x}", p);
			} else {
				print!("sp {:01x}", p - 4);
			}

			for i in 0..4 {
				print!("| {:02x} ", self.palette_ram.read(p * 4 + i));
			}
			print!("|\n----+----+----+----+----+\n");
		}
	}

	#[allow(dead_code)]
	fn dump_oam(&self) {
		println!("     || y0 | id | at | x0 || y1 | id | at | x1 || y2 | id | at | x2 || y3 | id | at | x3 ||");
		println!("-----++----+----+----+----++----+----+----+----++----+----+----+----++----+----+----+----++");

		for s in 0..16 {
			print!("sp {:02x}|", s * 4);
			for i in 0..4 {
				for j in 0..4 {
					let idx = s * 16 + i * 4 + j;
					print!("| {:02x} ", self.oam.read(idx));
				}
				print!("|");
			}
			print!("|\n-----++----+----+----+----++----+----+----+----++----+----+----+----++----+----+----+----++\n");
		}
		println!("");
	}

	pub fn dump(&self) {
		// println!("\nnametable 0\n");
		// self.dump_name_table(0);
		// println!("\nnametable 1\n");
		// self.dump_name_table(1);
		// println!("\ncolor palette\n");
		// self.dump_palette();

		// println!("\nOAM memory\n");
		self.dump_oam();
	}
}

impl CpuBus for MemoryMap {
	fn read(&mut self, addr: usize) -> u8 {
		match addr {
			0x0..=0x1FFF => self.cpu_ram.read(addr),
			0x2000..=0x3FFF => {
				let reg = addr & 0x07;
				match reg {
					2 => self.ppu_regs.ppu_stat_read(),
					4 => self.ppu_regs.oam_data_read(),
					7 => {
						// read the actual value into the data register and return the previous one
						let new_addr = self.ppu_regs.ppu_addr_get() as usize;
						let new_data = PpuBus::read(self, new_addr);
						self.ppu_regs.ppu_data_read(new_data)
					}
					_ => {
						panic!(
							"CpuBus read(): trying to read from a writeonly PPU register: {}",
							reg
						)
					}
				}
			}
			0x4000..=0x4013 | 0x4015 => {
				0
				// todo!("CpuBus::read(): APU is not supported yet, addr: 0x{:x}", addr)
			}
			0x4016 => self.ioctrl.read_controller1(),
			0x4017 => self.ioctrl.read_controller2(),
			0x4020..=0xFFFF => Segment::read(self.cartridge.as_ref(), addr),
			_ => panic!("CpuBus::read(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x0..=0x1FFF => self.cpu_ram.write(addr, val),
			0x2000..=0x3FFF => {
				let reg = addr & 0x07;
				match reg {
					0 => self.ppu_regs.ppu_ctrl_write(val),
					1 => self.ppu_regs.ppu_mask_write(val),
					3 => self.ppu_regs.oam_addr_write(val),
					4 => self.ppu_regs.oam_data_write(val),
					5 => self.ppu_regs.ppu_scroll_write(val),
					6 => self.ppu_regs.ppu_addr_write(val),
					7 => {
						// get address, mask probably not necessary
						let addr = self.ppu_regs.ppu_addr_get() as usize;
						// write value to address
						PpuBus::write(self, addr, val);
						// trigger further actions when writing to the PPU Data register
						self.ppu_regs.ppu_data_write();
					}
					_ => {
						panic!("CpuBus write(): trying to write to readonly PPU register: {}", reg)
					}
				}
			}
			0x4014 => {
				let page = (val as usize) << 8;
				for i in 0x00..0x100usize {
					let data = CpuBus::read(self, page + i);
					self.oam.write(i, data);
				}

				CpuBus::dma_transfer_occurred(self);
			}
			0x4000..=0x4013 | 0x4015 => {
				// todo!("CpuBus::write(): APU is not supported yet, addr: 0x{:x}", addr)
			}
			0x4016 | 0x4017 => self.ioctrl.reload_controller(val),
			0x4200..=0xFFFF => Segment::write(self.cartridge.as_mut(), addr, val),
			_ => panic!("CpuBus::write(): address out of memory range: 0x{:x}", addr),
		}
	}

	fn dma_transfer_occurred(&mut self) {
		self.dma_happened = true;
	}
}

impl PpuBus for MemoryMap {
	fn read(&mut self, addr: usize) -> u8 {
		let ret = match addr {
			0x0000..=0x1FFF
			| 0x2000..=0x23FF
			| 0x3000..=0x33FF
			| 0x2400..=0x27FF
			| 0x3400..=0x37FF
			| 0x2800..=0x2BFF
			| 0x3800..=0x3BFF
			| 0x2C00..=0x2FFF
			| 0x3C00..=0x3EFF => PpuSegment::read(self.cartridge.as_mut(), addr),
			0x3F00..=0x3FFF => {
				let mut a = addr & (self.palette_ram.size() - 1);
				if (a & 0x03) == 0 {
					a &= PALETTE_MIRROR;
				}
				self.palette_ram.read(a)
			}
			_ => panic!("PpuBus::read(): address out of memory range: 0x{:x}", addr),
		};

		if self.cartridge.irq() {
			self.irq_asserted = true;
		}

		ret
	}

	fn write(&mut self, addr: usize, val: u8) {
		match addr {
			0x0000..=0x1FFF
			| 0x2000..=0x23FF
			| 0x3000..=0x33FF
			| 0x2400..=0x27FF
			| 0x3400..=0x37FF
			| 0x2800..=0x2BFF
			| 0x3800..=0x3BFF
			| 0x2C00..=0x2FFF
			| 0x3C00..=0x3EFF => PpuSegment::write(self.cartridge.as_mut(), addr, val),
			0x3F00..=0x3FFF => {
				let mut a = addr & (self.palette_ram.size() - 1);
				if (a & 0x03) == 0 {
					a &= PALETTE_MIRROR;
				}
				self.palette_ram.write(a, val);
			}
			_ => panic!("PpuBus::write(): address out of memory range: 0x{:x}", addr),
		}

		if self.cartridge.irq() {
			self.irq_asserted = true;
		}
	}

	fn assert_nmi(&mut self) {
		self.nmi_asserted = true;
	}

	fn ppu_reg(&mut self) -> &mut PpuRegisters {
		&mut self.ppu_regs
	}

	fn oam(&mut self) -> &mut Ram<OAM_SIZE> {
		&mut self.oam
	}
}
