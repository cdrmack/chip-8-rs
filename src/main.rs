use raylib::prelude::*;

mod chip8;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(chip8::WIDTH as i32 * 20, chip8::HEIGHT as i32 * 20)
        .title("CHIP-8")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
    }
}
