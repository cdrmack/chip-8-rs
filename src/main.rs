struct chip8 {
    memory: [i8; 4096],
    data_registers: [i8; 16],
    index_register: i16,
    stack: [i16; 128],
    display: [bool; 2048],
    keyboard: [i8; 16],
}

impl chip8 {
    fn tick(&mut self) {
        // TODO: fetch, decode, execute
    }
}

fn main() {
    println!("Hello, world!");
}
