use super::addressing::*;
use crate::cpu::{Cpu, CARRY_IDX, DECIMAL_IDX, INTERRUPT_IDX, OVERFLOW_IDX};
use crate::mem::CpuBus;
use std::marker::PhantomData;

macro_rules! register_instruction_set {
	($instr:ident, $bit:ident, $state:ident) => {
		pub struct $instr<A> {
			phantom: PhantomData<A>,
		}

		impl<B: CpuBus, A: AddressMode<B>> Operation<B> for $instr<A> {
			fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
				cpu.set_statusbit($state, $bit);
				None
			}
		}

		impl<B: CpuBus> AddressOperation<B, Implied> for $instr<Implied> {
			fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
				2
			}
		}
	};
}

macro_rules! register_instruction_compare {
	($instr:ident, $register:ident, $name:expr) => {
		pub struct $instr<A> {
			phantom: PhantomData<A>,
		}

		impl<B: CpuBus, A: AddressMode<B>> Operation<B> for $instr<A> {
			fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
				let op = CpuBus::read(mem, addr.expect(concat!($name, " requires an address")));
				let res = cpu.$register.wrapping_sub(op);
				cpu.set_carry(cpu.$register >= op);
				cpu.set_negative(res);
				cpu.set_zero(res);

				None
			}
		}

		impl<B: CpuBus> AddressOperation<B, Immediate> for $instr<Immediate> {
			fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
				2
			}
		}

		impl<B: CpuBus> AddressOperation<B, Zeropage> for $instr<Zeropage> {
			fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
				3
			}
		}

		impl<B: CpuBus> AddressOperation<B, Absolute> for $instr<Absolute> {
			fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
				4
			}
		}
	};
}

register_instruction_set!(Clc, CARRY_IDX, false);
register_instruction_set!(Cld, DECIMAL_IDX, false);
register_instruction_set!(Cli, INTERRUPT_IDX, false);
register_instruction_set!(Clv, OVERFLOW_IDX, false);

register_instruction_set!(Sec, CARRY_IDX, true);
register_instruction_set!(Sed, DECIMAL_IDX, true);
register_instruction_set!(Sei, INTERRUPT_IDX, true);

register_instruction_compare!(Cpx, x, "CPX");
register_instruction_compare!(Cpy, y, "CPY");
register_instruction_compare!(Cmp, a, "CMP");

// CMP supports a lot more addressmodes which aren't covered in the macro
// -> implement it here by hand
impl<B: CpuBus> AddressOperation<B, ZeropageX> for Cmp<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Cmp<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Cmp<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Cmp<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Cmp<IndirectY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			6
		} else {
			5
		}
	}
}
