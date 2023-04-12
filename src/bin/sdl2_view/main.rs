extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::Duration;

fn main() {
    println!("bin\\sdl2_view(gui view): This is a shell U.I. via GUI based (mainly SDL2)");
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();

    let window = video_subsystem
        .window("bin\\sdl2_view(gui view) test", 1024, 768)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/sara-cal.png").unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Set the position and size of the sprite you want to display from the sprite sheet
        let src_rect = Rect::new((i % 3) * 32, (i / 3) * 32, 32, 32);
        // Set the position and size of where you want to display the sprite on the screen
        let dest_rect = Rect::new(100, 100, 32, 32);

        i = (i + 1) % 6;

        canvas.clear();
        canvas.copy(&texture, src_rect, dest_rect).unwrap();
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
    }
}
