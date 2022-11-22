use super::addressing::*;
use crate::cpu::Cpu;
use crate::mem::CpuBus;
use std::marker::PhantomData;

pub struct Jmp<A> {
	phantom: PhantomData<A>,
}

pub struct Jsr<A> {
	phantom: PhantomData<A>,
}

pub struct Rti<A> {
	phantom: PhantomData<A>,
}

pub struct Rts<A> {
	phantom: PhantomData<A>,
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Jmp<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = addr.expect("JMP requires an address") as u16;
		cpu.pc = op;
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Jmp<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, Indirect> for Jmp<Indirect> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Jsr<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = addr.expect("JSR requires an address") as u16;

		// because pc was already incremented and the address
		// of the last JSR instruction byte must be pushed
		cpu.push16(mem, cpu.pc - 1);
		cpu.pc = op;

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Jsr<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Rti<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.p = cpu.pop8(mem);
		cpu.pc = cpu.pop16(mem);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Rti<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Rts<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.pc = cpu.pop16(mem) + 1; // op points to the last byte of the JSR instruction
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Rts<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}
