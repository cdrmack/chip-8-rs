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
    sound_timer: u8,
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

        ram_with_fonts[0x50..=0x9F].copy_from_slice(&fontset);

        Chip8 {
            ram: ram_with_fonts,
            vram: [false; VRAM_SIZE],
            pc: 0x200,
            registers: [0; NUMBER_OF_REGISTERS],
            i: 0,
            stack: VecDeque::new(),
            keypad: [false; NUMBER_OF_KEYS], // 16 keys, 0..=F
            delay_timer: 0,
            sound_timer: 0,
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
        let nn = (opcode & 0x00FF) as u8;

        match nibbles {
            // clear screen
            (0x0, 0x0, 0xE, 0x0) => {
                self.vram = [false; VRAM_SIZE];
            }
            // return from a subroutine
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack.pop_back().unwrap();
            }
            // call machine code routine at NNN
            (0x0, _, _, _) => {
                // ignored by modern interpreters
            }
            // jump to address NNN
            (0x1, _, _, _) => {
                self.pc = nnn;
            }
            // call subroutine at NNN
            (0x2, _, _, _) => {
                self.stack.push_back(self.pc);
                self.pc = nnn;
            }
            // skip next instruction if VX equals NN
            (0x3, x, _, _) => {
                if self.registers[x as usize] == nn {
                    self.pc += 2;
                }
            }
            // skip next instruction if VX does not equal NN
            (0x4, x, _, _) => {
                if self.registers[x as usize] != nn {
                    self.pc += 2;
                }
            }
            // skip next instruction if VX equals VY
            (0x5, x, y, 0) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            (0x6, x, _, _) => {
                self.registers[x as usize] = nn;
            }
            // VX += nn
            (0x7, x, _, _) => {
                let vx = self.registers[x as usize] as u16;
                let val = nn as u16;
                let result = vx + val;
                self.registers[x as usize] = result as u8;
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
            // substract VY from VX
            // set VF to 0 if underflow, 1 otherwise
            (0x8, x, y, 5) => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.registers[x as usize] = vx.wrapping_sub(vy);
                self.registers[0xF] = if vx >= vy { 1 } else { 0 };
            }
            // set VF to the least-significant bit of VX
            // shift VX right by one bit
            (0x8, x, y, 6) => {
                let mut vx = self.registers[y as usize];
                self.registers[0xF] = vx & 1;
                vx = vx >> 1;
                self.registers[x as usize] = vx;
            }
            // substract VX from VY
            // set VF to 0 if underflow, 1 otherwise
            (0x8, x, y, 7) => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.registers[x as usize] = vy.wrapping_sub(vx);
                self.registers[0xF] = if vy >= vx { 1 } else { 0 };
            }
            // set VF to the least-significant bit of VX
            // shift VX left by one bit
            (0x8, x, y, 0xE) => {
                let mut vx = self.registers[y as usize];
                self.registers[0xF] = vx & 1;
                vx = vx << 1;
                self.registers[x as usize] = vx;
            }
            // skip next instruction if VX does not equal VY
            (0x9, x, y, 0) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 2;
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
                random_number &= nn;
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
            // skip next instruction if the key stored in VX is pressed
            (0xE, x, 9, 0xE) => {
                let vx = self.registers[x as usize];
                if self.keypad[vx as usize] {
                    self.pc += 2;
                }
            }
            // skip next instruction if the key stored in VX is not pressed
            (0xE, x, 0xA, 1) => {
                let vx = self.registers[x as usize];
                if !self.keypad[vx as usize] {
                    self.pc += 2;
                }
            }
            // set VX to the value of the delay timer
            (0xF, x, 0, 7) => {
                self.registers[x as usize] = self.delay_timer;
            }
            // wait for VX key to be pressed
            (0xF, x, 0, 0xA) => {
                let vx = self.registers[x as usize];
                if !self.keypad[vx as usize] {
                    self.pc -= 2; // decrement, we want to loop this until key is pressed
                }
            }
            // set delay timer to the value of VX
            (0xF, x, 1, 5) => {
                self.delay_timer = self.registers[x as usize];
            }
            // set sound timer to the value of VX
            (0xF, x, 1, 8) => {
                self.sound_timer = self.registers[x as usize];
            }
            // set I to VX + I
            (0xF, x, 1, 0xE) => {
                self.i += self.registers[x as usize] as u16;
            }
            // set I to the location of sprite for the character in VX
            (0xF, x, 2, 9) => {
                let sprite_location = 0x50 + (self.registers[x as usize] * 5); // sprites are stored in ram starting from 0x50, each sprite is 5 bytes
                self.i = sprite_location as u16;
            }
            // store binary-coded decimal of VX
            // hundreds digit in ram[I]
            // tens digit in ram[I+1]
            // ones digit in ram[I+2]
            (0xF, x, 3, 3) => {
                let vx = self.registers[x as usize];
                let ix = self.i as usize;
                self.ram[ix] = vx / 100;
                self.ram[ix + 1] = (vx % 100) / 10;
                self.ram[ix + 2] = vx % 10;
            }
            // store V0..=VX in memory starting at memory location I
            (0xF, x, 5, 5) => {
                for i in 0..=x {
                    self.ram[self.i as usize + i as usize] = self.registers[i as usize];
                }
            }
            // fill V0..=VX with values from memory starting at location I
            (0xF, x, 6, 5) => {
                for i in 0..=x {
                    self.registers[i as usize] = self.ram[self.i as usize + i as usize];
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
    fn test_default_values() {
        let chip = Chip8::new();
        assert_eq!(0x200, chip.pc);
        assert_eq!(0, chip.i);
    }

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

        assert_eq!(0x202, chip.pc);
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

        assert_eq!(0x202, chip.pc);
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

        assert_eq!(0x202, chip.pc);
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

        assert_eq!(0x202, chip.pc);
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
    fn test_8xy5_subtract_vy_from_vx_vx_is_bigger() {
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
    fn test_8xy5_subtract_vy_from_vx_s_smaller() {
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
    fn test_8xy5_subtract_vy_from_vx_vx_and_vy_are_equal() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 8;
        chip.registers[0xA] = 8;
        chip.registers[0xF] = 0;

        chip.decode(0x85A5);
        assert_eq!(0, chip.registers[0x5]);
        assert_eq!(8, chip.registers[0xA]);
        assert_eq!(1, chip.registers[0xF]);
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
    fn test_8xy7_subtract_vx_from_vy_vy_is_bigger() {
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
    fn test_8xy7_subtract_vx_from_vy_vy_is_smaller() {
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
    fn test_8xy7_subtract_vx_from_vy_vx_and_vy_are_equal() {
        let mut chip = Chip8::new();
        chip.registers[0x5] = 8;
        chip.registers[0xA] = 8;
        chip.registers[0xF] = 0;

        chip.decode(0x85A7);
        assert_eq!(0, chip.registers[0x5]);
        assert_eq!(8, chip.registers[0xA]);
        assert_eq!(1, chip.registers[0xF]);
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
        assert_eq!(8, chip.delay_timer);
    }
    #[test]
    fn test_fx18_set_sound_timer_to_vx() {
        let mut chip = Chip8::new();
        chip.registers[6] = 8;

        assert_eq!(0, chip.sound_timer);
        chip.decode(0xF618); // VX = 6
        assert_eq!(8, chip.sound_timer);
    }
    #[test]
    fn test_fx1e_add_vx_to_i() {
        let mut chip = Chip8::new();
        chip.registers[5] = 8;

        chip.decode(0xF51E);
        assert_eq!(8, chip.i);
    }
    #[test]
    fn test_fx55_store_registers_in_ram() {
        let mut chip = Chip8::new();
        chip.registers[0] = 8;
        chip.registers[1] = 6;
        chip.registers[2] = 5;
        chip.registers[3] = 7;
        chip.i = 0x4; // RAM start location

        assert_eq!(0, chip.ram[3]);
        assert_eq!(0, chip.ram[4]);
        assert_eq!(0, chip.ram[5]);
        assert_eq!(0, chip.ram[6]);
        assert_eq!(0, chip.ram[7]);
        assert_eq!(0, chip.ram[8]);
        chip.decode(0xF355); // VX = 3, store 0..=3
        assert_eq!(0, chip.ram[3]);
        assert_eq!(8, chip.ram[4]);
        assert_eq!(6, chip.ram[5]);
        assert_eq!(5, chip.ram[6]);
        assert_eq!(7, chip.ram[7]);
        assert_eq!(0, chip.ram[8]);
    }
    #[test]
    fn test_fx65_fill_registers_from_ram() {
        let mut chip = Chip8::new();
        chip.ram[3] = 0;
        chip.ram[4] = 8;
        chip.ram[5] = 6;
        chip.ram[6] = 5;
        chip.ram[7] = 7;
        chip.ram[8] = 0;
        chip.i = 0x4; // RAM start location

        assert_eq!(0, chip.registers[0]);
        assert_eq!(0, chip.registers[1]);
        assert_eq!(0, chip.registers[2]);
        assert_eq!(0, chip.registers[3]);
        assert_eq!(0, chip.registers[4]);
        chip.decode(0xF365); // VX = 3, fill 0..=3
        assert_eq!(8, chip.registers[0]);
        assert_eq!(6, chip.registers[1]);
        assert_eq!(5, chip.registers[2]);
        assert_eq!(7, chip.registers[3]);
        assert_eq!(0, chip.registers[4]);
    }
    #[test]
    fn test_fx29_set_i_to_location_of_sprite_in_vx() {
        let mut chip = Chip8::new();

        chip.registers[5] = 0x0; // character `0` starts at 0x50 (80)
        chip.decode(0xF529);
        assert_eq!(0x50, chip.i);

        chip.registers[6] = 0xA; // character `A` starts at 0x82 (130)
        chip.decode(0xF629);
        assert_eq!(0x82, chip.i);
    }
    #[test]
    fn test_fx33_binary_code_decimal_stored_in_vx() {
        let mut chip = Chip8::new();
        chip.registers[5] = 123;

        assert_eq!(0, chip.ram[chip.i as usize]);
        assert_eq!(0, chip.ram[chip.i as usize + 1]);
        assert_eq!(0, chip.ram[chip.i as usize + 2]);
        chip.decode(0xF533);
        assert_eq!(1, chip.ram[chip.i as usize]);
        assert_eq!(2, chip.ram[chip.i as usize + 1]);
        assert_eq!(3, chip.ram[chip.i as usize + 2]);
    }
    #[test]
    fn test_ex9e_skip_if_vx_button_is_pressed() {
        let mut chip = Chip8::new();
        chip.registers[5] = 0x2;

        let pc = chip.pc;
        chip.keypad[0x2] = false; // simulate that key is not pressed
        chip.decode(0xE59E);
        assert_eq!(pc, chip.pc); // pc is incremented in tick, not decode

        let pc = chip.pc;
        chip.keypad[0x2] = true; // simulate that key is pressed
        chip.decode(0xE59E);
        assert_eq!(pc + 2, chip.pc);
    }
    #[test]
    fn test_exa1_skip_if_vx_button_is_not_pressed() {
        let mut chip = Chip8::new();
        chip.registers[5] = 0x2;

        let pc = chip.pc;
        chip.keypad[0x2] = true; // simulate that key is pressed
        chip.decode(0xE5A1);
        assert_eq!(pc, chip.pc); // pc is incremented in tick, not decode

        let pc = chip.pc;
        chip.keypad[0x2] = false; // simulate that key is not pressed
        chip.decode(0xE5A1);
        assert_eq!(pc + 2, chip.pc);
    }
    #[test]
    fn test_fx0a_wait_for_key_to_be_pressed() {
        let mut chip = Chip8::new();
        chip.registers[5] = 0x2;
        chip.keypad[0x2] = false; // simulate that key is not pressed
        chip.ram[0x200] = 0xF5;
        chip.ram[0x200 + 1] = 0x0A;

        assert_eq!(0x200, chip.pc);
        chip.tick();
        assert_eq!(0x200, chip.pc);
        chip.tick();
        assert_eq!(0x200, chip.pc);
        chip.tick();
        assert_eq!(0x200, chip.pc);

        chip.keypad[0x2] = true; // simulate that key is pressed
        chip.tick();
        assert_eq!(0x200 + 2, chip.pc);
    }
}
