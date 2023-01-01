use rand::random;

const FONT_SET: &[u8] = &[
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

#[derive(Debug)]
pub struct Emulator {
    memory: [u8; 4096],     // Memory
    v: [u8; 16],            // Registers (v0 -> vf)
    st: u8,                 // Sound timer
    dt: u8,                 // Delay timer
    sp: usize,              // Stack pointer
    pc: usize,              // Program counter
    i: u16,                 // i register
    stack: [u16; 16],       // Stack
    key: [bool; 16],        // key[x] = True if a key is pressed
    screen: [[u8; 64]; 32], // Screen (64 x 32)
    draw: bool,             // True if the emulator should draw the screen in the next iteration
}

impl Emulator {
    pub fn new() -> Self {
        let mut e = Emulator {
            memory: [0; 4096],
            v: [0; 16],
            st: 0,
            dt: 0,
            sp: 0,
            pc: 0x200,
            i: 0,
            stack: [0; 16],
            key: [false; 16],
            screen: [[0; 64]; 32],
            draw: false,
        };

        for (i, val) in FONT_SET.iter().enumerate() {
            e.memory[i] = *val;
        }

        return e;
    }

    /// Reset all the registers and re-run the currently loaded program in the memory.
    pub fn reset(&mut self) {
        self.v = [0; 16];
        self.st = 0;
        self.dt = 0;
        self.sp = 0;
        self.pc = 0x200;
        self.i = 0;
        self.stack = [0; 16];
        self.key = [false; 16];
        self.screen = [[0; 64]; 32];
        self.draw = false;
    }

    pub fn set_key_state(&mut self, index: usize, down: bool) {
        if index > self.key.len() {
            panic!("Invalid key index specified: {}", index);
        }
        self.key[index] = down;
    }

    pub fn cycle(&mut self) -> u16 {
        // Reconstruct the opcode which is splitted in 2 bytes in the memory.
        let left = self.memory[self.pc] as u16;
        let right = self.memory[(self.pc + 1)] as u16;
        let op = left << 8 | right;

        self.process_opcode(op);

        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
            // TODO: beep
        }

        return op;
    }

    pub fn load_program(&mut self, program: &Vec<u8>) {
        if program.len() > (self.memory.len() - 0x200) {
            panic!("not enough memory");
        }

        for (i, val) in program.iter().enumerate() {
            self.memory[i + 0x200] = *val;
        }
    }

    pub fn get_screen(&self) -> &[[u8; 64]; 32] {
        &self.screen
    }

    pub fn should_draw(&mut self) -> bool {
        let sd = self.draw;
        self.draw = false;
        return sd;
    }

    fn process_opcode(&mut self, op: u16) {
        let x = ((op & 0x0F00) >> 8) as usize;
        let y = ((op & 0x00F0) >> 4) as usize;
        let nnn = op & 0x0FFF;
        let kk = (op & 0x00FF) as u8;

        // Increment the PC register for each cycle
        if self.pc + 2 < 0xFFF {
            self.next_instruction()
        }

        match op & 0xF000 {
            0x0000 => match op & 0x00FF {
                // Clear display
                0x00E0 => self.screen = [[0; 64]; 32],
                // Return from a subroutine
                0x00EE => {
                    self.pc = self.stack[self.sp] as usize;
                    self.sp -= 1;
                }
                // Ignore the 0nnn opcodes
                _ => return,
            },
            0x1000 => {
                self.pc = nnn as usize;
            }
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp] = self.pc as u16;
                self.pc = nnn as usize;
            }
            0x3000 => {
                if self.v[x] == kk {
                    self.next_instruction();
                }
            }
            0x4000 => {
                if self.v[x] != kk {
                    self.next_instruction();
                }
            }
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.next_instruction();
                }
            }
            0x6000 => {
                self.v[x] = kk;
            }
            0x7000 => {
                self.v[x] = self.v[x].checked_add(kk).unwrap_or(255);
            }
            0x8000 => match op & 0x000F {
                0x0000 => {
                    self.v[x] = self.v[y];
                }
                0x0001 => {
                    self.v[x] = self.v[x] | self.v[y];
                }
                0x0002 => {
                    self.v[x] = self.v[x] & self.v[y];
                }
                0x0003 => {
                    self.v[x] = self.v[x] ^ self.v[y];
                }
                0x0004 => {
                    let val = (self.v[x] as u16) + (self.v[y] as u16);
                    if val > 255 {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.v[x] = val as u8;
                }
                0x0005 => {
                    if self.v[x] > self.v[y] {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.v[x] = self.v[x].checked_sub(self.v[y]).unwrap_or(0);
                }
                0x0006 => {
                    let vx = self.v[x];
                    if vx & 0x0F == 1 {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.v[x] /= 2;
                }
                0x0007 => {
                    let vx = self.v[x];
                    let vy = self.v[y];

                    if vy > vx {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.v[x] = vy.checked_sub(vx).unwrap_or(0);
                }
                0x000E => {
                    let vx = self.v[x];

                    if vx & 0xF0 == 1 {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.v[x] *= 2;
                }
                _ => self.handle_invalid_op(op),
            },
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.next_instruction();
                }
            }
            0xA000 => {
                self.i = nnn;
            }
            0xB000 => {
                self.pc = (nnn + (self.v[0x0] as u16)) as usize;
            }
            0xC000 => {
                let rand_val: u8 = random();

                self.v[x] = rand_val & kk;
            }
            0xD000 => {
                let vx = self.v[x] as u16;
                let vy = self.v[y] as u16;
                let n = op & 0x000F;
                self.v[0xF] = 0;
                for j in 0..n {
                    let pixel = self.memory[(self.i + j) as usize];
                    for i in 0..8 {
                        if (pixel & (0x80 >> i)) != 0 {
                            let mut pos_x = vx + i;
                            let mut pos_y = vy + j;

                            if pos_x >= 64 {
                                pos_x = 0;
                            }
                            if pos_y >= 32 {
                                pos_y = 0;
                            }

                            if self.screen[pos_y as usize][pos_x as usize] == 1 {
                                self.v[0xF] = 1;
                            }

                            self.screen[pos_y as usize][pos_x as usize] ^= 1;
                        }
                    }
                }
                self.draw = true;
            }
            0xE000 => match op & 0x00FF {
                0x009E => {
                    let vx = self.v[x];

                    if self.key[vx as usize] {
                        self.next_instruction();
                    }
                }
                0x00A1 => {
                    let vx = self.v[x];

                    if !self.key[vx as usize] {
                        self.next_instruction();
                    }
                }
                _ => self.handle_invalid_op(op),
            },
            0xF000 => match op & 0x00FF {
                0x0007 => {
                    self.v[x] = self.dt;
                }
                0x000A => {
                    let mut pressed = false;
                    while !pressed {
                        for (i, key) in self.key.into_iter().enumerate() {
                            if key {
                                self.v[x] = i as u8;
                                pressed = true;
                            }
                        }
                    }
                }
                0x0015 => {
                    self.dt = self.v[x];
                }
                0x0018 => {
                    self.st = self.v[x];
                }
                0x001E => {
                    self.i = self.i + (self.v[x] as u16);
                }
                0x0029 => {
                    self.i = (self.v[x] as u16) * 0x5; // 0x5 is the length of one character
                }
                0x0033 => {
                    self.memory[self.i as usize] = self.v[x as usize] / 100;
                    self.memory[(self.i + 1) as usize] = (self.v[x as usize] / 10) % 10;
                    self.memory[(self.i + 2) as usize] = (self.v[x as usize] % 100) % 10;
                }
                0x0055 => {
                    for i in 0..(x + 1) {
                        self.memory[((i as u16) + self.i) as usize] = self.v[i as usize];
                    }
                    self.i = (x as u16) + 1;
                }
                0x0065 => {
                    for i in 0..(x + 1) {
                        self.v[i as usize] = self.memory[(self.i + (i as u16)) as usize]
                    }
                    self.i = (x as u16) + 1;
                }
                _ => self.handle_invalid_op(op),
            },
            _ => self.handle_invalid_op(op),
        }
    }

    /// Increment the PC register by two, to get the next opcode.
    fn next_instruction(&mut self) {
        self.pc += 2;
    }

    fn handle_invalid_op(&self, op: u16) {
        println!("invalid opcode: {:#x}", op);
    }
}
