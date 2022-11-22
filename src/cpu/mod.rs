mod addressing;
mod instructions;

use std::marker::PhantomData;

use crate::mem::CpuBus;

use addressing::*;
use instructions::*;

// interrupt-vector locations
const IRQ_VEC: u16 = 0xFFFE;
const BRK_VEC: u16 = 0xFFFE;
const RESET_VEC: u16 = 0xFFFC;
const NMI_VEC: u16 = 0xFFFA;

// status-register indexes
const CARRY_IDX: u8 = 0;
const ZERO_IDX: u8 = 1;
const INTERRUPT_IDX: u8 = 2;
const DECIMAL_IDX: u8 = 3;
const BRK_IDX: u8 = 4;
const UNUSED_IDX: u8 = 5;
const OVERFLOW_IDX: u8 = 6;
const NEGATIVE_IDX: u8 = 7;

#[derive(PartialEq)]
pub enum InterruptSource {
	RESET,
	NMI,
	IRQ,
	BRK,
	NONE,
}

#[derive(Default)]
pub struct Irq {
	pending: bool,
	src: InterruptSource,
}

#[derive(Default)]
pub struct CpuStat {
	interrupt_cnt: u64,
	cycle_cnt: u64,
	instr_cnt: u64,
}

#[derive(Default)]
pub struct Cpu<B: CpuBus> {
	pc: u16, // program counter
	sp: u8,  // stack pointer
	a: u8,   // accumulator register
	x: u8,   // index register x
	y: u8,   // index register y
	p: u8,   // processor status

	skip_cycles: usize, // cycles to skip from last opcode
	irq: Irq,           // interrupt-infos
	stat: CpuStat,      // some debug-info

	_phantom: PhantomData<B>,
}

impl Default for InterruptSource {
	fn default() -> Self {
		InterruptSource::NONE
	}
}

impl<B: CpuBus> Cpu<B> {
	pub fn new() -> Self {
		Self {
			pc: 0,
			sp: 0,
			a: 0,
			x: 0,
			y: 0,
			p: 0,
			skip_cycles: 0,
			irq: Default::default(),
			stat: Default::default(),
			_phantom: PhantomData,
		}
	}

	pub fn assert_interrupt(&mut self, src: InterruptSource) {
		self.irq.pending = true;
		self.irq.src = src;
	}

	// only for testing-reasons!!!
	pub fn set_pc(&mut self, addr: u16) {
		self.pc = addr;
		self.stat.cycle_cnt = 7;
	}

	pub fn log_cpu_stats(&self, mem: &mut B) -> String {
		let op = CpuBus::read(mem, self.pc as usize);
		String::from(format!(
			"{:04X}  {:02X}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
			self.pc, op, self.a, self.x, self.y, self.p, self.sp, self.stat.cycle_cnt
		))
	}

	fn reset(&mut self) {
		self.pc = 0;
		self.sp = 0xFD;
		self.a = 0;
		self.x = 0;
		self.y = 0;
		self.p = (1 << UNUSED_IDX) | (1 << INTERRUPT_IDX); // set interrupt-disable bit on startup
		self.irq.pending = false;
		self.irq.src = InterruptSource::NONE;
		self.stat.interrupt_cnt = 0;
		self.stat.cycle_cnt = 0;
		self.stat.instr_cnt = 0;
	}

	fn read16(mem: &mut B, addr: usize) -> u16 {
		let l = CpuBus::read(mem, addr) as u16;
		let h = CpuBus::read(mem, addr + 1) as u16;
		(h << 8) | l
	}

	fn push8(&mut self, mem: &mut B, val: u8) {
		CpuBus::write(mem, (self.sp as usize) + 0x100, val);
		self.sp = self.sp.wrapping_sub(1);
	}

	fn push16(&mut self, mem: &mut B, val: u16) {
		self.push8(mem, (val >> 8) as u8);
		self.push8(mem, (val & 0xFF) as u8);
	}

	fn pop8(&mut self, mem: &mut B) -> u8 {
		self.sp = self.sp.wrapping_add(1);
		CpuBus::read(mem, (self.sp as usize) + 0x100)
	}

	fn pop16(&mut self, mem: &mut B) -> u16 {
		let l = self.pop8(mem) as u16;
		let h = self.pop8(mem) as u16;

		(h << 8) | l
	}

	fn set_statusbit(&mut self, bit: bool, idx: u8) {
		self.p = (self.p & !(1 << idx)) | ((bit as u8) << idx);
	}

	fn get_statusbit(&self, idx: u8) -> bool {
		self.p & (1 << idx) > 0
	}

	fn set_negative(&mut self, val: u8) {
		self.set_statusbit(val > 0x7F, NEGATIVE_IDX);
	}

	fn is_negative(&self) -> bool {
		self.get_statusbit(NEGATIVE_IDX)
	}

	fn set_overflow(&mut self, x: u8, y: u8, res: u8) {
		let bit = (!(x ^ y) & (x ^ res) & 0x80) > 0;
		self.set_statusbit(bit, OVERFLOW_IDX);
	}

	fn set_overflow_raw(&mut self, bit: bool) {
		self.set_statusbit(bit, OVERFLOW_IDX);
	}

	fn is_overflow(&self) -> bool {
		self.get_statusbit(OVERFLOW_IDX)
	}

	fn set_break(&mut self, state: bool) {
		self.set_statusbit(state, BRK_IDX);
	}

	fn set_unused(&mut self, state: bool) {
		self.set_statusbit(state, UNUSED_IDX);
	}

	fn set_interrupt(&mut self, state: bool) {
		self.set_statusbit(state, INTERRUPT_IDX);
	}

	fn set_zero(&mut self, val: u8) {
		self.set_statusbit(val == 0x00, ZERO_IDX);
	}

	fn is_zero(&self) -> bool {
		self.get_statusbit(ZERO_IDX)
	}

	fn set_carry(&mut self, state: bool) {
		self.set_statusbit(state, CARRY_IDX);
	}

	fn is_carry(&self) -> bool {
		self.get_statusbit(CARRY_IDX)
	}

	fn interrupt(&mut self, mem: &mut B) -> bool {
		if !self.irq.pending {
			return false;
		}
		self.irq.pending = false;

		match &self.irq.src {
			InterruptSource::RESET => {
				self.reset();
				self.pc = Self::read16(mem, RESET_VEC as usize);
			}
			InterruptSource::NMI => {
				self.push16(mem, self.pc);
				let mut flags = self.p | (1 << UNUSED_IDX);
				flags &= !(1 << BRK_IDX);
				self.push8(mem, flags);

				self.pc = Self::read16(mem, NMI_VEC as usize);
			}
			InterruptSource::IRQ => {
				self.push16(mem, self.pc);
				let mut flags = self.p | (1 << UNUSED_IDX);
				flags &= !(1 << BRK_IDX);
				self.push8(mem, flags);

				self.set_interrupt(true);
				self.pc = Self::read16(mem, IRQ_VEC as usize);
			}
			InterruptSource::BRK => {
				self.push16(mem, self.pc);
				let flags = self.p | (1 << UNUSED_IDX) | (1 << BRK_IDX);
				self.push8(mem, flags);

				self.set_interrupt(true);
				self.pc = Self::read16(mem, BRK_VEC as usize);
			}
			_ => {}
		}

		self.irq.src = InterruptSource::NONE;
		true
	}

	pub fn dma_transaction_occurred(&mut self) {
		self.skip_cycles += if (self.stat.cycle_cnt & 0x01) > 0 {
			// if cycle count is odd, the CPU is stalled for 1 additional cycle
			514
		} else {
			513
		}
	}

	// cpu execution
	fn exec_instruction(&mut self, mem: &mut B) -> usize {
		let instr = CpuBus::read(mem, self.pc as usize);
		self.pc += 1;

		match instr {
			0x00 => system::Brk::<Implied>::do_instruction(self, mem),
			0x01 => bitwise::Ora::<IndirectX>::do_instruction(self, mem),
			0x03 => undocumented::Slo::<IndirectX>::do_instruction(self, mem),
			0x04 => system::Nop::<Zeropage>::do_instruction(self, mem), // undocumented
			0x05 => bitwise::Ora::<Zeropage>::do_instruction(self, mem),
			0x06 => bitwise::Asl::<Zeropage>::do_instruction(self, mem),
			0x07 => undocumented::Slo::<Zeropage>::do_instruction(self, mem),
			0x08 => stack::Php::<Implied>::do_instruction(self, mem),
			0x09 => bitwise::Ora::<Immediate>::do_instruction(self, mem),
			0x0A => bitwise::AslA::<Accumulator>::do_instruction(self, mem),
			0x0B => undocumented::Anc::<Immediate>::do_instruction(self, mem),
			0x0C => system::Nop::<Absolute>::do_instruction(self, mem), // undocumented
			0x0D => bitwise::Ora::<Absolute>::do_instruction(self, mem),
			0x0E => bitwise::Asl::<Absolute>::do_instruction(self, mem),
			0x0F => undocumented::Slo::<Absolute>::do_instruction(self, mem),
			0x10 => branch::Bpl::<Relative>::do_instruction(self, mem),
			0x11 => bitwise::Ora::<IndirectY>::do_instruction(self, mem),
			0x13 => undocumented::Slo::<IndirectY>::do_instruction(self, mem),
			0x14 => system::Nop::<ZeropageX>::do_instruction(self, mem), // undocumented
			0x15 => bitwise::Ora::<ZeropageX>::do_instruction(self, mem),
			0x16 => bitwise::Asl::<ZeropageX>::do_instruction(self, mem),
			0x17 => undocumented::Slo::<ZeropageX>::do_instruction(self, mem), // undocumented
			0x18 => registers::Clc::<Implied>::do_instruction(self, mem),
			0x19 => bitwise::Ora::<AbsoluteY>::do_instruction(self, mem),
			0x1A => system::Nop::<Implied>::do_instruction(self, mem), // undocumented
			0x1B => undocumented::Slo::<AbsoluteY>::do_instruction(self, mem),
			0x1C => system::Nop::<AbsoluteX>::do_instruction(self, mem), // undocumented
			0x1D => bitwise::Ora::<AbsoluteX>::do_instruction(self, mem),
			0x1E => bitwise::Asl::<AbsoluteX>::do_instruction(self, mem),
			0x1F => undocumented::Slo::<AbsoluteX>::do_instruction(self, mem),
			0x20 => jump::Jsr::<Absolute>::do_instruction(self, mem),
			0x21 => bitwise::And::<IndirectX>::do_instruction(self, mem),
			0x23 => undocumented::Rla::<IndirectX>::do_instruction(self, mem),
			0x24 => bitwise::Bit::<Zeropage>::do_instruction(self, mem),
			0x25 => bitwise::And::<Zeropage>::do_instruction(self, mem),
			0x26 => bitwise::Rol::<Zeropage>::do_instruction(self, mem),
			0x27 => undocumented::Rla::<Zeropage>::do_instruction(self, mem),
			0x28 => stack::Plp::<Implied>::do_instruction(self, mem),
			0x29 => bitwise::And::<Immediate>::do_instruction(self, mem),
			0x2A => bitwise::RolA::<Accumulator>::do_instruction(self, mem),
			0x2B => undocumented::Anc::<Immediate>::do_instruction(self, mem),
			0x2C => bitwise::Bit::<Absolute>::do_instruction(self, mem),
			0x2D => bitwise::And::<Absolute>::do_instruction(self, mem),
			0x2E => bitwise::Rol::<Absolute>::do_instruction(self, mem),
			0x2F => undocumented::Rla::<Absolute>::do_instruction(self, mem),
			0x30 => branch::Bmi::<Relative>::do_instruction(self, mem),
			0x31 => bitwise::And::<IndirectY>::do_instruction(self, mem),
			0x33 => undocumented::Rla::<IndirectY>::do_instruction(self, mem),
			0x34 => system::Nop::<ZeropageX>::do_instruction(self, mem), // undocumented
			0x35 => bitwise::And::<ZeropageX>::do_instruction(self, mem),
			0x36 => bitwise::Rol::<ZeropageX>::do_instruction(self, mem),
			0x37 => undocumented::Rla::<ZeropageX>::do_instruction(self, mem),
			0x38 => registers::Sec::<Implied>::do_instruction(self, mem),
			0x39 => bitwise::And::<AbsoluteY>::do_instruction(self, mem),
			0x3A => system::Nop::<Implied>::do_instruction(self, mem), // undocumented
			0x3C => system::Nop::<AbsoluteX>::do_instruction(self, mem), // undocumented
			0x3B => undocumented::Rla::<AbsoluteY>::do_instruction(self, mem),
			0x3D => bitwise::And::<AbsoluteX>::do_instruction(self, mem),
			0x3E => bitwise::Rol::<AbsoluteX>::do_instruction(self, mem),
			0x3F => undocumented::Rla::<AbsoluteX>::do_instruction(self, mem),
			0x40 => jump::Rti::<Implied>::do_instruction(self, mem),
			0x41 => bitwise::Eor::<IndirectX>::do_instruction(self, mem),
			0x43 => undocumented::Sre::<IndirectX>::do_instruction(self, mem),
			0x44 => system::Nop::<Zeropage>::do_instruction(self, mem), // undocumented
			0x45 => bitwise::Eor::<Zeropage>::do_instruction(self, mem),
			0x46 => bitwise::Lsr::<Zeropage>::do_instruction(self, mem),
			0x47 => undocumented::Sre::<Zeropage>::do_instruction(self, mem),
			0x48 => stack::Pha::<Implied>::do_instruction(self, mem),
			0x49 => bitwise::Eor::<Immediate>::do_instruction(self, mem),
			0x4A => bitwise::LsrA::<Accumulator>::do_instruction(self, mem),
			0x4B => undocumented::Alr::<Immediate>::do_instruction(self, mem),
			0x4C => jump::Jmp::<Absolute>::do_instruction(self, mem),
			0x4D => bitwise::Eor::<Absolute>::do_instruction(self, mem),
			0x4E => bitwise::Lsr::<Absolute>::do_instruction(self, mem),
			0x4F => undocumented::Sre::<Absolute>::do_instruction(self, mem),
			0x50 => branch::Bvc::<Relative>::do_instruction(self, mem),
			0x51 => bitwise::Eor::<IndirectY>::do_instruction(self, mem),
			0x53 => undocumented::Sre::<IndirectY>::do_instruction(self, mem),
			0x54 => system::Nop::<ZeropageX>::do_instruction(self, mem), // undocumented
			0x55 => bitwise::Eor::<ZeropageX>::do_instruction(self, mem),
			0x56 => bitwise::Lsr::<ZeropageX>::do_instruction(self, mem),
			0x57 => undocumented::Sre::<ZeropageX>::do_instruction(self, mem),
			0x58 => registers::Cli::<Implied>::do_instruction(self, mem),
			0x59 => bitwise::Eor::<AbsoluteY>::do_instruction(self, mem),
			0x5A => system::Nop::<Implied>::do_instruction(self, mem), // undocumented
			0x5B => undocumented::Sre::<AbsoluteY>::do_instruction(self, mem),
			0x5C => system::Nop::<AbsoluteX>::do_instruction(self, mem), // undocumented
			0x5D => bitwise::Eor::<AbsoluteX>::do_instruction(self, mem),
			0x5E => bitwise::Lsr::<AbsoluteX>::do_instruction(self, mem),
			0x5F => undocumented::Sre::<AbsoluteX>::do_instruction(self, mem),
			0x60 => jump::Rts::<Implied>::do_instruction(self, mem),
			0x61 => math::Adc::<IndirectX>::do_instruction(self, mem),
			0x63 => undocumented::Rra::<IndirectX>::do_instruction(self, mem),
			0x64 => system::Nop::<Zeropage>::do_instruction(self, mem), // undocumented
			0x65 => math::Adc::<Zeropage>::do_instruction(self, mem),
			0x66 => bitwise::Ror::<Zeropage>::do_instruction(self, mem),
			0x67 => undocumented::Rra::<Zeropage>::do_instruction(self, mem),
			0x68 => stack::Pla::<Implied>::do_instruction(self, mem),
			0x69 => math::Adc::<Immediate>::do_instruction(self, mem),
			0x6A => bitwise::RorA::<Accumulator>::do_instruction(self, mem),
			0x6B => undocumented::Arr::<Immediate>::do_instruction(self, mem),
			0x6C => jump::Jmp::<Indirect>::do_instruction(self, mem),
			0x6D => math::Adc::<Absolute>::do_instruction(self, mem),
			0x6E => bitwise::Ror::<Absolute>::do_instruction(self, mem),
			0x6F => undocumented::Rra::<Absolute>::do_instruction(self, mem),
			0x70 => branch::Bvs::<Relative>::do_instruction(self, mem),
			0x71 => math::Adc::<IndirectY>::do_instruction(self, mem),
			0x73 => undocumented::Rra::<IndirectY>::do_instruction(self, mem),
			0x74 => system::Nop::<ZeropageX>::do_instruction(self, mem), // undocumented
			0x75 => math::Adc::<ZeropageX>::do_instruction(self, mem),
			0x76 => bitwise::Ror::<ZeropageX>::do_instruction(self, mem),
			0x77 => undocumented::Rra::<ZeropageX>::do_instruction(self, mem),
			0x78 => registers::Sei::<Implied>::do_instruction(self, mem),
			0x79 => math::Adc::<AbsoluteY>::do_instruction(self, mem),
			0x7A => system::Nop::<Implied>::do_instruction(self, mem), // undocumented
			0x7B => undocumented::Rra::<AbsoluteY>::do_instruction(self, mem),
			0x7C => system::Nop::<AbsoluteX>::do_instruction(self, mem), // undocumented
			0x7D => math::Adc::<AbsoluteX>::do_instruction(self, mem),
			0x7E => bitwise::Ror::<AbsoluteX>::do_instruction(self, mem),
			0x7F => undocumented::Rra::<AbsoluteX>::do_instruction(self, mem),
			0x80 => system::Nop::<Immediate>::do_instruction(self, mem), // undocumented
			0x81 => storage::Sta::<IndirectX>::do_instruction(self, mem),
			0x82 => system::Nop::<Immediate>::do_instruction(self, mem), // undocumented
			0x83 => undocumented::Sax::<IndirectX>::do_instruction(self, mem),
			0x84 => storage::Sty::<Zeropage>::do_instruction(self, mem),
			0x85 => storage::Sta::<Zeropage>::do_instruction(self, mem),
			0x86 => storage::Stx::<Zeropage>::do_instruction(self, mem),
			0x87 => undocumented::Sax::<Zeropage>::do_instruction(self, mem),
			0x88 => math::Dey::<Implied>::do_instruction(self, mem),
			0x89 => system::Nop::<Immediate>::do_instruction(self, mem), // undocumented
			0x8A => storage::Txa::<Implied>::do_instruction(self, mem),
			0x8B => undocumented::Ane::<Immediate>::do_instruction(self, mem),
			0x8C => storage::Sty::<Absolute>::do_instruction(self, mem),
			0x8D => storage::Sta::<Absolute>::do_instruction(self, mem),
			0x8E => storage::Stx::<Absolute>::do_instruction(self, mem),
			0x8F => undocumented::Sax::<Absolute>::do_instruction(self, mem),
			0x90 => branch::Bcc::<Relative>::do_instruction(self, mem),
			0x91 => storage::Sta::<IndirectY>::do_instruction(self, mem),
			0x93 => undocumented::Sha::<IndirectY>::do_instruction(self, mem),
			0x94 => storage::Sty::<ZeropageX>::do_instruction(self, mem),
			0x95 => storage::Sta::<ZeropageX>::do_instruction(self, mem),
			0x96 => storage::Stx::<ZeropageY>::do_instruction(self, mem),
			0x97 => undocumented::Sax::<ZeropageY>::do_instruction(self, mem),
			0x98 => storage::Tya::<Implied>::do_instruction(self, mem),
			0x99 => storage::Sta::<AbsoluteY>::do_instruction(self, mem),
			0x9A => storage::Txs::<Implied>::do_instruction(self, mem),
			0x9B => undocumented::Tas::<AbsoluteY>::do_instruction(self, mem),
			0x9C => undocumented::Shy::<AbsoluteX>::do_instruction(self, mem),
			0x9D => storage::Sta::<AbsoluteX>::do_instruction(self, mem),
			0x9E => undocumented::Shx::<AbsoluteY>::do_instruction(self, mem),
			0x9F => undocumented::Sha::<AbsoluteY>::do_instruction(self, mem),
			0xA0 => storage::Ldy::<Immediate>::do_instruction(self, mem),
			0xA1 => storage::Lda::<IndirectX>::do_instruction(self, mem),
			0xA2 => storage::Ldx::<Immediate>::do_instruction(self, mem),
			0xA3 => undocumented::Lax::<IndirectX>::do_instruction(self, mem),
			0xA4 => storage::Ldy::<Zeropage>::do_instruction(self, mem),
			0xA5 => storage::Lda::<Zeropage>::do_instruction(self, mem),
			0xA6 => storage::Ldx::<Zeropage>::do_instruction(self, mem),
			0xA7 => undocumented::Lax::<Zeropage>::do_instruction(self, mem),
			0xA8 => storage::Tay::<Implied>::do_instruction(self, mem),
			0xA9 => storage::Lda::<Immediate>::do_instruction(self, mem),
			0xAA => storage::Tax::<Implied>::do_instruction(self, mem),
			0xAB => undocumented::Lxa::<Immediate>::do_instruction(self, mem),
			0xAC => storage::Ldy::<Absolute>::do_instruction(self, mem),
			0xAD => storage::Lda::<Absolute>::do_instruction(self, mem),
			0xAE => storage::Ldx::<Absolute>::do_instruction(self, mem),
			0xAF => undocumented::Lax::<Absolute>::do_instruction(self, mem),
			0xB0 => branch::Bcs::<Relative>::do_instruction(self, mem),
			0xB1 => storage::Lda::<IndirectY>::do_instruction(self, mem),
			0xB3 => undocumented::Lax::<IndirectY>::do_instruction(self, mem),
			0xB4 => storage::Ldy::<ZeropageX>::do_instruction(self, mem),
			0xB5 => storage::Lda::<ZeropageX>::do_instruction(self, mem),
			0xB6 => storage::Ldx::<ZeropageY>::do_instruction(self, mem),
			0xB7 => undocumented::Lax::<ZeropageY>::do_instruction(self, mem),
			0xB8 => registers::Clv::<Implied>::do_instruction(self, mem),
			0xB9 => storage::Lda::<AbsoluteY>::do_instruction(self, mem),
			0xBA => storage::Tsx::<Implied>::do_instruction(self, mem),
			0xBC => storage::Ldy::<AbsoluteX>::do_instruction(self, mem),
			0xBB => undocumented::Las::<AbsoluteY>::do_instruction(self, mem),
			0xBD => storage::Lda::<AbsoluteX>::do_instruction(self, mem),
			0xBE => storage::Ldx::<AbsoluteY>::do_instruction(self, mem),
			0xBF => undocumented::Lax::<AbsoluteY>::do_instruction(self, mem),
			0xC0 => registers::Cpy::<Immediate>::do_instruction(self, mem),
			0xC1 => registers::Cmp::<IndirectX>::do_instruction(self, mem),
			0xC2 => system::Nop::<Immediate>::do_instruction(self, mem), // undocumented
			0xC3 => undocumented::Dcp::<IndirectX>::do_instruction(self, mem),
			0xC4 => registers::Cpy::<Zeropage>::do_instruction(self, mem),
			0xC5 => registers::Cmp::<Zeropage>::do_instruction(self, mem),
			0xC6 => math::Dec::<Zeropage>::do_instruction(self, mem),
			0xC7 => undocumented::Dcp::<Zeropage>::do_instruction(self, mem),
			0xC8 => math::Iny::<Implied>::do_instruction(self, mem),
			0xC9 => registers::Cmp::<Immediate>::do_instruction(self, mem),
			0xCA => math::Dex::<Implied>::do_instruction(self, mem),
			0xCB => undocumented::Sbx::<Immediate>::do_instruction(self, mem),
			0xCC => registers::Cpy::<Absolute>::do_instruction(self, mem),
			0xCD => registers::Cmp::<Absolute>::do_instruction(self, mem),
			0xCE => math::Dec::<Absolute>::do_instruction(self, mem),
			0xCF => undocumented::Dcp::<Absolute>::do_instruction(self, mem),
			0xD0 => branch::Bne::<Relative>::do_instruction(self, mem),
			0xD1 => registers::Cmp::<IndirectY>::do_instruction(self, mem),
			0xD3 => undocumented::Dcp::<IndirectY>::do_instruction(self, mem),
			0xD4 => system::Nop::<ZeropageX>::do_instruction(self, mem), // undocumented
			0xD5 => registers::Cmp::<ZeropageX>::do_instruction(self, mem),
			0xD6 => math::Dec::<ZeropageX>::do_instruction(self, mem),
			0xD7 => undocumented::Dcp::<ZeropageX>::do_instruction(self, mem),
			0xD8 => registers::Cld::<Implied>::do_instruction(self, mem),
			0xD9 => registers::Cmp::<AbsoluteY>::do_instruction(self, mem),
			0xDA => system::Nop::<Implied>::do_instruction(self, mem), // undocumented
			0xDB => undocumented::Dcp::<AbsoluteY>::do_instruction(self, mem),
			0xDC => system::Nop::<AbsoluteX>::do_instruction(self, mem), // undocumented
			0xDD => registers::Cmp::<AbsoluteX>::do_instruction(self, mem),
			0xDE => math::Dec::<AbsoluteX>::do_instruction(self, mem),
			0xDF => undocumented::Dcp::<AbsoluteX>::do_instruction(self, mem),
			0xE0 => registers::Cpx::<Immediate>::do_instruction(self, mem),
			0xE1 => math::Sbc::<IndirectX>::do_instruction(self, mem),
			0xE2 => system::Nop::<Immediate>::do_instruction(self, mem), // undocumented
			0xE3 => undocumented::Isc::<IndirectX>::do_instruction(self, mem),
			0xE4 => registers::Cpx::<Zeropage>::do_instruction(self, mem),
			0xE5 => math::Sbc::<Zeropage>::do_instruction(self, mem),
			0xE6 => math::Inc::<Zeropage>::do_instruction(self, mem),
			0xE7 => undocumented::Isc::<Zeropage>::do_instruction(self, mem),
			0xE8 => math::Inx::<Implied>::do_instruction(self, mem),
			0xE9 => math::Sbc::<Immediate>::do_instruction(self, mem),
			0xEA => system::Nop::<Implied>::do_instruction(self, mem),
			0xEB => math::Sbc::<Immediate>::do_instruction(self, mem), // undocumented
			0xEC => registers::Cpx::<Absolute>::do_instruction(self, mem),
			0xED => math::Sbc::<Absolute>::do_instruction(self, mem),
			0xEE => math::Inc::<Absolute>::do_instruction(self, mem),
			0xEF => undocumented::Isc::<Absolute>::do_instruction(self, mem),
			0xF0 => branch::Beq::<Relative>::do_instruction(self, mem),
			0xF1 => math::Sbc::<IndirectY>::do_instruction(self, mem),
			0xF3 => undocumented::Isc::<IndirectY>::do_instruction(self, mem),
			0xF4 => system::Nop::<ZeropageX>::do_instruction(self, mem), // undocumented
			0xF5 => math::Sbc::<ZeropageX>::do_instruction(self, mem),
			0xF6 => math::Inc::<ZeropageX>::do_instruction(self, mem),
			0xF7 => undocumented::Isc::<ZeropageX>::do_instruction(self, mem),
			0xF8 => registers::Sed::<Implied>::do_instruction(self, mem),
			0xF9 => math::Sbc::<AbsoluteY>::do_instruction(self, mem),
			0xFA => system::Nop::<Implied>::do_instruction(self, mem), // undocumented
			0xFB => undocumented::Isc::<AbsoluteY>::do_instruction(self, mem),
			0xFC => system::Nop::<AbsoluteX>::do_instruction(self, mem), // undocumented
			0xFD => math::Sbc::<AbsoluteX>::do_instruction(self, mem),
			0xFE => math::Inc::<AbsoluteX>::do_instruction(self, mem),
			0xFF => undocumented::Isc::<AbsoluteX>::do_instruction(self, mem),
			_ => {
				println!("reset op-code: 0x{:x}", instr);
				self.reset();
				0
			}
		}
	}

	pub fn step(&mut self, mem: &mut B) {
		self.stat.cycle_cnt += 1;

		// TODO:
		// For now we execute at the 1st cycle and skip the remaining ones. In order to do it
		// correct, we probably have to do it vice-versa -> skip first cycles and execute at the
		// last ones.
		if self.skip_cycles > 0 {
			self.skip_cycles -= 1;
			return;
		}

		let cycles = if self.interrupt(mem) {
			self.stat.interrupt_cnt += 1;
			7 // not sure if this is correct
		} else {
			self.stat.instr_cnt += 1;
			self.exec_instruction(mem)
		};

		// we already executed one of the needed cycles
		self.skip_cycles = cycles - 1;
	}
}
