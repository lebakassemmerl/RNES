#![allow(dead_code)]

mod cartridge;
mod cpu;
mod io;
mod mem;
mod nes;
mod ppu;
mod sdl2_wrapper;
mod util;

use std::env;
use std::sync::{
	mpsc::{self, Receiver, Sender},
	Arc, RwLock,
};
use std::thread;
use std::time::Instant;

use io::JoyPad;
use nes::Nes;
use sdl2_wrapper::engine;

type ShFb = Arc<RwLock<Vec<u8>>>;

fn main() {
	let (tx_quit, rx_quit): (Sender<bool>, Receiver<bool>) = mpsc::channel();
	let (tx_fb, rx_fb): (Sender<ShFb>, Receiver<ShFb>) = mpsc::channel();
	let (tx_tb, rx_tb): (Sender<ShFb>, Receiver<ShFb>) = mpsc::channel();
	let (tx_joy, rx_joy): (Sender<[JoyPad; 2]>, Receiver<[JoyPad; 2]>) = mpsc::channel();

	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		panic!("Please pass the path to the desired ROM!");
	}

	let mut nes = nes::Nes::new(args[1].as_str()).unwrap();

	let thr = engine::start(rx_quit, rx_fb, rx_tb, tx_joy);
	tx_tb.send(nes.tile_buf()).unwrap();

	println!("Start");
	nes.start();
	loop {
		if let Ok(b) = rx_joy.try_recv() {
			nes.button_update(b);
		}

		let frame_start = Instant::now();
		if nes.run_frame() {
			// the PPU finished rendering the framebuffer -> render it via SDL2
			tx_fb.send(nes.get_fb()).unwrap_or(());
			tx_tb.send(nes.tile_buf()).unwrap_or(());
		} else {
			// vertical blank of the PPU finished -> 1 frame finished -> wait until 1/60Hz elapse
			let frame_time = Instant::now().duration_since(frame_start);
			if frame_time < Nes::FRAME_TIME_NS {
				thread::sleep(Nes::FRAME_TIME_NS - frame_time);
			}
		}

		if thr.is_finished() {
			break;
		}
	}

	println!("Save the game!");
	nes.save();

	println!("Quit");
	tx_quit.send(false).unwrap_or(());
	return;
}
