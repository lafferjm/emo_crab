use std::fs::File;
use std::io::Read;

use macroquad::{color, shapes};
use macroquad::window::{Conf, next_frame, clear_background};

struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
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
            0x6 => self.set_register(instruction),
            0x7 => self.add_to_register(instruction),
            0xA => self.set_index_register(instruction),
            0xD => self.draw(instruction),
            _ => eprintln!("Unknown instruction: {:?}", instruction),
        }
    }

    pub fn render(&self) {
        for x in 0..64 {
            for y in 0..32 {
                if (self.video[y * 64 + x] == 1) {
                    shapes::draw_rectangle(x as f32 * 16., y as f32 * 16., 16., 16., color::WHITE);
                }
            }
        }
    }

    fn jump(&mut self, instruction: u16) {
        self.pc = instruction & 0x0FFF;
    }

    fn set_register(&mut self, instruction: u16) {
        let register = (instruction & 0x0F00) >> 8;
        let value = instruction & 0x00FF;

        self.registers[register as usize] = value as u8;
    }

    fn add_to_register(&mut self, instruction: u16) {
        let register = (instruction & 0x0F00) >> 8;
        let value = instruction & 0x00FF;

        self.registers[register as usize] = self.registers[register as usize] + value as u8;
    }

    fn set_index_register(&mut self, instruction: u16) {
        let value = instruction & 0x0FFF;
        self.index = value;
    }

    fn draw(&mut self, instruction: u16) {
        let x = (instruction & 0x0F00) >> 8;
        let x = self.registers[x as usize] % 64;

        let y = (instruction & 0x00F0) >> 4;
        let y = self.registers[y as usize] % 32;

        let height = instruction & 0x000F;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite = self.memory[(self.index + row) as usize];

            for column in 0..8 {
                // 0x80 == 1000_0000 we are checking each value by shifting each time
                let pixel = sprite & (0x80 >> column);

                if pixel != 0 {
                    let x_pos = (x + column) % 64;
                    let y_pos = (y + row as u8) % 32;
                    let index = y_pos as usize * 64 + x_pos as usize;

                    if self.video[index] == 1 {
                        self.registers[0xF] = 1;
                    }
                    self.video[index] ^= 1;
                }

            }
        }
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
        clear_background(color::BLACK);

        let instruction = chip8.get_instruction();
        chip8.decode_instruction(instruction);

        chip8.render();
        next_frame().await
    }
}
