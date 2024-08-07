use raylib::prelude::*;
use std::{fs::File, io::Read};

const PIXEL_SIZE: usize = 10;

mod chip8;

fn draw(chip: &chip8::Chip8, renderer: &mut RaylibDrawHandle) {
    for (i, val) in chip.get_vram().iter().enumerate() {
        let color = if *val { Color::WHITE } else { Color::BLACK };

        renderer.draw_rectangle(
            (i % chip8::WIDTH * PIXEL_SIZE) as i32,
            (i / chip8::WIDTH * PIXEL_SIZE) as i32,
            PIXEL_SIZE as i32,
            PIXEL_SIZE as i32,
            color,
        );
    }
}

fn main() {
    let (mut rl_handle, thread) = raylib::init()
        .size(
            (chip8::WIDTH * PIXEL_SIZE) as i32,
            (chip8::HEIGHT * PIXEL_SIZE) as i32,
        )
        .title("CHIP-8-rs")
        .build();

    let mut chip = chip8::Chip8::new();
    let mut f = File::open("rom.ch8").expect("file not found");
    let mut buffer = [0u8; 4096 - 0x200];
    let result = f.read(&mut buffer);
    if result.is_ok() {
        println!("read {} bytes from rom", result.unwrap());
        chip.load(&buffer);
    } else {
        panic!("reading rom returned error")
    }

    while !rl_handle.window_should_close() {
        let mut draw_handle = rl_handle.begin_drawing(&thread);
        chip.tick();
        draw(&chip, &mut draw_handle);
    }
}
