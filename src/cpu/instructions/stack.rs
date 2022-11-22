use super::addressing::*;
use crate::cpu::Cpu;
use crate::mem::CpuBus;
use std::marker::PhantomData;

pub struct Plp<A> {
	phantom: PhantomData<A>,
}

pub struct Php<A> {
	phantom: PhantomData<A>,
}

pub struct Pla<A> {
	phantom: PhantomData<A>,
}

pub struct Pha<A> {
	phantom: PhantomData<A>,
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Pla<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.a = cpu.pop8(mem);
		cpu.set_negative(cpu.a);
		cpu.set_zero(cpu.a);

		None
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Plp<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.p = cpu.pop8(mem);
		cpu.set_break(false);
		None
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Pha<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.push8(mem, cpu.a);
		None
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Php<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.push8(mem, cpu.p);
		cpu.set_unused(true);

		// According to the spec this should be done but in the logfile of the test-rom it's not
		// done so lets also don't do it.
		// cpu.set_break(true);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Pha<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Php<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Plp<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Pla<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}
