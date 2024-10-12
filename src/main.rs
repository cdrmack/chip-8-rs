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
    loop {
        chip.keypad = [false; 16];

        let key = rl_handle.get_key_pressed();
        if key.is_none() {
            return;
        }

        match key.unwrap() {
            KeyboardKey::KEY_ONE => chip.keypad[0] = true,
            KeyboardKey::KEY_TWO => chip.keypad[1] = true,
            KeyboardKey::KEY_THREE => chip.keypad[2] = true,
            KeyboardKey::KEY_FOUR => chip.keypad[3] = true,
            KeyboardKey::KEY_Q => chip.keypad[4] = true,
            KeyboardKey::KEY_W => chip.keypad[5] = true,
            KeyboardKey::KEY_E => chip.keypad[6] = true,
            KeyboardKey::KEY_R => chip.keypad[7] = true,
            KeyboardKey::KEY_A => chip.keypad[8] = true,
            KeyboardKey::KEY_S => chip.keypad[9] = true,
            KeyboardKey::KEY_D => chip.keypad[10] = true,
            KeyboardKey::KEY_F => chip.keypad[11] = true,
            KeyboardKey::KEY_Z => chip.keypad[12] = true,
            KeyboardKey::KEY_X => chip.keypad[13] = true,
            KeyboardKey::KEY_C => chip.keypad[14] = true,
            KeyboardKey::KEY_V => chip.keypad[15] = true,
            _ => (),
        }
    }
}
