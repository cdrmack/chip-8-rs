use chip8::Chip8;
use raylib::consts::KeyboardKey;
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

    chip.keypad[0x1] = if rl_handle.is_key_down(KeyboardKey::KEY_ONE) {
        true
    } else {
        false
    };
    chip.keypad[0x2] = if rl_handle.is_key_down(KeyboardKey::KEY_TWO) {
        true
    } else {
        false
    };
    chip.keypad[0x3] = if rl_handle.is_key_down(KeyboardKey::KEY_THREE) {
        true
    } else {
        false
    };
    chip.keypad[0xC] = if rl_handle.is_key_down(KeyboardKey::KEY_FOUR) {
        true
    } else {
        false
    };
    chip.keypad[0x4] = if rl_handle.is_key_down(KeyboardKey::KEY_Q) {
        true
    } else {
        false
    };
    chip.keypad[0x5] = if rl_handle.is_key_down(KeyboardKey::KEY_W) {
        true
    } else {
        false
    };
    chip.keypad[0x6] = if rl_handle.is_key_down(KeyboardKey::KEY_E) {
        true
    } else {
        false
    };
    chip.keypad[0xD] = if rl_handle.is_key_down(KeyboardKey::KEY_R) {
        true
    } else {
        false
    };
    chip.keypad[0x7] = if rl_handle.is_key_down(KeyboardKey::KEY_A) {
        true
    } else {
        false
    };
    chip.keypad[0x8] = if rl_handle.is_key_down(KeyboardKey::KEY_S) {
        true
    } else {
        false
    };
    chip.keypad[0x9] = if rl_handle.is_key_down(KeyboardKey::KEY_D) {
        true
    } else {
        false
    };
    chip.keypad[0xE] = if rl_handle.is_key_down(KeyboardKey::KEY_F) {
        true
    } else {
        false
    };
    chip.keypad[0xA] = if rl_handle.is_key_down(KeyboardKey::KEY_Z) {
        true
    } else {
        false
    };
    chip.keypad[0x0] = if rl_handle.is_key_down(KeyboardKey::KEY_X) {
        true
    } else {
        false
    };
    chip.keypad[0xB] = if rl_handle.is_key_down(KeyboardKey::KEY_C) {
        true
    } else {
        false
    };
    chip.keypad[0xF] = if rl_handle.is_key_down(KeyboardKey::KEY_V) {
        true
    } else {
        false
    };
}
