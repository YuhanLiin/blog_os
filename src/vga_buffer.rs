use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Gray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: u8,
}

impl ScreenChar {
    fn new(foreground: Color, background: Color, ascii_character: u8) -> ScreenChar {
        let color_code = (background as u8) << 4 | foreground as u8;
        ScreenChar {
            color_code,
            ascii_character,
        }
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    foreground: Color,
    background: Color,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.buffer.chars[self.row_position][self.column_position].write(ScreenChar::new(
                    self.foreground,
                    self.background,
                    byte,
                ));
                self.column_position += 1;
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            for r in 1..BUFFER_HEIGHT {
                for c in 0..BUFFER_WIDTH {
                    let ch = self.buffer.chars[r][c].read();
                    self.buffer.chars[r - 1][c].write(ch);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        }

        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar::new(self.foreground, Color::Black, b' ');

        for c in 0..BUFFER_WIDTH {
            self.buffer.chars[row][c].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        foreground: Color::Blue,
        background: Color::White,
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => ($crate::vga_buffer::_print(format_args!($($args)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($args:tt)*) => ($crate::print!("{}\n", format_args!($($args)*)));
}

#[cfg(test)]
mod test {
    use super::*;

    test!(test_println {
        println!("Simple output");
    });

    test!(test_print_many {
        for _ in 0..200 {
            println!("output");
        }
    });

    test!(test_print_output {
        let s = "Single line";
        print!("{}", s);

        let writer = WRITER.lock();
        for (i, c) in s.chars().enumerate() {
            let schar = writer.buffer.chars[writer.row_position][i].read();
            assert_eq!(char::from(schar.ascii_character), c);
        }
    });
}
