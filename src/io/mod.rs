pub mod iocontrol;

#[derive(Default, Copy, Clone)]
pub struct JoyPad {
	pub up: bool,
	pub down: bool,
	pub left: bool,
	pub right: bool,
	pub start: bool,
	pub select: bool,
	pub a: bool,
	pub b: bool,
}
