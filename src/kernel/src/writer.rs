use crate::serial::SerialPort;
use core::fmt;

pub trait Writer: fmt::Write {
    fn print(&mut self, text: &str);
    fn set_color(&mut self, color: WriterColor);
    fn print_with_color(&mut self, text: &str, color: WriterColor);
    fn is_valid(&self) -> bool;
    fn write_string(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}

pub struct SerialWriter {
    port: SerialPort,
    valid: bool
}

pub struct VGAWriter {
    width: u8,
    height: u8,
    cursor_x: u8,
    cursor_y: u8,
}

impl SerialWriter {
    pub fn new(port: SerialPort) -> SerialWriter {
        SerialWriter {
            valid: port.is_valid(),
            port: port
        }
    }
}

impl Writer for SerialWriter {
    fn print(&mut self, text: &str) {
        for char in text.bytes() {
            self.port.write_serial(char);
        }
    }
    fn set_color(&mut self, color: WriterColor) {
        let color_code = match color {
            WriterColor::Black => b"30",
            WriterColor::Red => b"31",
            WriterColor::Green => b"32",
            WriterColor::Yellow => b"33",
            WriterColor::Blue => b"34",
            WriterColor::Magenta => b"35",
            WriterColor::Cyan => b"36",
            WriterColor::White => b"37",
            WriterColor::BrightBlack => b"90",
            WriterColor::BrightRed => b"91",
            WriterColor::BrightGreen => b"92",
            WriterColor::BrightYellow => b"93",
            WriterColor::BrightBlue => b"94",
            WriterColor::BrightMagenta => b"95",
            WriterColor::BrightCyan => b"96",
            WriterColor::BrightWhite => b"97",
        };

        // Write the color code command
        self.port.write_serial(b'\x1B');
        self.port.write_serial(b'[');
        self.port.write_serial(color_code[0]);
        self.port.write_serial(color_code[1]);
        self.port.write_serial(b'm');
    }
    fn print_with_color(&mut self, text: &str, color: WriterColor) {
        self.set_color(color);
        self.print(text);
    }
    fn is_valid(&self) -> bool {
        return self.valid;
    }
}
impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}

#[repr(u8)]
pub enum WriterColor {
    White,
    Red,
    Yellow,
    Green,
    Blue,
    Magenta,
    Cyan,
    Black,
    BrightRed,
    BrightYellow,
    BrightGreen,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    BrightBlack,
}