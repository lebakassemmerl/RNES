use super::JoyPad;

const IDX_A: usize = 0;
const IDX_B: usize = 1;
const IDX_SELECT: usize = 2;
const IDX_START: usize = 3;
const IDX_UP: usize = 4;
const IDX_DOWN: usize = 5;
const IDX_LEFT: usize = 6;
const IDX_RIGHT: usize = 7;

struct Controller {
	btns_state: u8,
	btns_latched: u8,
	read_cnt: u8,

	connected: bool,
}

pub struct IOControl {
	ctrlr1: Controller,
	ctrlr2: Controller,
}

impl Controller {
	pub fn new(connected: bool) -> Self {
		Self {
			connected,
			btns_state: 0,
			btns_latched: 0,
			read_cnt: 0,
		}
	}

	pub fn read(&mut self) -> u8 {
		const PREV_BYTE_ON_BUS: u8 = 0x40;

		if !self.connected {
			PREV_BYTE_ON_BUS | 0x00
		} else if self.read_cnt < 8 {
			let ret = (self.btns_latched & 0x01) | PREV_BYTE_ON_BUS;
			self.read_cnt += 1;
			self.btns_latched >>= 1;
			ret
		} else {
			PREV_BYTE_ON_BUS | 0x01
		}
	}

	pub fn buttons_reload(&mut self) {
		self.btns_latched = self.btns_state;
		self.read_cnt = 0;
	}

	pub fn buttons_update(&mut self, jp: &JoyPad) {
		self.btns_state = (jp.a as u8) << IDX_A;
		self.btns_state |= (jp.b as u8) << IDX_B;
		self.btns_state |= (jp.start as u8) << IDX_START;
		self.btns_state |= (jp.select as u8) << IDX_SELECT;
		self.btns_state |= (jp.up as u8) << IDX_UP;
		self.btns_state |= (jp.down as u8) << IDX_DOWN;
		self.btns_state |= (jp.left as u8) << IDX_LEFT;
		self.btns_state |= (jp.right as u8) << IDX_RIGHT;
	}
}

impl IOControl {
	pub fn new(c1_connected: bool, c2_connected: bool) -> Self {
		Self {
			ctrlr1: Controller::new(c1_connected),
			ctrlr2: Controller::new(c2_connected),
		}
	}

	pub fn refresh_controller(&mut self, ctrl1: &JoyPad, ctrl2: &JoyPad) {
		self.ctrlr1.buttons_update(ctrl1);
		self.ctrlr2.buttons_update(ctrl2);
	}

	pub fn read_controller1(&mut self) -> u8 {
		self.ctrlr1.read()
	}

	pub fn read_controller2(&mut self) -> u8 {
		self.ctrlr2.read()
	}

	pub fn reload_controller(&mut self, val: u8) {
		if (val & 0x01) > 0 {
			self.ctrlr1.buttons_reload();
			self.ctrlr2.buttons_reload();
		}
	}
}
