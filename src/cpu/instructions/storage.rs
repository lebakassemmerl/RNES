use super::addressing::*;
use crate::cpu::Cpu;
use crate::mem::CpuBus;
use std::marker::PhantomData;

macro_rules! load_instruction {
	($instr:ident, $register:ident, $name:expr) => {
		pub struct $instr<A> {
			phantom: PhantomData<A>,
		}

		impl<B: CpuBus, A: AddressMode<B>> Operation<B> for $instr<A> {
			fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
				let op = CpuBus::read(mem, addr.expect(concat!($name, " requires an address")));
				cpu.$register = op;

				cpu.set_negative(cpu.$register);
				cpu.set_zero(cpu.$register);

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

macro_rules! store_instruction {
	($instr:ident, $register:ident, $name:expr) => {
		pub struct $instr<A> {
			phantom: PhantomData<A>,
		}

		impl<B: CpuBus, A: AddressMode<B>> Operation<B> for $instr<A> {
			fn exec(cpu: &mut Cpu<B>, mem: &mut B, addr: Option<usize>) -> Option<usize> {
				let a = addr.expect(concat!($name, " requires an address"));
				CpuBus::write(mem, a, cpu.$register);

				None
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

macro_rules! transfer_instruction {
	($instr:ident, $src_reg:ident, $dest_reg:ident, $name:expr) => {
		pub struct $instr<A> {
			phantom: PhantomData<A>,
		}
		impl<B: CpuBus, A: AddressMode<B>> Operation<B> for $instr<A> {
			fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
				cpu.$dest_reg = cpu.$src_reg;

				cpu.set_negative(cpu.$dest_reg);
				cpu.set_zero(cpu.$dest_reg);

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

pub struct Txs<A> {
	phantom: PhantomData<A>,
}

impl<B: CpuBus, A: AddressMode<B>> Operation<B> for Txs<A> {
	fn exec(cpu: &mut Cpu<B>, _mem: &mut B, _addr: Option<usize>) -> Option<usize> {
		cpu.sp = cpu.x;
		None
	}
}

impl<B: CpuBus> AddressOperation<B, Implied> for Txs<Implied> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		2
	}
}

//* LOAD *

// LDA
load_instruction!(Lda, a, "LDA");
impl<B: CpuBus> AddressOperation<B, ZeropageX> for Lda<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Lda<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Lda<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Lda<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Lda<IndirectY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			6
		} else {
			5
		}
	}
}

// LDX
load_instruction!(Ldx, x, "LDX");
impl<B: CpuBus> AddressOperation<B, ZeropageY> for Ldx<ZeropageY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Ldx<AbsoluteY> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

// LDY
load_instruction!(Ldy, y, "LDY");
impl<B: CpuBus> AddressOperation<B, ZeropageX> for Ldy<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Ldy<AbsoluteX> {
	fn cycles(_extra_cycles: usize, boundary: bool) -> usize {
		if boundary {
			5
		} else {
			4
		}
	}
}

//* STORE *

// STA
store_instruction!(Sta, a, "STA");
impl<B: CpuBus> AddressOperation<B, ZeropageX> for Sta<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteX> for Sta<AbsoluteX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, AbsoluteY> for Sta<AbsoluteY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		5
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectX> for Sta<IndirectX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

impl<B: CpuBus> AddressOperation<B, IndirectY> for Sta<IndirectY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		6
	}
}

// STX
store_instruction!(Stx, x, "STX");
impl<B: CpuBus> AddressOperation<B, ZeropageY> for Stx<ZeropageY> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

// STY
store_instruction!(Sty, y, "STY");
impl<B: CpuBus> AddressOperation<B, ZeropageX> for Sty<ZeropageX> {
	fn cycles(_extra_cycles: usize, _boundary: bool) -> usize {
		4
	}
}

//* TRANSFER *
transfer_instruction!(Tax, a, x, "TAX");
transfer_instruction!(Tay, a, y, "TAY");
transfer_instruction!(Tsx, sp, x, "TSX");
transfer_instruction!(Txa, x, a, "TXA");
transfer_instruction!(Tya, y, a, "TYA");
