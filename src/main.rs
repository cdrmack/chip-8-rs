use chip8::Chip8;
use clap::Parser;
use raylib::consts::KeyboardKey;
use raylib::prelude::*;
use std::{fs::File, io::Read};

#[derive(Parser, Debug)]
struct Args {
    /// ROM to load
    #[arg(short, required = true)]
    rom: String,
}

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
    let mut f = File::open(Args::parse().rom).expect("file not found");

    let mut chip = chip8::Chip8::new();
    let mut buffer = [0u8; 4096 - 0x200];
    let result = f.read(&mut buffer);
    if result.is_ok() {
        println!("read {} bytes from rom", result.unwrap());
        chip.load(&buffer);
    } else {
        panic!("reading rom returned error")
    }

    let (mut rl_handle, thread) = raylib::init()
        .size(
            (chip8::WIDTH * PIXEL_SIZE) as i32,
            (chip8::HEIGHT * PIXEL_SIZE) as i32,
        )
        .title("CHIP-8-rs")
        .build();

    while !rl_handle.window_should_close() {
        // input
        handle_input(&mut rl_handle, &mut chip);

        // tick
        chip.tick();

        // draw
        let mut draw_handle = rl_handle.begin_drawing(&thread);
        draw(&chip, &mut draw_handle);
    }
}

/*
 * 1 2 3 C -> 1 2 3 4
 * 4 5 6 D -> Q W E R
 * 7 8 9 E -> A S D E
 * A 0 B F -> Z X C V
 */
fn handle_input(rl_handle: &mut RaylibHandle, chip: &mut Chip8) {
    chip.keypad = [false; 16];

    chip.keypad[0x1] = rl_handle.is_key_down(KeyboardKey::KEY_ONE);

    chip.keypad[0x2] = rl_handle.is_key_down(KeyboardKey::KEY_TWO);

    chip.keypad[0x3] = rl_handle.is_key_down(KeyboardKey::KEY_THREE);

    chip.keypad[0xC] = rl_handle.is_key_down(KeyboardKey::KEY_FOUR);

    chip.keypad[0x4] = rl_handle.is_key_down(KeyboardKey::KEY_Q);

    chip.keypad[0x5] = rl_handle.is_key_down(KeyboardKey::KEY_W);

    chip.keypad[0x6] = rl_handle.is_key_down(KeyboardKey::KEY_E);

    chip.keypad[0xD] = rl_handle.is_key_down(KeyboardKey::KEY_R);

    chip.keypad[0x7] = rl_handle.is_key_down(KeyboardKey::KEY_A);

    chip.keypad[0x8] = rl_handle.is_key_down(KeyboardKey::KEY_S);

    chip.keypad[0x9] = rl_handle.is_key_down(KeyboardKey::KEY_D);

    chip.keypad[0xE] = rl_handle.is_key_down(KeyboardKey::KEY_F);

    chip.keypad[0xA] = rl_handle.is_key_down(KeyboardKey::KEY_Z);

    chip.keypad[0x0] = rl_handle.is_key_down(KeyboardKey::KEY_X);

    chip.keypad[0xB] = rl_handle.is_key_down(KeyboardKey::KEY_C);

    chip.keypad[0xF] = rl_handle.is_key_down(KeyboardKey::KEY_V);
}
