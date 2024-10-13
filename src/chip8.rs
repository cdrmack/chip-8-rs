use rand::prelude::*;
use std::collections::VecDeque;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const VRAM_SIZE: usize = WIDTH * HEIGHT;
const NUMBER_OF_REGISTERS: usize = 16;
const NUMBER_OF_KEYS: usize = 16;

pub struct Chip8 {
    ram: [u8; RAM_SIZE],
    vram: [bool; VRAM_SIZE],
    pc: usize,
    registers: [u8; NUMBER_OF_REGISTERS],
    i: u16,
    stack: VecDeque<usize>,
    pub keypad: [bool; NUMBER_OF_KEYS],
    delay_timer: u8,
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
            keypad: [false; NUMBER_OF_KEYS],
            delay_timer: 0,
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
        let nn = (opcode & 0x00FF) as usize;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => {
                self.vram = [false; VRAM_SIZE];
            }
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack.pop_back().unwrap();
            }
            // jump to a machine code routine at NNN
            (0x0, _, _, _) => {
                // ignored by modern interpreters
            }
            (0x1, _, _, _) => {
                self.pc = nnn;
            }
            (0x2, _, _, _) => {
                self.stack.push_back(self.pc);
                self.pc = nnn;
            }
            (0x3, x, _, _) => {
                if self.registers[x as usize] == nn as u8 {
                    self.pc += 1;
                }
            }
            (0x4, x, _, _) => {
                if self.registers[x as usize] != nn as u8 {
                    self.pc += 1;
                }
            }
            (0x5, x, y, 0) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 1;
                }
            }
            (0x6, x, _, _) => {
                self.registers[x as usize] = nn as u8;
            }
            (0x7, x, _, _) => {
                self.registers[x as usize] += nn as u8;
            }
            (0x8, x, y, 0) => {
                self.registers[x as usize] = self.registers[y as usize];
            }
            (0x8, x, y, 1) => {
                self.registers[x as usize] |= self.registers[y as usize];
            }
            (0x8, x, y, 2) => {
                self.registers[x as usize] &= self.registers[y as usize];
            }
            (0x8, x, y, 3) => {
                self.registers[x as usize] ^= self.registers[y as usize];
            }
            (0x8, x, y, 4) => {
                let vx = self.registers[x as usize] as u16;
                let vy = self.registers[y as usize] as u16;
                let result = vx + vy;

                self.registers[x as usize] = result as u8;

                if result > 0xFF {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            (0x8, x, y, 5) => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.registers[0xF] = if vx > vy { 1 } else { 0 };
                self.registers[x as usize] = vx.wrapping_sub(vy);
            }
            // set VF to the least-significant bit of VX
            // shift VX right by one bit
            (0x8, x, y, 6) => {
                let mut vx = self.registers[y as usize];
                self.registers[0xF] = vx & 1;
                vx = vx >> 1;
                self.registers[x as usize] = vx;
            }
            (0x8, x, y, 7) => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.registers[0xF] = if vy > vx { 1 } else { 0 };
                self.registers[x as usize] = vy.wrapping_sub(vx);
            }
            // set VF to the least-significant bit of VX
            // shift VX left by one bit
            (0x8, x, y, 0xE) => {
                let mut vx = self.registers[y as usize];
                self.registers[0xF] = vx & 1;
                vx = vx << 1;
                self.registers[x as usize] = vx;
            }
            (0x9, x, y, 0) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 1;
                }
            }
            (0xA, _, _, _) => {
                self.i = nnn as u16;
            }
            // jump to address NNN + V0
            (0xB, _, _, _) => {
                self.pc = nnn + (self.registers[0] as usize);
            }
            // generate random number
            // binary AND with NN
            // store in VX
            (0xC, x, _, _) => {
                let mut rng = rand::thread_rng();
                let mut random_number: u8 = rng.gen();
                random_number &= nn as u8;
                self.registers[x as usize] = random_number;
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
            (0xE, _x, 9, 0xE) => {
                // TODO
            }
            (0xE, _x, 0xA, 1) => {
                // TODO
            }
            // set VX to the value of the delay timer
            (0xF, x, 0, 7) => {
                self.registers[x as usize] = self.delay_timer;
            }
            (0xF, _x, 0, 0xA) => {
                // TODO
            }
            // set delay timer to the value of VX
            (0xF, x, 1, 5) => {
                self.delay_timer = self.registers[x as usize];
            }
            (0xF, _x, 1, 8) => {
                // TODO
            }
            (0xF, _x, 1, 0xE) => {
                // TODO
            }
            (0xF, _x, 2, 9) => {
                // TODO
            }
            (0xF, _x, 3, 3) => {
                // TODO
            }
            (0xF, _x, 5, 5) => {
                // TODO
            }
            (0xF, _x, 6, 5) => {
                // TODO
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

    #[test]
    fn test_3xnn_should_skip_instruction() {
        let mut chip = Chip8::new();
        chip.registers[0xA] = 0x42;
        assert_eq!(0x200, chip.pc);

        chip.decode(0x3A42);

        assert_eq!(0x201, chip.pc);
    }

    #[test]
    fn test_3xnn_should_not_skip_instruction() {
        let mut chip = Chip8::new();
        chip.registers[0xA] = 0x41;
        assert_eq!(0x200, chip.pc);

        chip.decode(0x3A42);

        assert_eq!(0x200, chip.pc);
    }

    #[test]
    fn test_4xnn_should_skip_instruction() {
        let mut chip = Chip8::new();
        chip.registers[0xA] = 0x41;
        assert_eq!(0x200, chip.pc);

        chip.decode(0x4A42);

        assert_eq!(0x201, chip.pc);
    }

    #[test]
    fn test_4xnn_should_not_skip_instruction() {
        let mut chip = Chip8::new();
        chip.registers[0xA] = 0x42;
        assert_eq!(0x200, chip.pc);

        chip.decode(0x4A42);

        assert_eq!(0x200, chip.pc);
    }

    #[test]
    fn test_5xy0_should_skip_instruction() {
        let mut chip = Chip8::new();
        chip.registers[0xA] = 0x41;
        chip.registers[0xB] = 0x41;
        assert_eq!(0x200, chip.pc);

        chip.decode(0x5AB0);

        assert_eq!(0x201, chip.pc);
    }

    #[test]
    fn test_5xy0_should_not_skip_instruction() {
        let mut chip = Chip8::new();
        chip.registers[0xA] = 0x41;
        chip.registers[0xB] = 0x42;
        assert_eq!(0x200, chip.pc);

        chip.decode(0x5AB0);

        assert_eq!(0x200, chip.pc);
    }

    #[test]
    fn test_9xy0_should_skip_instruction() {
        let mut chip = Chip8::new();
        chip.registers[0xA] = 0x41;
        chip.registers[0xB] = 0x42;
        assert_eq!(0x200, chip.pc);

        chip.decode(0x9AB0);

        assert_eq!(0x201, chip.pc);
    }

    #[test]
    fn test_9xy0_should_not_skip_instruction() {
        let mut chip = Chip8::new();
        chip.registers[0xA] = 0x41;
        chip.registers[0xB] = 0x41;
        assert_eq!(0x200, chip.pc);

        chip.decode(0x9AB0);

        assert_eq!(0x200, chip.pc);
    }
    #[test]
    fn test_8xy0_should_copy_vy_to_vx() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 8;
        chip.registers[0xA] = 16;
        assert_eq!(8, chip.registers[0x5]);
        assert_eq!(16, chip.registers[0xA]);

        chip.decode(0x85A0);
        assert_eq!(16, chip.registers[0x5]);
        assert_eq!(16, chip.registers[0xA]);
    }
    #[test]
    fn test_8xy1_vx_or_vy() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 0b0001_1100;
        chip.registers[0xA] = 0b1011_0001;

        chip.decode(0x85A1);
        assert_eq!(0b1011_1101, chip.registers[0x5]);
        assert_eq!(0b1011_0001, chip.registers[0xA]);
    }
    #[test]
    fn test_8xy2_vx_and_vy() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 0b0001_1100;
        chip.registers[0xA] = 0b1011_0001;

        chip.decode(0x85A2);
        assert_eq!(0b0001_0000, chip.registers[0x5]);
        assert_eq!(0b1011_0001, chip.registers[0xA]);
    }
    #[test]
    fn test_8xy3_vx_xor_vy() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 0b0001_1100;
        chip.registers[0xA] = 0b1011_0001;

        chip.decode(0x85A3);
        assert_eq!(0b1010_1101, chip.registers[0x5]);
        assert_eq!(0b1011_0001, chip.registers[0xA]);
    }
    #[test]
    fn test_8xy4_add_vy_to_vx_no_carry() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 12;
        chip.registers[0xA] = 30;
        chip.registers[0xF] = 0;

        chip.decode(0x85A4);
        assert_eq!(42, chip.registers[0x5]);
        assert_eq!(30, chip.registers[0xA]);
        assert_eq!(0, chip.registers[0xF]);
    }
    #[test]
    fn test_8xy4_add_vy_to_vx_with_carry() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 8;
        chip.registers[0xA] = 0xFF;
        chip.registers[0xF] = 0;

        chip.decode(0x85A4);
        assert_eq!(7, chip.registers[0x5]);
        assert_eq!(0xFF, chip.registers[0xA]);
        assert_eq!(1, chip.registers[0xF]);
    }
    #[test]
    fn test_8xy5_subtract_vy_from_vx_with_carry() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 8;
        chip.registers[0xA] = 2;
        chip.registers[0xF] = 0;

        chip.decode(0x85A5);
        assert_eq!(6, chip.registers[0x5]);
        assert_eq!(2, chip.registers[0xA]);
        assert_eq!(1, chip.registers[0xF]);
    }
    #[test]
    fn test_8xy5_subtract_vy_from_vx_no_carry() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 2;
        chip.registers[0xA] = 8;
        chip.registers[0xF] = 0;

        chip.decode(0x85A5);
        assert_eq!(250, chip.registers[0x5]);
        assert_eq!(8, chip.registers[0xA]);
        assert_eq!(0, chip.registers[0xF]);
    }
    #[test]
    fn test_8xy6_store_vy_shifted_right_in_vx_lsb_1() {
        let mut chip = Chip8::new();
        chip.registers[0xF] = 0;
        chip.registers[0x5] = 0b1111_0000;
        chip.registers[0x6] = 0b0110_0001;

        chip.decode(0x8566);
        assert_eq!(1, chip.registers[0xF]);
        assert_eq!(0b0110_0001, chip.registers[0x6]);
        assert_eq!(0b0011_0000, chip.registers[0x5]);
    }
    #[test]
    fn test_8xy6_store_vy_shifted_right_in_vx_lsb_0() {
        let mut chip = Chip8::new();
        chip.registers[0xF] = 0;
        chip.registers[0x5] = 0b1111_0000;
        chip.registers[0x6] = 0b0110_0000;

        chip.decode(0x8566);
        assert_eq!(0, chip.registers[0xF]);
        assert_eq!(0b0110_0000, chip.registers[0x6]);
        assert_eq!(0b0011_0000, chip.registers[0x5]);
    }
    #[test]
    fn test_8xy7_subtract_vx_from_vy_with_carry() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 2;
        chip.registers[0xA] = 8;
        chip.registers[0xF] = 0;

        chip.decode(0x85A7);
        assert_eq!(6, chip.registers[0x5]);
        assert_eq!(8, chip.registers[0xA]);
        assert_eq!(1, chip.registers[0xF]);
    }
    #[test]
    fn test_8xy7_subtract_vx_from_vy_no_carry() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 8;
        chip.registers[0xA] = 2;
        chip.registers[0xF] = 0;

        chip.decode(0x85A7);
        assert_eq!(250, chip.registers[0x5]);
        assert_eq!(2, chip.registers[0xA]);
        assert_eq!(0, chip.registers[0xF]);
    }
    #[test]
    fn test_8xye_store_vy_shifted_left_in_vx_lsb_1() {
        let mut chip = Chip8::new();
        chip.registers[0xF] = 0;
        chip.registers[0x5] = 0b1111_0000;
        chip.registers[0x6] = 0b0110_0001;

        chip.decode(0x856E);
        assert_eq!(1, chip.registers[0xF]);
        assert_eq!(0b0110_0001, chip.registers[0x6]);
        assert_eq!(0b1100_0010, chip.registers[0x5]);
    }
    #[test]
    fn test_8xye_store_vy_shifted_left_in_vx_lsb_0() {
        let mut chip = Chip8::new();
        chip.registers[0xF] = 0;
        chip.registers[0x5] = 0b1111_0000;
        chip.registers[0x6] = 0b0110_0000;

        chip.decode(0x856E);
        assert_eq!(0, chip.registers[0xF]);
        assert_eq!(0b0110_0000, chip.registers[0x6]);
        assert_eq!(0b1100_0000, chip.registers[0x5]);
    }
    #[test]
    fn test_bnnn_jump_to_address_plus_v0() {
        let mut chip = Chip8::new();
        chip.registers[0x0] = 5;

        chip.decode(0xB123); // 0x123 = 291
        assert_eq!(296, chip.pc);
    }
    #[ignore]
    #[test]
    fn test_cxnn_binary_and_random_with_nn_store_in_vx() {
        // cannot test because of random number
        let mut chip = Chip8::new();
        chip.registers[0x0] = 5;

        chip.decode(0xC0FF);
        //assert_eq!(???, chip.registers[0]);
    }
    #[test]
    fn test_fx07_set_vx_to_delay_timer() {
        let mut chip = Chip8::new();
        chip.delay_timer = 8;

        chip.decode(0xF507); // VX = 5
        assert_eq!(8, chip.registers[5]);
    }
    #[test]
    fn test_fx15_set_delay_timer_to_vx() {
        let mut chip = Chip8::new();
        chip.registers[6] = 8;

        assert_eq!(0, chip.delay_timer);
        chip.decode(0xF615); // VX = 6
        assert_eq!(8, chip.registers[6]);
    }
}
