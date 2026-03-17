use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::{Duration, Instant},
};

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCALE: usize = 10;

struct Chip8 {
    draw_flag: bool,
    gfx: [i8; 2048],
    key: [i8; 16],
    pc: u16,
    ir: i16,
    sp: u16,
    vr: [i8; 16],
    stack: [u16; 16],
    memory: [i8; 4096],
    delay_timer: u8,
    sound_timer: u8,
}

const FONTS: [u8; 80] = [
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

impl Chip8 {
    fn rand_byte(&self) -> i8 {
        let mut h = DefaultHasher::new();
        self.pc.hash(&mut h);
        Instant::now().hash(&mut h); // Instant implements Hash
        h.finish() as i8
    }
    fn new() -> Self {
        let mut chip = Chip8 {
            draw_flag: false,
            gfx: [0; 2048],
            key: [0; 16],
            pc: 0x200,
            ir: 0,
            sp: 0,
            vr: [0; 16],
            stack: [0; 16],
            memory: [0; 4096],
            delay_timer: 0,
            sound_timer: 0,
        };
        for (i, &byte) in FONTS.iter().enumerate() {
            chip.memory[i] = byte as i8;
        }
        chip
    }
    // Fixed
    fn load(&mut self, rom: Vec<u8>) {
        for addr in 0..rom.len() {
            self.memory[addr + 0x200] = rom[addr] as i8;
            // println!("setting rom '{:X}' to mem[{:X}]={:X}", rom[addr], addr+0x200, self.memory[addr + 0x200])
        }
    }

    fn execute(&mut self) {
        let opcode: u16 = ((self.memory[self.pc as usize] as u8 as u16) << 8)
            | (self.memory[self.pc as usize + 1] as u8 as u16);
        // println!("PC:{:X} OP:{:X}", self.pc, opcode);
        match opcode {
            0x00EE => {
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            0x0000..=0x0FFF => {
                // // execute subroutine
                // self.stack[self.sp as usize] = self.pc + 1;
                // self.pc = opcode & 0x0FFF;
                // unimplemented!("you should not be here :O");
                self.pc += 2;
            }
            0x1000..=0x1FFF => {
                // jump to address 0x1NNN
                self.pc = opcode & 0x0FFF;
            }
            0x2000..=0x2FFF => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc + 2;
                self.pc = opcode & 0x0FFF;
            }
            0x3000..=0x3FFF => {
                // Skip the following instruction
                // if the value of register VX equals NN
                if self.vr[((opcode & 0x0F00) >> 8) as usize] == (opcode & 0x00FF) as i8 {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x4000..=0x4FFF => {
                // Skip the following instruction
                // if the value of register VX does not equal NN
                if self.vr[((opcode & 0x0F00) >> 8) as usize] != (opcode & 0x00FF) as i8 {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x5000..=0x5FFF => {
                // Skip the following instruction if the value of
                // register VX is equal to the value of register VY
                let index1 = ((opcode & 0x0F00) >> 8) as usize;
                let index2 = ((opcode & 0x00F0) >> 4) as usize;
                if self.vr[index1] == self.vr[index2] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x6000..=0x6FFF => {
                // Skip the following instruction if the value of
                // register VX is equal to the value of register VY
                let index = ((opcode & 0x0F00) >> 8) as usize;
                let value = opcode & 0x00FF;
                self.vr[index] = value as i8;
                self.pc += 2;
            }
            0x7000..=0x7FFF => {
                // Skip the following instruction if the value of
                // register VX is equal to the value of register VY
                let index = ((opcode & 0x0F00) >> 8) as usize;
                let value = opcode & 0x00FF;
                self.vr[index] = self.vr[index].overflowing_add(value as i8).0;
                self.pc += 2;
            }
            0x8000..=0x8FFF => {
                // Skip the following instruction if the value of
                // register VX is equal to the value of register VY
                let indicator_nibble = opcode & 0x000F;
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                match indicator_nibble {
                    // Fixed
                    0 => {
                        self.vr[x] = self.vr[y];
                    }
                    1 => {
                        self.vr[x] |= self.vr[y];
                    }
                    2 => {
                        self.vr[x] &= self.vr[y];
                    }
                    3 => {
                        self.vr[x] ^= self.vr[y];
                    }
                    4 => {
                        let (sum, overflow) =
                            i8::overflowing_add(self.vr[y as usize], self.vr[x as usize]);
                        self.vr[x as usize] = sum;
                        self.vr[0xF] = if overflow { 1 } else { 0 };
                    }
                    5 => {
                        // VX = VX - VY
                        let (result, overflow) =
                            u8::overflowing_sub(self.vr[x] as u8, self.vr[y] as u8);
                        self.vr[x] = result as i8;
                        self.vr[0xF] = if overflow { 0 } else { 1 }; // VF=1 if NO borrow
                    }
                    6 => {
                        // VX >>= 1
                        self.vr[0xF] = self.vr[x] & 1;
                        self.vr[x] = ((self.vr[x] as u8) >> 1) as i8;
                    }
                    7 => {
                        // VX = VY - VX
                        let (result, overflow) =
                            u8::overflowing_sub(self.vr[y] as u8, self.vr[x] as u8);
                        self.vr[x] = result as i8;
                        self.vr[0xF] = if overflow { 0 } else { 1 };
                    }
                    0xE => {
                        self.vr[0xF] = ((self.vr[x] as u8) >> 7) as i8; // cast to u8 first
                        self.vr[x] = ((self.vr[x] as u8) << 1) as i8;
                    }

                    a => unimplemented!("cannot deal with 0x8{}{}{}", x, y, a),
                }
                self.pc += 2;
            }
            0x9000..=0x9FFF => {
                // Skip the following instruction if the value of
                // register VX is equal to the value of register VY
                let index1 = ((opcode & 0x0F00) >> 8) as usize;
                let index2 = ((opcode & 0x00F0) >> 4) as usize;
                if self.vr[index1] != self.vr[index2] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0xA000..=0xAFFF => {
                self.ir = (opcode & 0x0FFF) as i16;
                self.pc += 2;
            }
            0xB000..=0xBFFF => {
                let addr = opcode & 0x0FFF;
                self.pc = addr + self.vr[0] as u16;
            }
            0xC000..=0xCFFF => {
                let kk = (opcode & 0x00FF) as i8;
                let x = ((opcode & 0x0F00) >> 8) as usize;
                self.vr[x] = self.rand_byte() & kk;
                self.pc += 2;
            }
            0xD000..=0xDFFF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                let n = ((opcode & 0x000F) >> 0) as usize;
                self.vr[0xF] = 0;
                for row in 0..n {
                    let byte = self.memory[(self.ir + row as i16) as usize];
                    for col in 0..8 {
                        let last_bit = (byte >> (7 - col)) & 1;
                        let px = (self.vr[x] as usize + col) % 64;
                        let py = (self.vr[y] as usize + row) % 32;
                        if self.gfx[px + py * 64] != 0 && last_bit != 0 {
                            self.vr[0xF] = 1; // collision!
                        }
                        self.gfx[px + py * 64] ^= last_bit as i8;
                        self.draw_flag = true;
                    }
                }
                self.pc += 2;
            }
            0xE000..=0xEFFF if (opcode & 0x00FF) == 0x9E => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                if self.key[self.vr[n] as usize] > 0 {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0xE000..=0xEFFF if (opcode & 0x00FF) == 0xA1 => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                if self.key[self.vr[n] as usize] == 0 {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x07 => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                self.vr[n] = self.delay_timer as i8;
                self.pc += 2;
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x0A => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let mut pressed = false;
                for (i, &key) in self.key.iter().enumerate() {
                    if key > 0 {
                        self.vr[x] = i as i8;
                        pressed = true;
                        break;
                    }
                }
                if pressed {
                    self.pc += 2; // only advance if a key was pressed
                }
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x15 => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                self.delay_timer = self.vr[n] as u8;
                self.pc += 2;
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x18 => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                self.sound_timer = self.vr[n] as u8;
                self.pc += 2;
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x1E => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                self.ir += self.vr[n] as i16;
                self.pc += 2;
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x29 => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                let vx = self.vr[n] & 0x0F;
                self.ir = 5 * vx as i16;
                self.pc += 2;
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x33 => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                let vx = self.vr[n] as u8; // use u8 to avoid negative values

                self.memory[self.ir as usize] = (vx / 100) as i8; // hundreds
                self.memory[self.ir as usize + 1] = ((vx / 10) % 10) as i8; // tens
                self.memory[self.ir as usize + 2] = (vx % 10) as i8; // ones

                self.pc += 2;
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x55 => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                for i in 0..=n {
                    self.memory[self.ir as usize + i] = self.vr[i];
                }
                self.pc += 2;
            }
            0xF000..=0xFFFF if (opcode & 0x00FF) == 0x65 => {
                let n = ((opcode & 0x0F00) >> 8) as usize;
                for i in 0..=n {
                    self.vr[i] = self.memory[self.ir as usize + i];
                }
                self.pc += 2;
            }
            _ => {
                unimplemented!("opcode {} not implemented", opcode);
            }
        }
    }
}   

const KEY_MAP: [(minifb::Key, u32); 16] = [
    (Key::X, 0x0),
    (Key::Key1, 0x1),
    (Key::Key2, 0x2),
    (Key::Key3, 0x3),
    (Key::Q, 0x4),
    (Key::W, 0x5),
    (Key::E, 0x6),
    (Key::R, 0x7),
    (Key::A, 0x8),
    (Key::S, 0x9),
    (Key::D, 0xA),
    (Key::F, 0xB),
    (Key::Z, 0xC),
    (Key::Key4, 0xD),
    (Key::C, 0xE),
    (Key::V, 0xF),
];
fn main() {
    let rom_path = std::env::args().nth(1).expect("usage: chip8 <rom>");
    let program = std::fs::read(rom_path).expect("could not read rom");

    let mut chip = Chip8::new();
    chip.load(program);

    let mut window = Window::new(
        "CHIP-8",
        WIDTH * SCALE,
        HEIGHT * SCALE,
        WindowOptions::default(),
    )
    .unwrap();

    let mut buffer: Vec<u32> = vec![0; WIDTH * SCALE * HEIGHT * SCALE];
    let cycle_duration = Duration::from_millis(2);
    let timer_duration = Duration::from_millis(1);
    let mut last_timer = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let cycle_start = Instant::now();

        if last_timer.elapsed() >= timer_duration {
            if chip.delay_timer > 0 {
                chip.delay_timer -= 1;
            }
            if chip.sound_timer > 0 {
                chip.sound_timer -= 1;
            }
            last_timer = Instant::now();
        }

        let elapsed = cycle_start.elapsed();
        if elapsed < cycle_duration {
            std::thread::sleep(cycle_duration - elapsed);
        }

        chip.key = [0; 16]; // clear keys each frame

        for (minifb_key, chip8_key) in KEY_MAP {
            if window.is_key_down(minifb_key) {
                chip.key[chip8_key as usize] = 1;
            }
        }

        if chip.draw_flag {
            for (i, &pixel) in chip.gfx.iter().enumerate() {
                let px = i % WIDTH; // pixel x in chip8 space
                let py = i / WIDTH; // pixel y in chip8 space
                let color = if pixel > 0 { 0xFFFFFF } else { 0x000000 };

                for sy in 0..SCALE {
                    for sx in 0..SCALE {
                        let x = px * SCALE + sx;
                        let y = py * SCALE + sy;
                        buffer[y * WIDTH * SCALE + x] = color;
                    }
                }
            }
            chip.draw_flag = false;
        }

        chip.execute();

        window
            .update_with_buffer(&buffer, WIDTH * SCALE, HEIGHT * SCALE)
            .unwrap();
    }
}
