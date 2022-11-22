use super::addressing::*;
use crate::cpu::Cpu;
use crate::mem::CpuBus;
use std::marker::PhantomData;

pub struct And<A> {
	phantom: PhantomData<A>,
}

// create own instruction for left-shifting the accumulator
pub struct AslA<A> {
	phantom: PhantomData<A>,
}

pub struct Asl<A> {
	phantom: PhantomData<A>,
}

pub struct Bit<A> {
	phantom: PhantomData<A>,
}

pub struct Eor<A> {
	phantom: PhantomData<A>,
}

// create own instruction for right-shifting the accumulator
pub struct LsrA<A> {
	phantom: PhantomData<A>,
}

pub struct Lsr<A> {
	phantom: PhantomData<A>,
}

pub struct Ora<A> {
	phantom: PhantomData<A>,
}

// create own instruction for left-rotating the accumulator
pub struct RolA<A> {
	phantom: PhantomData<A>,
}

pub struct Rol<A> {
	phantom: PhantomData<A>,
}

// create own instruction for right-rotating the accumulator
pub struct RorA<A> {
	phantom: PhantomData<A>,
}

pub struct Ror<A> {
	phantom: PhantomData<A>,
}

// AND

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for And<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("AND requires address"));
		cpu.a &= op;
		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for And<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for And<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for And<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for And<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for And<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for And<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for And<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for And<IndirectY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			6
		} else {
			5
		}
	}
}

// ASL accumulator

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for AslA<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.set_carry(cpu.a & 0x80 > 0);
		cpu.a <<= 1;
		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Accumulator> for AslA<Accumulator> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// ASL

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Asl<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let a = addr.expect("ASL pre_exec() requires an address");

		let mut op = CpuBus::read(mem, a);
		cpu.set_carry(op & 0x80 > 0);
		op <<= 1;
		cpu.set_zero(op);
		cpu.set_negative(op);

		CpuBus::write(mem, a, op);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Asl<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Asl<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Asl<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Asl<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

// BIT

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Bit<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("BIT pre_exec() requires an address"));
		cpu.set_statusbit(op & (1 << 7) > 0, 7); // copy negative bit into statusregister
		cpu.set_statusbit(op & (1 << 6) > 0, 6); // copy overflow bit into statusregister
		cpu.set_zero(op & cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Bit<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Bit<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

// EOR

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Eor<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("EOR pre_exec() requires an address"));
		cpu.a ^= op;
		cpu.set_negative(cpu.a);
		cpu.set_zero(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Eor<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Eor<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Eor<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Eor<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Eor<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Eor<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Eor<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Eor<IndirectY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			6
		} else {
			5
		}
	}
}

// LSR accumulator

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for LsrA<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.set_carry(cpu.a & 0x01 > 0);
		cpu.a >>= 1;
		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Accumulator> for LsrA<Accumulator> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// LSR

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Lsr<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let a = addr.expect("LSR pre_exec() requires an address");

		let mut op = CpuBus::read(mem, a);
		cpu.set_carry(op & 0x01 > 0);
		op >>= 1;
		cpu.set_zero(op);
		cpu.set_negative(op);

		CpuBus::write(mem, a, op);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Lsr<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Lsr<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Lsr<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Lsr<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

// ORA

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Ora<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("ORA requires address"));
		cpu.a |= op;
		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Ora<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Ora<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Ora<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Ora<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Ora<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Ora<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Ora<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Ora<IndirectY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			6
		} else {
			5
		}
	}
}

// ROL accumulator

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for RolA<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		let carry = cpu.is_carry() as u8;

		cpu.set_carry(cpu.a & 0x80 > 0);
		cpu.a <<= 1;
		cpu.a |= carry;
		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Accumulator> for RolA<Accumulator> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// ROL

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Rol<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let a = addr.expect("ROL pre_exec() requires an address");

		let mut op = CpuBus::read(mem, a);
		let carry = cpu.is_carry() as u8;
		cpu.set_carry(op & 0x80 > 0);
		op <<= 1;
		op |= carry;
		cpu.set_zero(op);
		cpu.set_negative(op);

		CpuBus::write(mem, a, op);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Rol<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Rol<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Rol<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Rol<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

// ROR accumulator

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for RorA<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		let carry = cpu.is_carry() as u8;

		cpu.set_carry(cpu.a & 0x01 > 0);
		cpu.a >>= 1;
		cpu.a |= carry << 7;
		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Accumulator> for RorA<Accumulator> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

// ROL

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Ror<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let a = addr.expect("ROR pre_exec() requires an address");

		let mut op = CpuBus::read(mem, a);
		let carry = cpu.is_carry() as u8;
		cpu.set_carry(op & 0x01 > 0);
		op >>= 1;
		op |= carry << 7;
		cpu.set_zero(op);
		cpu.set_negative(op);

		CpuBus::write(mem, a, op);
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Ror<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Ror<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Ror<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Ror<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}
