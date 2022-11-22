use super::addressing::*;
use crate::cpu::Cpu;
use crate::mem::CpuBus;
use std::marker::PhantomData;

macro_rules! branch_instruction_set {
	($instr:ident, $bit_fn:ident, $instr_name:expr) => {
		pub struct $instr<A> {
			phantom: PhantomData<A>,
		}

		impl<B: CpuBus, A: AddressMode<B>> Operation<B> for $instr<A> {
			fn exec(cpu: &mut Cpu<B>, _mem: &mut B, addr: Option<usize>) -> Option<usize> {
				let op = addr.expect(concat!($instr_name, "requires an address")) as i8;
				Some(branch(cpu, op, cpu.$bit_fn()))
			}
		}

		impl<B: CpuBus> AddressOperation<B, Relative> for $instr<Relative> {
			fn cycles(extra_cycles: usize, _boundary: bool) -> usize {
				2 + extra_cycles
			}
		}
	};
}

macro_rules! branch_instruction_clear {
	($instr:ident, $bit_fn:ident, $instr_name:expr) => {
		pub struct $instr<A> {
			phantom: PhantomData<A>,
		}

		impl<B: CpuBus, A: AddressMode<B>> Operation<B> for $instr<A> {
			fn exec(cpu: &mut Cpu<B>, _mem: &mut B, addr: Option<usize>) -> Option<usize> {
				let op = addr.expect(concat!($instr_name, "requires an address")) as i8;
				Some(branch(cpu, op, !cpu.$bit_fn()))
			}
		}

		impl<B: CpuBus> AddressOperation<B, Relative> for $instr<Relative> {
			fn cycles(extra_cycles: usize, _boundary: bool) -> usize {
				2 + extra_cycles
			}
		}
	};
}

branch_instruction_set!(Bcs, is_carry, "BCS");
branch_instruction_set!(Beq, is_zero, "BEQ");
branch_instruction_set!(Bmi, is_negative, "BMI");
branch_instruction_set!(Bvs, is_overflow, "BVS");

branch_instruction_clear!(Bcc, is_carry, "BCC");
branch_instruction_clear!(Bne, is_zero, "BNE");
branch_instruction_clear!(Bpl, is_negative, "BPL");
branch_instruction_clear!(Bvc, is_overflow, "BVC");

fn branch<B: CpuBus>(cpu: &mut Cpu<B>, op: i8, flag: bool) -> usize {
	if flag {
		let pc_old = cpu.pc;
		let pc_new = (cpu.pc as i16) + (op as i16);
		cpu.pc = pc_new as u16;

		if (pc_old >> 8) == (cpu.pc >> 8) {
			// no page-boundary crossed
			1
		} else {
			// page-boundary crossed
			2
		}
	} else {
		0
	}
}
