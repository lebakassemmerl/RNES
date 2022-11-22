//! Object Attribute Memory (OAM)

use super::Segment;

const SIZE: usize = 256;

pub struct Oam {
    data: [u8; SIZE],
}

impl Oam {
    pub fn empty() -> Self {
        Self {
            data: [0; SIZE],
        }
    }
}

impl Segment for Oam {
    fn write(&mut self, addr: usize, val: u8) {
        self.data[addr & SIZE] = val;
    }

    fn read(&self, addr: usize) -> u8 {
        self.data[addr & SIZE]
    }
}