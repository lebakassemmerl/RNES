use super::addressing::*;
use crate::cpu::{Cpu, InterruptSource};
use crate::mem::CpuBus;
use std::marker::PhantomData;

pub struct Nop<A>(PhantomData<A>);
pub struct Brk<A>(PhantomData<A>);

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Nop<A> {
	fn exec(_cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Nop<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// The following implemented addressmodes for the nop instructions are undocumented features
impl<B: CpuBus> AddressOperation<B, Immediate> for Nop<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Nop<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Nop<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Nop<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Nop<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Brk<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		// Note that BRK is weird in that the instruction is 1 byte, but the return address we store
		// is 2 bytes after the instruction, so the byte after BRK will be skipped upon return
		// (RTI). Usually an NOP is inserted after a BRK for this reason.
		cpu.push16(mem, cpu.pc + 1);
		cpu.push_processor_status(mem, true);
		cpu.set_interrupt_disable_bit(true);

		cpu.set_interrupt_disable_bit(true);
		cpu.pc = Cpu::<B>::read16(mem, Cpu::<B>::BRK_VEC as usize);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Brk<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}
