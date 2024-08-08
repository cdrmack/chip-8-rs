use std::collections::VecDeque;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const VRAM_SIZE: usize = WIDTH * HEIGHT;
const NUMBER_OF_REGISTERS: usize = 16;

pub struct Chip8 {
    ram: [u8; RAM_SIZE],
    vram: [bool; VRAM_SIZE],
    pc: usize,
    registers: [u8; NUMBER_OF_REGISTERS],
    i: u16,
    stack: VecDeque<usize>,
}

impl Chip8 {
    pub fn new() -> Self {
        let fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        let mut ram_with_fonts = [0; RAM_SIZE];

        ram_with_fonts[0x050..=0x09F].copy_from_slice(&fontset);

        Chip8 {
            ram: ram_with_fonts,
            vram: [false; VRAM_SIZE],
            pc: 0x200,
            registers: [0; NUMBER_OF_REGISTERS],
            i: 0,
            stack: VecDeque::new(),
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = 0x200 + i;
            if addr < RAM_SIZE {
                self.ram[addr] = byte;
            } else {
                panic!("not enough RAM")
            }
        }
    }

    pub fn get_vram(&self) -> &[bool; VRAM_SIZE] {
        &self.vram
    }

    fn fetch(&self) -> u16 {
        if self.pc < 0x200 {
            panic!("trying to access reserved memory");
        }

        let first: u16 = (self.ram[self.pc] as u16) << 8;
        let second: u16 = self.ram[self.pc + 1] as u16;
        first | second
    }

    fn decode(&mut self, opcode: u16) {
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = (opcode & 0x0FFF) as usize;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => {
                self.vram = [false; VRAM_SIZE];
            }
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack.pop_back().unwrap();
            }
            (0x1, _, _, _) => {
                self.pc = nnn;
            }
            (0x2, _, _, _) => {
                self.stack.push_back(self.pc);
                self.pc = nnn;
            }
            (0x6, x, _, _) => {
                let value: u8 = (opcode & 0x00FF) as u8;
                self.registers[x as usize] = value;
            }
            (0x7, x, _, _) => {
                let value: u8 = (opcode & 0x00FF) as u8;
                self.registers[x as usize] += value;
            }
            (0xA, _, _, _) => {
                self.i = opcode & 0x0FFF;
            }
            (0xD, x, y, n) => {
                let vram_x = self.registers[x as usize] as usize % WIDTH;
                let vram_y = self.registers[y as usize] as usize % HEIGHT;
                self.registers[0xF] = 0;

                let mut clip_x = 0;
                let mut clip_y = 0;

                if WIDTH - vram_x < 8 {
                    clip_x = 8 - (WIDTH - vram_x);
                }

                if HEIGHT - 1 - vram_y < (n as usize) {
                    clip_y = n as usize - (HEIGHT - 1 - vram_y);
                }

                for row in 0..(n - clip_y as u8) {
                    let sprite_data = self.ram[(self.i + row as u16) as usize];
                    for column in 0..(8 - clip_x) {
                        let location = vram_x + column + ((vram_y + row as usize) * WIDTH);
                        let sprite_pixel_on = (sprite_data & (0x80 >> column)) != 0;
                        if sprite_pixel_on {
                            if self.vram[location] {
                                self.vram[location] = false;
                                self.registers[0xF] = 1;
                            } else {
                                self.vram[location] = true;
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();
        // TODO: remove this guard
        if self.pc < RAM_SIZE - 2 {
            self.pc += 2;
        }
        self.decode(opcode);
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

    #[test]
    fn test_00e0_should_clear_screen() {
        let mut chip = Chip8::new();

        for e in chip.vram.iter_mut() {
            *e = true;
        }

        chip.decode(0x00E0);

        let iter = chip.vram.iter().filter(|x| **x == true);
        assert_eq!(0, iter.count());
    }

    #[test]
    fn test_1nnn_should_jump() {
        let mut chip = Chip8::new();
        assert_eq!(0x200, chip.pc);

        chip.decode(0x142C);
        assert_eq!(0x42C as usize, chip.pc);
    }

    #[test]
    fn test_6xnn_should_store_number_in_register_x() {
        let mut chip = Chip8::new();
        let register_number = 8;
        assert_eq!(0, chip.registers[register_number]);

        chip.decode(0x6842);
        assert_eq!(0x42, chip.registers[register_number]);
    }

    #[test]
    fn test_7xnn_should_add_value_to_register_x() {
        let mut chip = Chip8::new();
        let register_number = 8;
        chip.decode(0x6842);
        assert_eq!(0x42, chip.registers[register_number]);

        chip.decode(0x7808);
        assert_eq!(0x42 + 0x08, chip.registers[register_number]);
    }

    #[test]
    fn test_annn_store_address_in_register() {
        let mut chip = Chip8::new();
        assert_eq!(0, chip.i);

        chip.decode(0xA123);
        assert_eq!(0x123, chip.i);
    }

    #[test]
    fn test_dxyn_updates_vram() {
        let mut chip = Chip8::new();
        let x = 0;
        let y = 1;
        chip.registers[x] = 8;
        chip.registers[y] = 16;

        chip.i = 0x200;
        chip.ram[0x200] = 0b1001_1001;
        chip.ram[0x201] = 0b0110_0110;

        let start_position = chip.registers[x] as usize + (chip.registers[y] as usize * WIDTH);

        // first row
        assert_eq!([false; 8], chip.vram[start_position..start_position + 8]);
        // second row
        assert_eq!(
            [false; 8],
            chip.vram[start_position + WIDTH..start_position + WIDTH + 8]
        );

        chip.decode(0xD012);

        // first row
        assert_eq!(
            [true, false, false, true, true, false, false, true],
            chip.vram[start_position..start_position + 8]
        );
        // second row
        assert_eq!(
            [false, true, true, false, false, true, true, false],
            chip.vram[start_position + WIDTH..start_position + WIDTH + 8]
        );

        assert_eq!(0, chip.registers[0xF]);
    }

    #[test]
    fn test_dxyn_updates_vram_and_vf_register() {
        let mut chip = Chip8::new();
        chip.registers[0] = 0;

        chip.i = 0x200;
        chip.ram[0x200] = 0b1000_0000;
        chip.vram[0] = true;

        assert_eq!(
            [true, false, false, false, false, false, false, false],
            chip.vram[0..8]
        );
        assert_eq!(0, chip.registers[0xF]);

        chip.decode(0xD001);

        assert_eq!([false; 8], chip.vram[0..8]);
        assert_eq!(1, chip.registers[0xF]);
    }

    #[test]
    fn test_dxyn_updates_vram_and_clip_on_x() {
        let mut chip = Chip8::new();
        let position_x: usize = 62;
        chip.registers[0] = position_x as u8; // should update vram[62..=63] only
        chip.registers[1] = 0;

        chip.i = 0x200;
        chip.ram[0x200] = 0xFF;

        assert_eq!(
            [false, false, false, false, false, false, false, false],
            chip.vram[position_x..position_x + 8],
        );

        chip.decode(0xD011);
        assert_eq!(
            [true, true, false, false, false, false, false, false], // 62, 63, EDGE (clip here)
            chip.vram[position_x..position_x + 8],
        );
    }

    #[test]
    fn test_dxyn_updates_vram_and_clip_on_y() {
        let mut chip = Chip8::new();
        let position_y: usize = 30;
        chip.registers[0] = 0;
        chip.registers[1] = position_y as u8;

        chip.i = 0x200;
        chip.ram[0x200] = 0xF0;
        chip.ram[0x201] = 0xF0;
        chip.ram[0x202] = 0xF0;
        chip.ram[0x203] = 0xF0;

        // row 30
        assert_eq!(
            [false, false, false, false, false, false, false, false],
            chip.vram[position_y * WIDTH..position_y * WIDTH + 8],
        );

        // row 31
        assert_eq!(
            [false, false, false, false, false, false, false, false],
            chip.vram[position_y * WIDTH..position_y * WIDTH + 8],
        );

        chip.decode(0xD014);

        // row 30
        assert_eq!(
            [true, true, true, true, false, false, false, false],
            chip.vram[position_y * WIDTH..position_y * WIDTH + 8],
        );

        // row 31
        assert_eq!(
            [true, true, true, true, false, false, false, false],
            chip.vram[position_y * WIDTH..position_y * WIDTH + 8],
        );
    }

    #[test]
    fn test_2nnn_should_update_pc_and_stack() {
        let mut chip = Chip8::new();
        assert_eq!(0x200, chip.pc);
        assert!(chip.stack.is_empty());

        chip.decode(0x2123);

        assert_eq!(Some(&0x200), chip.stack.back());
        assert_eq!(0x123, chip.pc);
    }

    #[test]
    fn test_00ee_should_pop_stack_and_update_pc() {
        let mut chip = Chip8::new();
        assert_eq!(0x200, chip.pc);
        assert!(chip.stack.is_empty());
        chip.stack.push_back(0x123);

        chip.decode(0x00EE);

        assert!(chip.stack.is_empty());
        assert_eq!(0x123, chip.pc);
    }
}
