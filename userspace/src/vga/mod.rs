use sip;

pub const HEIGHT: usize = 25;
pub const WIDTH: usize = 80;

pub struct Vga {
    writer: Writer,
}

impl Vga {
    pub fn open() -> Vga {
        let buffer = sip::physical_map::<Buffer>(0xb8000).unwrap();
        Vga {
            writer: Writer {
                col: 0,
                row: 0,
                color: ColorCode::new(Color::White, Color::Black),
                buffer,
            },
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..HEIGHT {
            self.writer.clear_row(row);
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.writer.write_bytes(bytes);
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    byte: u8,
    color: ColorCode,
}

impl ScreenChar {
    fn write(&mut self, new: ScreenChar) {
        unsafe { (self as *mut ScreenChar).write_volatile(new); }
    }

    fn read(&self) -> ScreenChar {
        unsafe { (self as *const ScreenChar).read_volatile() }
    }
}

struct Buffer {
    chars: [[ScreenChar; WIDTH]; HEIGHT],
}

struct Writer {
    col: usize,
    row: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    fn newline(&mut self) {
        if self.row == HEIGHT - 1 {
            for row in 1..HEIGHT {
                for col in 0..WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(HEIGHT-1);
        } else {
            self.row += 1;
        }

        self.col = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            byte: b' ',
            color: self.color,
        };

        for col in 0..WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.col >= WIDTH {
                    self.newline();
                }

                let row = self.row;
                let col = self.col;

                let color = self.color;
                self.buffer.chars[row][col].write(ScreenChar {
                    byte,
                    color,
                });
                self.col += 1;
            }
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            match byte {
                0x20...0x7e | b'\n' => self.write_byte(*byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}