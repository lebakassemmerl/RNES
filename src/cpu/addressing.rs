use crate::cpu::Cpu;
use crate::mem::CpuBus;

pub struct Accumulator;
pub struct Implied;
pub struct Immediate;
pub struct Relative;
pub struct Zeropage;
pub struct ZeropageX;
pub struct ZeropageY;
pub struct Absolute;
pub struct AbsoluteX;
pub struct AbsoluteY;
pub struct Indirect;
pub struct IndirectX;
pub struct IndirectY;

fn get_op16<B: CpuBus>(cpu: &Cpu<B>, mem: &mut B) -> u16 {
	let pc = cpu.pc + 2;
	((CpuBus::read(mem, (pc as usize) - 1) as u16) << 8)
		| (CpuBus::read(mem, (pc as usize) - 2) as u16)
}

fn get_op8<B: CpuBus>(cpu: &Cpu<B>, mem: &mut B) -> u8 {
	let pc = cpu.pc + 1;
	CpuBus::read(mem, (pc as usize) - 1)
}

pub trait AddressMode<B: CpuBus> {
	// return address and true if page-boundary was crossed, otherwise false
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool);

	// return the number of operand-bytes
	fn operand_bytes() -> u16;
}

impl<B: CpuBus> AddressMode<B> for Accumulator {
	fn get_address(_cpu: &mut Cpu<B>, _mem: &mut B) -> (Option<usize>, bool) {
		(None, false)
	}
	fn operand_bytes() -> u16 {
		0
	}
}

impl<B: CpuBus> AddressMode<B> for Implied {
	fn get_address(_cpu: &mut Cpu<B>, _mem: &mut B) -> (Option<usize>, bool) {
		(None, false)
	}

	fn operand_bytes() -> u16 {
		0
	}
}

impl<B: CpuBus> AddressMode<B> for Immediate {
	fn get_address(cpu: &mut Cpu<B>, _mem: &mut B) -> (Option<usize>, bool) {
		(Some(cpu.pc as usize), false)
	}

	fn operand_bytes() -> u16 {
		1
	}
}

impl<B: CpuBus> AddressMode<B> for Relative {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		(Some(get_op8(cpu, mem) as usize), false)
	}

	fn operand_bytes() -> u16 {
		1
	}
}

impl<B: CpuBus> AddressMode<B> for Zeropage {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		(Some(get_op8(cpu, mem) as usize), false)
	}

	fn operand_bytes() -> u16 {
		1
	}
}

impl<B: CpuBus> AddressMode<B> for ZeropageX {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		(Some(get_op8(cpu, mem).wrapping_add(cpu.x) as usize), false)
	}

	fn operand_bytes() -> u16 {
		1
	}
}

impl<B: CpuBus> AddressMode<B> for ZeropageY {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		(Some(get_op8(cpu, mem).wrapping_add(cpu.y) as usize), false)
	}

	fn operand_bytes() -> u16 {
		1
	}
}

impl<B: CpuBus> AddressMode<B> for Absolute {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		(Some(get_op16(cpu, mem) as usize), false)
	}

	fn operand_bytes() -> u16 {
		2
	}
}

impl<B: CpuBus> AddressMode<B> for AbsoluteX {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		let addr_base = get_op16(cpu, mem);
		let addr_real = addr_base.wrapping_add(cpu.x as u16);
		let boundary = if (addr_base >> 8) == (addr_real >> 8) {
			false
		} else {
			true
		};

		(Some(addr_real as usize), boundary)
	}

	fn operand_bytes() -> u16 {
		2
	}
}

impl<B: CpuBus> AddressMode<B> for AbsoluteY {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		let addr_base = get_op16(cpu, mem);
		let addr_real = addr_base.wrapping_add(cpu.y as u16);
		let boundary = if (addr_base >> 8) == (addr_real >> 8) {
			false
		} else {
			true
		};

		(Some(addr_real as usize), boundary)
	}

	fn operand_bytes() -> u16 {
		2
	}
}

impl<B: CpuBus> AddressMode<B> for Indirect {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		let addr_l = get_op16(cpu, mem) as usize;

		// if a page-overflow occurs, the address wraps back around to the SAME
		// page as the first address was, the next address is NOT located on the
		// next page, but always on the SAME!!!
		let addr_h = ((addr_l as u8).wrapping_add(1) as usize) | (addr_l & 0xFF00);
		let addr_real =
			((CpuBus::read(mem, addr_h) as u16) << 8) | (CpuBus::read(mem, addr_l) as u16);
		(Some(addr_real as usize), false)
	}

	fn operand_bytes() -> u16 {
		2
	}
}

impl<B: CpuBus> AddressMode<B> for IndirectX {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		let addr1 = get_op8(cpu, mem).wrapping_add(cpu.x);
		let addr2 = addr1.wrapping_add(1);
		let addr_real = ((CpuBus::read(mem, addr2 as usize) as u16) << 8)
			| (CpuBus::read(mem, addr1 as usize) as u16);
		(Some(addr_real as usize), false)
	}

	fn operand_bytes() -> u16 {
		1
	}
}

impl<B: CpuBus> AddressMode<B> for IndirectY {
	fn get_address(cpu: &mut Cpu<B>, mem: &mut B) -> (Option<usize>, bool) {
		let addr_start1 = get_op8(cpu, mem);
		let addr_start2 = addr_start1.wrapping_add(1);
		let addr_next = ((CpuBus::read(mem, addr_start2 as usize) as u16) << 8)
			| (CpuBus::read(mem, addr_start1 as usize) as u16);
		let addr_real = addr_next.wrapping_add(cpu.y as u16);
		let boundary = if (addr_next >> 8) == (addr_real >> 8) {
			false
		} else {
			true
		};

		(Some(addr_real as usize), boundary)
	}

	fn operand_bytes() -> u16 {
		1
	}
}

pub trait Operation<B: CpuBus> {
	fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize>;
}

pub trait AddressOperation<B: CpuBus, A: AddressMode<B>>: Operation<B> {
	fn cycles(extra_cycles: usize, boundary: bool) -> usize;

	fn do_instruction(cpu: &mut Cpu<B>, mem: &mut B) -> usize {
		let (addr, boundary) = A::get_address(cpu, mem);
		cpu.pc = cpu.pc.wrapping_add(A::operand_bytes()); // increment pc correctly

		// run the actual instruction
		if let Some(ex_c) = Self::exec(cpu, mem, addr) {
			Self::cycles(ex_c, boundary)
		} else {
			Self::cycles(0, boundary)
		}
	}
}
