use super::addressing::*;
use crate::cpu::Cpu;
use crate::mem::CpuBus;
use std::marker::PhantomData;

pub struct Alr<A> {
	phantom: PhantomData<A>,
}

pub struct Anc<A> {
	phantom: PhantomData<A>,
}

pub struct Ane<A> {
	phantom: PhantomData<A>,
}

pub struct Arr<A> {
	phantom: PhantomData<A>,
}

pub struct Dcp<A> {
	phantom: PhantomData<A>,
}

pub struct Isc<A> {
	phantom: PhantomData<A>,
}

pub struct Las<A> {
	phantom: PhantomData<A>,
}

pub struct Lax<A> {
	phantom: PhantomData<A>,
}

pub struct Lxa<A> {
	phantom: PhantomData<A>,
}

pub struct Rla<A> {
	phantom: PhantomData<A>,
}

pub struct Rra<A> {
	phantom: PhantomData<A>,
}

pub struct Sax<A> {
	phantom: PhantomData<A>,
}

pub struct Sbx<A> {
	phantom: PhantomData<A>,
}

pub struct Sha<A> {
	phantom: PhantomData<A>,
}

pub struct Shx<A> {
	phantom: PhantomData<A>,
}

pub struct Shy<A> {
	phantom: PhantomData<A>,
}

pub struct Slo<A> {
	phantom: PhantomData<A>,
}

pub struct Sre<A> {
	phantom: PhantomData<A>,
}

pub struct Tas<A> {
	phantom: PhantomData<A>,
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Alr<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("ALR requires an address"));
		cpu.a &= op;
		cpu.set_carry(cpu.a & 0x01 > 0);
		cpu.a >>= 1;

		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Alr<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Anc<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("ANC requires an address"));
		cpu.a &= op;

		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);
		cpu.set_carry(cpu.is_negative());

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Anc<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Ane<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		// This operation is highly unstable, the constant below depends on temperature, series and
		// maybe other factors of the chip. We set it to 0xFF here.
		// See https://www.masswerk.at/6502/6502_instruction_set.html#LAS
		const CONSTANT: u8 = 0xFF;

		let op = CpuBus::read(mem, addr.expect("ANE requires an address"));

		cpu.a = (cpu.a | CONSTANT) & cpu.x & op;

		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Ane<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Arr<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("ARR requires an address"));

		cpu.a &= op;
		cpu.a = ((cpu.a & 0x01) << 7) | (cpu.a >> 1);

		let bit5_6 = (cpu.a >> 5) & 0x03;
		match bit5_6 {
			0b00 => {
				cpu.set_carry(false);
				cpu.set_overflow_raw(false);
			}
			0b01 => {
				cpu.set_carry(false);
				cpu.set_overflow_raw(true);
			}
			0b10 => {
				cpu.set_carry(true);
				cpu.set_overflow_raw(true);
			}
			0b11 => {
				cpu.set_carry(true);
				cpu.set_overflow_raw(false);
			}
			_ => panic!("ARR: variable bit5_6 has an invalid value: {}", bit5_6),
		}

		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Arr<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Dcp<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("DCP requires an address");
		let mut op = CpuBus::read(mem, address);

		op = op.wrapping_sub(1);
		CpuBus::write(mem, address, op);

		let res = cpu.a.wrapping_sub(op);
		cpu.set_carry(cpu.a >= op);
		cpu.set_negative(res);
		cpu.set_zero(res);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Dcp<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Dcp<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Dcp<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Dcp<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Dcp<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Dcp<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Dcp<IndirectY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Isc<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("ISC requires an address");
		let mut op = CpuBus::read(mem, address);

		op = op.wrapping_add(1);
		CpuBus::write(mem, address, op);

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

impl<B: CpuBus> AddressOperation<B, Zeropage> for Isc<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Isc<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Isc<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Isc<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Isc<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Isc<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Isc<IndirectY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Las<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let res = cpu.sp & CpuBus::read(mem, addr.expect("LAS requires an address"));

		cpu.a = res;
		cpu.x = res;
		cpu.sp = res;

		cpu.set_zero(res);
		cpu.set_negative(res);

		None
	}
}
impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Las<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Lax<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("LAX requires an address"));

		cpu.a = op;
		cpu.x = op;

		cpu.set_zero(op);
		cpu.set_negative(op);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Lax<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageY> for Lax<ZeropageY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Lax<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Lax<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Lax<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Lax<IndirectY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			6
		} else {
			5
		}
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Lxa<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		// This operation is highly unstable, the constant below depends on temperature, series and
		// maybe other factors of the chip. We set it to 0xFF here.
		// See https://www.masswerk.at/6502/6502_instruction_set.html#LXA
		const CONSTANT: u8 = 0xFF;

		let mut op = CpuBus::read(mem, addr.expect("LXA requires an address"));
		op = (cpu.a | CONSTANT) & op;

		cpu.a = op;
		cpu.x = op;

		cpu.set_zero(op);
		cpu.set_negative(op);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Lxa<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Rla<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("RLA requires an address");
		let mut op = CpuBus::read(mem, address);
		let carry = (op & 0x80) > 0;

		op <<= 1;
		op |= cpu.is_carry() as u8;
		cpu.a &= op;
		CpuBus::write(mem, address, op);

		cpu.set_carry(carry);
		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Rla<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Rla<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Rla<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Rla<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Rla<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Rla<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Rla<IndirectY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Rra<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("RRA requires an address");
		let mut m = CpuBus::read(mem, address);

		let carry = m & 0x01;
		m >>= 1;
		m |= (cpu.is_carry() as u8) << 7;
		CpuBus::write(mem, address, m);

		let res16 = (cpu.a as u16) + (m as u16) + (carry as u16);
		cpu.set_overflow(cpu.a, m, res16 as u8); // set the bit before the actual operation
		cpu.a = res16 as u8;

		cpu.set_negative(cpu.a);
		cpu.set_zero(cpu.a);
		cpu.set_carry(res16 > 0xFF);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Rra<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Rra<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Rra<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Rra<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Rra<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Rra<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Rra<IndirectY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Sax<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		CpuBus::write(mem, addr.expect("SAX requires an address"), cpu.a & cpu.x);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Sax<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		3
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageY> for Sax<ZeropageY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Sax<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Sax<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Sbx<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let op = CpuBus::read(mem, addr.expect("SBX requires an address"));

		let res = (cpu.a & cpu.x).wrapping_sub(op);

		cpu.set_carry(res > cpu.x);
		cpu.set_zero(res);
		cpu.set_negative(res);
		cpu.x = res;

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Immediate> for Sbx<Immediate> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Sha<A> {
	// unstable

	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("SHA requires an address");
		let h = (((address + 1) >> 8) & 0xFF) as u8;

		CpuBus::write(mem, address, cpu.a & cpu.x & h);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Sha<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Sha<IndirectY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Shx<A> {
	// unstable

	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("SHX requires an address");
		let h = (((address + 1) >> 8) & 0xFF) as u8;

		CpuBus::write(mem, address, cpu.x & h);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Shx<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Shy<A> {
	// unstable

	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("SHY requires an address");
		let h = (((address + 1) >> 8) & 0xFF) as u8;

		CpuBus::write(mem, address, cpu.y & h);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Shy<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Slo<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("SLO requires an address");

		let mut m = CpuBus::read(mem, address);
		cpu.set_carry((m & 0x80) > 0);
		m <<= 1;
		cpu.a |= m;
		CpuBus::write(mem, address, m);

		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Slo<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Slo<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Slo<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Slo<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Slo<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Slo<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Slo<IndirectY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Sre<A> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("SRE requires an address");

		let mut m = CpuBus::read(mem, address);
		cpu.set_carry((m & 0x01) > 0);
		m >>= 1;
		cpu.a ^= m;
		CpuBus::write(mem, address, m);

		cpu.set_zero(cpu.a);
		cpu.set_negative(cpu.a);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, Zeropage> for Sre<Zeropage> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, ZeropageX> for Sre<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, Absolute> for Sre<Absolute> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Sre<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Sre<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		7
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Sre<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Sre<IndirectY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		8
	}
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Tas<A> {
	// unstable

	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
		let address = addr.expect("TAS requires an address");
		let h = (((address + 1) >> 8) & 0xFF) as u8;

		cpu.sp = cpu.a & cpu.x;
		CpuBus::write(mem, address, cpu.a & cpu.x & h);

		None
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Tas<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}
