use raylib::prelude::*;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const RAM: usize = 4096;
const VRAM: usize = WIDTH * HEIGHT;

struct Chip8 {
    ram: [u8; RAM],
    vram: [bool; VRAM],
    pc: usize,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            ram: [0; RAM],
            vram: [false; VRAM],
            pc: 0x200,
        }
    }

    fn fetch(&self) -> u16 {
        if self.pc < 0x200 {
            panic!("trying to access reserved memory");
        }

        let first: u16 = (self.ram[self.pc] as u16) << 8;
        let second: u16 = self.ram[self.pc + 1] as u16;
        first | second
    }

    fn tick(&mut self) {
        let _opcode = self.fetch();
        // TODO: decode
        // TODO: execute
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32 * 20, HEIGHT as i32 * 20)
        .title("CHIP-8")
        .build();

    let _chip = Chip8::new();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch() {
        let mut chip = Chip8::new();
        chip.ram[chip.pc] = 0xA2;
        chip.ram[chip.pc + 1] = 0xF0;

        assert_eq!(0xA2F0, chip.fetch());
    }

    #[test]
    #[should_panic]
    fn test_fetch_reserved() {
        let mut chip = Chip8::new();
        chip.pc = 0x1FF;
        chip.fetch();
    }
}
