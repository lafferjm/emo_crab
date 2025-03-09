use std::fs::File;
use std::io::Read;

use macroquad::window::{Conf, next_frame};

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
        // stolen from varius chip 8 emulators
        let font: [u8; 80] = [
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

        let mut memory: [u8; 4096] = [0x0; 4096];
        for (i, value) in font.iter().enumerate() {
            memory[0x50 + i] = *value;
        }

        Self {
            registers: [0x0; 16],
            memory,
            index: 0x0,
            pc: 0x200,
            // Maybe I can replace stack and sp with a vector, future test
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

    pub fn get_instruction(&mut self) -> u16 {
        let instruction = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);

        self.pc = self.pc + 2;

        instruction
    }

    pub fn decode_instruction(&mut self, instruction: u16) {
        match (instruction & 0xF000) >> 12 {
            0x0 => self.video = [0x0; 64 * 32],
            0x1 => self.jump(instruction),
            0x6 => println!("Set Register: {:?}", instruction),
            0x7 => println!("Set index: {:?}", instruction),
            0xD => println!("Draw: {:?}", instruction),
            _ => eprintln!("Unknown instruction"),
        }
    }

    fn jump(&mut self, instruction: u16) {
        self.pc = instruction & 0x0FFF;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Emo Crab".to_string(),
        fullscreen: false,
        window_resizable: false,
        window_width: 64 * 16,
        window_height: 32 * 16,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut chip8 = Chip8::new();

    if let Err(e) = chip8.load_rom("./roms/ibm_logo.ch8") {
        eprintln!("Error loading rom: {}", e);
        std::process::exit(1);
    }

    loop {
        let instruction = chip8.get_instruction();
        chip8.decode_instruction(instruction);

        next_frame().await
    }
}
