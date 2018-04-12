extern crate sdl2;
#[macro_use] extern crate failure;

use std::time::Duration;

use failure::err_msg;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub(crate) type Result<T> = std::result::Result<T, failure::Error>;

fn run() -> Result<()> {
    let ctx = sdl2::init().map_err(err_msg)?;
    let video = ctx.video().map_err(err_msg)?;

    let window = video.window("Fractal", 800, 600)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    canvas.set_draw_color(Color::RGB(60, 60, 60));
    canvas.clear();
    canvas.present();

    let mut event_pump = ctx.event_pump().map_err(err_msg)?;

    loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return Ok(());
                },
                _ => {},
            }
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn main() {
    match run() {
        Ok(_) => {},
        Err(e) => {
            println!("error running: {}", e);
        },
    }
}
