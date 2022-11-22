extern crate sdl2;

use std::sync::{
	mpsc::{Receiver, Sender},
	Arc, RwLock,
};
use std::thread::{self, JoinHandle};
use std::time::{self, Duration};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::EventPump;

use crate::io::JoyPad;
use crate::nes::Nes;

const SCREEN_SCALE: u32 = 3;
const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 240;
const SCREEN_WIDTH_SCALED: u32 = SCREEN_WIDTH * SCREEN_SCALE;
const SCREEN_HEIGHT_SCALED: u32 = SCREEN_HEIGHT * SCREEN_SCALE;

const TILES_SCALE: u32 = 2;
const TILES_WIDTH: u32 = 256;
const TILES_HEIGHT: u32 = 128;
const TILES_WIDTH_SCALED: u32 = TILES_WIDTH * TILES_SCALE;
const TILES_HEIGHT_SCALED: u32 = TILES_HEIGHT * TILES_SCALE;

const BORDER: i32 = 5;
const WINDOW_WIDTH: u32 = SCREEN_WIDTH_SCALED + TILES_WIDTH_SCALED + 3 * (BORDER as u32);
const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT_SCALED + 2 * (BORDER as u32);

const SCREEN: &ConstRect =
	&ConstRect::new(BORDER, BORDER, SCREEN_WIDTH_SCALED, SCREEN_HEIGHT_SCALED);
const TILES: &ConstRect = &ConstRect::new(
	2 * BORDER + (SCREEN_WIDTH_SCALED as i32),
	(WINDOW_HEIGHT as i32) - BORDER - (TILES_HEIGHT_SCALED as i32),
	TILES_WIDTH_SCALED,
	TILES_HEIGHT_SCALED,
);

struct ConstRect {
	off_x: i32,
	off_y: i32,
	width: u32,
	height: u32,
}

impl ConstRect {
	const fn new(off_x: i32, off_y: i32, width: u32, height: u32) -> Self {
		Self {
			off_x,
			off_y,
			width,
			height,
		}
	}
}

impl From<&ConstRect> for Rect {
	fn from(rect: &ConstRect) -> Self {
		Self::new(rect.off_x, rect.off_y, rect.width, rect.height)
	}
}

fn handle_events(
	ev_pump: &mut EventPump,
	tx_joy: &Sender<[JoyPad; 2]>,
	jp: &mut [JoyPad; 2],
) -> bool {
	let mut send_keys = false;

	let mut changed = |btn: &mut bool, state: bool| {
		send_keys = true;
		*btn = state;
	};

	for event in ev_pump.poll_iter() {
		match event {
			Event::Quit {
				..
			} => return true,
			Event::KeyDown {
				keycode: Some(Keycode::Escape) | Some(Keycode::Q),
				..
			} => return true,
			Event::KeyDown {
				keycode: Some(k),
				..
			} => match k {
				Keycode::W => changed(&mut jp[0].up, true),
				Keycode::A => changed(&mut jp[0].left, true),
				Keycode::S => changed(&mut jp[0].down, true),
				Keycode::D => changed(&mut jp[0].right, true),
				Keycode::J => changed(&mut jp[0].b, true),
				Keycode::K => changed(&mut jp[0].a, true),
				Keycode::Space => changed(&mut jp[0].start, true),
				Keycode::Return => changed(&mut jp[0].select, true),
				_ => {}
			},
			Event::KeyUp {
				keycode: Some(k),
				..
			} => match k {
				Keycode::W => changed(&mut jp[0].up, false),
				Keycode::A => changed(&mut jp[0].left, false),
				Keycode::S => changed(&mut jp[0].down, false),
				Keycode::D => changed(&mut jp[0].right, false),
				Keycode::J => changed(&mut jp[0].b, false),
				Keycode::K => changed(&mut jp[0].a, false),
				Keycode::Space => changed(&mut jp[0].start, false),
				Keycode::Return => changed(&mut jp[0].select, false),
				_ => {}
			},
			_ => {}
		}
	}

	if send_keys {
		tx_joy.send([jp[0].clone(), jp[1].clone()]).unwrap();
	}
	false
}

// keep this function for now since i invested 5h to get it correct..
/*
fn update_texture(
	text_buf: &mut [u8],
	src_buf: &Vec<u8>,
	scaling_factor: usize,
	text_width: usize,
) {
	let sf = scaling_factor;
	let sf_sq = sf * sf;
	let w = text_width;
	let width_raw = w * 3;
	let c1 = sf_sq * w;
	let c2 = sf * w;

	for (i, _) in src_buf.iter().enumerate().step_by(3) {
		let line = i / width_raw;
		let col = (i % width_raw) / 3;
		for y in 0..scaling_factor {
			for x in 0..scaling_factor {
				let idx = 3 * ((line * c1) + y * c2 + col * sf + x);
				text_buf[idx + 0] = src_buf[i + 0];
				text_buf[idx + 1] = src_buf[i + 1];
				text_buf[idx + 2] = src_buf[i + 2];
			}
		}
	}
}
*/

pub fn start(
	rx_quit: Receiver<bool>,
	rx_fb: Receiver<Arc<RwLock<Vec<u8>>>>,
	rx_tb: Receiver<Arc<RwLock<Vec<u8>>>>,
	tx_joystick: Sender<[JoyPad; 2]>,
) -> JoinHandle<()> {
	thread::spawn(move || {
		let mut jp: [JoyPad; 2] = [Default::default(), Default::default()];

		let ctx = sdl2::init().unwrap();
		let video_subsystem = ctx.video().unwrap();

		let window = video_subsystem
			.window("RNES", WINDOW_WIDTH, WINDOW_HEIGHT)
			.position_centered()
			.opengl()
			.build()
			.map_err(|e| e.to_string())
			.unwrap();

		let mut event_pump = ctx.event_pump().unwrap();
		let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();
		let texture_creator = canvas.texture_creator();

		let mut texture_nes = texture_creator
			.create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
			.map_err(|e| e.to_string())
			.unwrap();
		let mut texture_tiles = texture_creator
			.create_texture_streaming(PixelFormatEnum::RGB24, TILES_WIDTH, TILES_HEIGHT)
			.map_err(|e| e.to_string())
			.unwrap();

		canvas.set_draw_color(Color::RGB(220, 220, 255));
		canvas.clear();
		canvas.present();

		let mut time_last_frame = time::Instant::now();
		'running: loop {
			if let Ok(quit) = rx_quit.try_recv() {
				if quit {
					break 'running;
				}
			}

			if handle_events(&mut event_pump, &tx_joystick, &mut jp) {
				break 'running;
			}

			let now = time::Instant::now();
			if now.duration_since(time_last_frame) < Nes::FRAME_TIME_NS {
				std::thread::sleep(Duration::from_micros(100));
				continue;
			}
			time_last_frame = now;

			let now = time::Instant::now();

			// if we got a new framebuffer, we update the texture
			if let Ok(fb) = rx_fb.try_recv() {
				texture_nes
					.with_lock(None, |buf: &mut [u8], _pitch: usize| {
						let fb_vec = &*fb.read().unwrap();
						buf.copy_from_slice(fb_vec.as_slice());
					})
					.unwrap();
				canvas
					.copy_ex(&texture_nes, None, Rect::from(SCREEN), 0., None, false, false)
					.unwrap();
			}

			// if we got a new tilebuffer, we update the texture
			if let Ok(tb) = rx_tb.try_recv() {
				texture_tiles
					.with_lock(None, |buf: &mut [u8], _pitch: usize| {
						let tb_vec = &*tb.read().unwrap();
						buf.copy_from_slice(tb_vec.as_slice());
					})
					.unwrap();
				canvas
					.copy_ex(&texture_tiles, None, Rect::from(TILES), 0., None, false, false)
					.unwrap();
			}

			let elapsed = now.elapsed();
			//println!("rendering took {}us", elapsed.as_micros());

			canvas.present();
			// std::thread::sleep(Duration::new(0, 1_000_000));
		}
	})
}
