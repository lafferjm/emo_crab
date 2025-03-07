use std::fs::File;
use std::io::Read;

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

    pub fn load_rom(&mut self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;

        for (i, value) in contents.iter().enumerate() {
            self.memory[0x200 + i] = *value;
        }

        Ok(())
    }
}

fn main() {
    let mut chip8 = Chip8::new();

    if let Err(e) = chip8.load_rom("./roms/test_opcode.ch8") {
        eprintln!("Error loading rom: {}", e);
        std::process::exit(1);
    }
}
