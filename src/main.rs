use raylib::prelude::*;

const WIDTH: i32 = 64;
const HEIGHT: i32 = 32;
const MULTIPLAYER: i32 = 10;

struct Chip8 {
    memory: [i8; 4096],
    data_registers: [i8; 16],
    index_register: i16,
    stack: [i16; 16],
    display: [bool; 2048],
    keyboard: [i8; 16],
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            data_registers: [0; 16],
            index_register: 0,
            stack: [0; 16],
            display: [false; 2048],
            keyboard: [0; 16],
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH * MULTIPLAYER, HEIGHT * MULTIPLAYER)
        .title("CHIP-8")
        .build();

    let _chip = Chip8::new();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
    }
}
