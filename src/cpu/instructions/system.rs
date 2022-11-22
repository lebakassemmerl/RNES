use super::addressing::*;
use crate::cpu::{Cpu, InterruptSource};
use crate::mem::CpuBus;
use std::marker::PhantomData;

pub struct Nop<A> {
	phantom: PhantomData<A>,
}

pub struct Brk<A> {
	phantom: PhantomData<A>,
}

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
		cpu.assert_interrupt(InterruptSource::BRK);
		cpu.interrupt(mem);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Brk<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}
