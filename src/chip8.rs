pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const VRAM_SIZE: usize = WIDTH * HEIGHT;
const NUMBER_OF_REGISTERS: usize = 16;

pub struct Chip8 {
    ram: [u8; RAM_SIZE],
    vram: [bool; VRAM_SIZE],
    pc: usize,
    registers: [u8; NUMBER_OF_REGISTERS], // V0, V1, ..., VF
    i: u16,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            ram: [0; RAM_SIZE],
            vram: [false; VRAM_SIZE],
            pc: 0x200,
            registers: [0; NUMBER_OF_REGISTERS],
            i: 0,
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

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => {
                self.vram = [false; VRAM_SIZE];
            }
            (0x1, _, _, _) => {
                self.pc = (opcode & 0x0FFF) as usize;
            }
            (0x6, _, _, _) => {
                let register_number = ((opcode & 0x0F00) >> 8) as u8;
                let value: u8 = (opcode & 0x00FF) as u8;
                self.registers[register_number as usize] = value;
            }
            (0x7, _, _, _) => {
                let register_number = ((opcode & 0x0F00) >> 8) as u8;
                let value: u8 = (opcode & 0x00FF) as u8;
                self.registers[register_number as usize] += value;
            }
            (0xA, _, _, _) => {
                self.i = opcode & 0x0FFF;
            }
            (0xD, _, _, _) => {
                // TODO: display/draw
            }
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();
        self.decode(opcode);
        // TODO: execute
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
}
