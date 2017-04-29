extern crate piston_window;
extern crate image as im;
extern crate vecmath;

use piston_window::*;
use vecmath::*;

macro_rules! nnn { ($e:expr) => { $e & 0xfff } }
macro_rules! n { ($e:expr) => { ($e & 0xf) as u8 } }
macro_rules! x { ($e:expr) => { (($e >> 8) & 0xf) as u8 } }
macro_rules! y { ($e:expr) => { (($e >> 4) & 0xf) as u8 } }
macro_rules! kk { ($e:expr) => { ($e & 0xff) as u8 } }
macro_rules! x_i { ($e:expr) => { x!($e) as usize } }
macro_rules! y_i { ($e:expr) => { y!($e) as usize } }

struct Chip8 {
    mem: [u8; 0x1000],
    reg: [u8; 0x10],
    I: u16,
    PC: u16,
    SP: u8,
    stack: [u16; 0x10],
    delay_timer: u8,
    sound_timer: u8,
    display: [u8; 64 * 32],
}

impl Chip8 {
    fn new() -> Chip8 {
        // The sprite table of 0..0xf, each 5 bytes long
        let sprites: [u8; 0x50] =
            [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
             0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
             0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
             0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
             0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
             0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80];

        // Load the sprite table into memory
        let mut mem = [0; 0x1000];
        for i in 0..0x50 {
            mem[i] = sprites[i];
        }
        
        Chip8 {
            mem: [0; 0x1000],
            reg: [0; 0x10],
            I: 0,
            PC: 0,
            SP: 0,
            stack: [0; 0x10],
            sound_timer: 0,
            delay_timer: 0,
            display: [0; 64 * 32],
        }
    }

    fn fetch_opcode(&self) -> u16 {
        let hb = self.mem[self.I as usize] as u16;
        let lb = self.mem[(self.I + 1) as usize] as u16;
        (hb << 8) | lb
    }

    pub fn step(&mut self) {
        let op: u16 = self.fetch_opcode();

        // Save a lot of typing
        let mut mem = &mut self.mem;
        let mut reg = &mut self.reg;
        let mut stack = &mut self.stack;
        let mut I = self.I;
        let mut PC = self.PC;
        let mut SP = self.SP;

        match (op >> 12) & 0xf {
            0x0 => {
                match op & 0xff {
                    // CLS
                    0xe0 => {} // TODO
                    // RET
                    0xee => {
                        PC = stack[SP as usize];
                        SP -= 1;
                    }
                    // SYS addr
                    _ => {}
                }
            }
            // JP addr
            0x1 => {
                PC = nnn!(op);
            }
            // CALL addr
            0x2 => {
                SP += 1;
                stack[nnn!(op) as usize];
                PC = nnn!(op);
            }
            // SE Vx, byte
            0x3 => {
                if reg[x_i!(op)] == kk!(op) as u8 {
                    PC += 2;
                }
            }
            // SNE Vx, byte
            0x4 => {
                if reg[x_i!(op)] != kk!(op) {
                    PC += 2;
                }
            }
            // SE Vx, Vy
            0x5 => {
                if reg[x_i!(op)] == reg[y_i!(op)] {
                    PC += 2;
                }
            }
            // LD Vx, byte
            0x6 => {
                reg[x_i!(op)] = kk!(op);
            }
            // ADD Vx, byte
            0x7 => {
                reg[x_i!(op)] += kk!(op);
            }
            // LD Vx, Vy
            0x8 => {
                let vx = reg[x_i!(op)];
                let vy = reg[y_i!(op)];
                match op & 0xf {
                    0x0 => {
                        reg[x_i!(op)] += reg[y_i!(op)];
                    }
                    0x1 => {
                        reg[x_i!(op)] |= reg[y_i!(op)];
                    }
                    0x2 => {
                        reg[x_i!(op)] &= reg[y_i!(op)];
                    }
                    0x3 => {
                        reg[x_i!(op)] ^= reg[y_i!(op)];
                    }
                    0x4 => {
                        let sum: u16 = vx as u16 + vy as u16;
                        reg[x_i!(op)] = (sum & 0xf) as u8;
                        reg[0xf] = (sum > 0xff) as u8;
                    }
                    0x5 => {
                        reg[x_i!(op)] -= reg[y_i!(op)];
                        reg[0xf] = (vx > vy) as u8;
                    }
                    0x6 => {
                        reg[0xf] = vx & 0x1;
                        reg[x_i!(op)] = vx >> 2;
                    }
                    0x7 => {
                        reg[x_i!(op)] -= reg[y_i!(op)];
                        reg[0xf] = (vx < vy) as u8;
                    }
                    0x8 => {
                        reg[0xf] = (vx >> 0x7) & 0x1;
                        reg[x_i!(op)] = vx << 2;
                    }
                    _ => {}
                }
            }
            0x9 => {
                match op & 0xf {
                    0x0 => {
                        if reg[x_i!(op)] != reg[y_i!(op)] {
                            PC += 2;
                        }
                    }
                    _ => {}
                }
            }
            0xa => {
                I = nnn!(op);
            }
            0xb => {
                PC = nnn!(op) + reg[0] as u16;
            }
            0xc => {
                reg[x_i!(op)] = kk!(op) & 42;
            } // random
            0xd => unimplemented!(), // draw
            0xe => {
                match op & 0xff {
                    0x9e => unimplemented!(), // skip Vx
                    0xa1 => unimplemented!(),
                    _ => {}
                }
            }
            0xf => {
                match op & 0xff {
                    0x07 => unimplemented!(), // delay timer
                    0x0a => unimplemented!(), // input
                    0x15 => unimplemented!(), // delay timer
                    0x18 => unimplemented!(), //sound timer
                    0x1e => {
                        I += reg[x_i!(op)] as u16;
                    }
                    0x29 => unimplemented!(), // sprites
                    0x33 => {
                        let vx = reg[x_i!(op)];
                        mem[I as usize] = vx / 100;
                        mem[(I + 1) as usize] = (vx % 100) / 10;
                        mem[(I + 2) as usize] = vx % 10;
                    }
                    0x55 => {
                        for i in 0..0x10 {
                            mem[(I + i) as usize] = reg[i as usize];
                        }
                    }
                    0x65 => {
                        for i in 0..0x10 {
                            reg[i as usize] = mem[(I + i) as usize];
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_macros() {
    let myshort: u16 = 0xabcd;
    assert!(nnn!(myshort) == 0xbcd);
    assert!(n!(myshort) == 0xd);
    assert!(x!(myshort) == 0xb);
    assert!(y!(myshort) == 0xc);
    assert!(kk!(myshort) == 0xcd);
}

fn main() {
    let opengl = OpenGL::V3_2;
    let (width, height) = (300, 300);
    let mut window: PistonWindow = WindowSettings::new("piston: paint", (width, height))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let mut canvas = im::ImageBuffer::new(width, height);
    let mut draw = false;
    let mut texture = Texture::from_image(&mut window.factory, &canvas, &TextureSettings::new())
        .unwrap();

    let mut last_pos: Option<[f64; 2]> = None;

    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            texture.update(&mut window.encoder, &canvas).unwrap();
            window.draw_2d(&e, |c, g| {
                clear([1.0; 4], g);
                image(&texture, c.transform, g);
            });
        }
        if let Some(button) = e.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                draw = true;
            }
        };
        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                draw = false;
                last_pos = None
            }
        };
        if draw {
            if let Some(pos) = e.mouse_cursor_args() {
                let (x, y) = (pos[0] as f32, pos[1] as f32);

                if let Some(p) = last_pos {
                    let (last_x, last_y) = (p[0] as f32, p[1] as f32);
                    let distance = vec2_len(vec2_sub(p, pos)) as u32;

                    for i in 0..distance {
                        let diff_x = x - last_x;
                        let diff_y = y - last_y;
                        let delta = i as f32 / distance as f32;
                        let new_x = (last_x + (diff_x * delta)) as u32;
                        let new_y = (last_y + (diff_y * delta)) as u32;
                        if new_x < width && new_y < height {
                            canvas.put_pixel(new_x, new_y, im::Rgba([0, 0, 0, 255]));
                        };
                    }
                };

                last_pos = Some(pos)
            };

        }
    }
}
