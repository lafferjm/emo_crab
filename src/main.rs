struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u8,
    pc: u16,
    stack: [u8; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; 64 * 32],
    opcode: u16,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            registers: [0x0; 16],
            memory: [0x0; 4096],
            index: 0x0,
            pc: 0x200,
            stack: [0x0; 16],
            sp: 0x0,
            delay_timer: 0x0,
            sound_timer: 0x0,
            keypad: [0x0; 16],
            video: [0x0; 64 * 32],
            opcode: 0x0,
        }
    }
}

fn main() {
    println!("Hello, world!");
    let chip8 = Chip8::new();
}
