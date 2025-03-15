use std::env;
use std::fs::File;
use std::io::Read;

use macroquad::window::{Conf, clear_background, next_frame};
use macroquad::{color, shapes};

struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u8; 64 * 32],
}

impl Chip8 {
    pub fn new() -> Self {
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
        for (i, &value) in font.iter().enumerate() {
            memory[0x50 + i] = value;
        }

        Self {
            registers: [0x0; 16],
            memory,
            index: 0x0,
            pc: 0x200,
            stack: [0x0; 16],
            sp: 0x0,
            delay_timer: 0x0,
            sound_timer: 0x0,
            keypad: [0x0; 16],
            video: [0x0; 64 * 32],
        }
    }

    pub fn load_rom(&mut self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;

        for (i, &value) in contents.iter().enumerate() {
            self.memory[0x200 + i] = value;
        }

        Ok(())
    }

    pub fn get_instruction(&mut self) -> u16 {
        let instruction =
            (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16;
        self.pc += 2;
        instruction
    }

    pub fn decode_instruction(&mut self, instruction: u16) {
        match (instruction & 0xF000) >> 12 {
            0x0 => {
                if instruction & 0x00FF == 0xE0 {
                    self.video = [0x0; 64 * 32];
                }
                if instruction & 0x00FF == 0xEE {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
            }
            0x1 => self.jump(instruction),
            0x2 => self.call_subroutine(instruction),
            0x3 => self.skip_if_equal(instruction),
            0x4 => self.skip_if_not_equal(instruction),
            0x5 => self.skip_if_registers_equal(instruction),
            0x6 => self.set_register(instruction),
            0x7 => self.add_to_register(instruction),
            0x8 => match instruction & 0x000F {
                0x0 => self.set_x_register(instruction),
                0x1 => self.binary_or(instruction),
                0x2 => self.binary_and(instruction),
                0x3 => self.logical_xor(instruction),
                _ => eprintln!("Unknown instruction: {:?}", instruction),
            },
            0x9 => self.skip_if_registers_not_equal(instruction),
            0xA => self.set_index_register(instruction),
            0xD => self.draw(instruction),
            _ => eprintln!("Unknown instruction: {:?}", instruction),
        }
    }

    pub fn render(&self) {
        for x in 0..64 {
            for y in 0..32 {
                if self.video[y * 64 + x] == 1 {
                    shapes::draw_rectangle(x as f32 * 16., y as f32 * 16., 16., 16., color::WHITE);
                }
            }
        }
    }

    fn jump(&mut self, instruction: u16) {
        self.pc = instruction & 0x0FFF;
    }

    fn set_register(&mut self, instruction: u16) {
        let register = ((instruction & 0x0F00) >> 8) as usize;
        let value = (instruction & 0x00FF) as u8;
        self.registers[register] = value;
    }

    fn add_to_register(&mut self, instruction: u16) {
        let register = ((instruction & 0x0F00) >> 8) as usize;
        let value = (instruction & 0x00FF) as u8;
        self.registers[register] = self.registers[register].wrapping_add(value);
    }

    fn set_index_register(&mut self, instruction: u16) {
        self.index = instruction & 0x0FFF;
    }

    fn draw(&mut self, instruction: u16) {
        let x = ((instruction & 0x0F00) >> 8) as usize;
        let x = self.registers[x] % 64;

        let y = ((instruction & 0x00F0) >> 4) as usize;
        let y = self.registers[y] % 32;

        let height = (instruction & 0x000F) as u16;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite = self.memory[(self.index + row) as usize];
            for column in 0..8 {
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

    fn call_subroutine(&mut self, instruction: u16) {
        let location = instruction & 0x0FFF;
        self.stack[self.sp as usize] = self.pc;
        self.sp = self.sp.wrapping_add(1);
        self.pc = location;
    }

    fn skip_if_equal(&mut self, instruction: u16) {
        let register = ((instruction & 0x0F00) >> 8) as usize;
        let value = (instruction & 0x00FF) as u8;
        if self.registers[register] == value {
            self.pc += 2;
        }
    }

    fn skip_if_not_equal(&mut self, instruction: u16) {
        let register = ((instruction & 0x0F00) >> 8) as usize;
        let value = (instruction & 0x00FF) as u8;
        if self.registers[register] != value {
            self.pc += 2;
        }
    }

    fn skip_if_registers_equal(&mut self, instruction: u16) {
        let x_register = ((instruction & 0x0F00) >> 8) as usize;
        let y_register = ((instruction & 0x00F0) >> 4) as usize;
        if self.registers[x_register] == self.registers[y_register] {
            self.pc += 2;
        }
    }

    fn skip_if_registers_not_equal(&mut self, instruction: u16) {
        let x_register = ((instruction & 0x0F00) >> 8) as usize;
        let y_register = ((instruction & 0x00F0) >> 4) as usize;
        if self.registers[x_register] != self.registers[y_register] {
            self.pc += 2;
        }
    }

    fn set_x_register(&mut self, instruction: u16) {
        let x_register = ((instruction & 0x0F00) >> 8) as usize;
        let y_register = ((instruction & 0x00F0) >> 4) as usize;
        self.registers[x_register] = self.registers[y_register];
    }

    fn binary_or(&mut self, instruction: u16) {
        let x_register = ((instruction & 0x0F00) >> 8) as usize;
        let y_register = ((instruction & 0x00F0) >> 4) as usize;
        self.registers[x_register] |= self.registers[y_register];
    }

    fn binary_and(&mut self, instruction: u16) {
        let x_register = ((instruction & 0x0F00) >> 8) as usize;
        let y_register = ((instruction & 0x00F0) >> 4) as usize;
        self.registers[x_register] &= self.registers[y_register];
    }

    fn logical_xor(&mut self, instruction: u16) {
        let x_register = ((instruction & 0x0F00) >> 8) as usize;
        let y_register = ((instruction & 0x00F0) >> 4) as usize;
        self.registers[x_register] ^= self.registers[y_register];
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
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Usage: emo_crab <rom>");
        std::process::exit(1);
    }

    let mut chip8 = Chip8::new();

    if let Err(e) = chip8.load_rom(&args[1]) {
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
