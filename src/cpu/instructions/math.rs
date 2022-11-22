use super::addressing::*;
use crate::cpu::Cpu;
use crate::mem::CpuBus;
use std::marker::PhantomData;

pub struct Adc<A>(PhantomData<A>);
pub struct Dec<A>(PhantomData<A>);
pub struct Dex<A>(PhantomData<A>);
pub struct Dey<A>(PhantomData<A>);
pub struct Inc<A>(PhantomData<A>);
pub struct Inx<A>(PhantomData<A>);
pub struct Iny<A>(PhantomData<A>);
pub struct Sbc<A>(PhantomData<A>);

// ADC

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Adc<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("ADC requires an address"));

		let res16 = (cpu.a as u16) + (op as u16) + (cpu.is_carry() as u16);
		cpu.set_overflow(cpu.a, op, res16 as u8); // set the bit before the actual operation
		cpu.a = res16 as u8;

		cpu.set_negative(cpu.a);
		cpu.set_zero(cpu.a);
		cpu.set_carry(res16 > 0xFF);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Adc<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Adc<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Adc<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Adc<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Adc<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Adc<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Adc<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Adc<IndirectY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			6
		} else {
			5
		}
	}
}

// DEC

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Dec<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let a = addr.expect("DEC requires an address");

		let mut op = CpuBus::read(mem, a);
		op = op.wrapping_sub(1);
		cpu.set_negative(op);
		cpu.set_zero(op);

		CpuBus::write(mem, a, op);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Dec<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Dec<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Dec<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Dec<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

// DEX

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Dex<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.x = cpu.x.wrapping_sub(1);

		cpu.set_negative(cpu.x);
		cpu.set_zero(cpu.x);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Dex<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// DEY

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Dey<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.y = cpu.y.wrapping_sub(1);

		cpu.set_negative(cpu.y);
		cpu.set_zero(cpu.y);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Dey<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// INC

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Inc<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let a = addr.expect("INC requires an address");

		let mut op = CpuBus::read(mem, a);
		op = op.wrapping_add(1);
		cpu.set_negative(op);
		cpu.set_zero(op);

		CpuBus::write(mem, a, op);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Inc<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Inc<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Inc<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Inc<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

// INX

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Inx<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.x = cpu.x.wrapping_add(1);
		cpu.set_negative(cpu.x);
		cpu.set_zero(cpu.x);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Inx<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// INY

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Iny<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.y = cpu.y.wrapping_add(1);

		cpu.set_negative(cpu.y);
		cpu.set_zero(cpu.y);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Iny<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// SBC

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Sbc<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("SBC requires an address"));

		let res16 = (cpu.a as i16) - (op as i16) - (!cpu.is_carry() as i16);
		let conv_op = ((op as i16) * -1) as u8;

		cpu.set_overflow(cpu.a, conv_op, res16 as u8); // set the bit before the actual operation
		cpu.a = res16 as u8;

		cpu.set_negative(cpu.a);
		cpu.set_zero(cpu.a);
		cpu.set_carry(res16 >= 0);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Sbc<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Sbc<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Sbc<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Sbc<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Sbc<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Sbc<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Sbc<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Sbc<IndirectY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			6
		} else {
			5
		}
	}
}
