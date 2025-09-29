#![allow(dead_code)]

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
    color: u8
}

impl VGAWriter {
    pub fn new(width: u8, height: u8) -> VGAWriter {
        VGAWriter {
            width: width,
            height: height,
            cursor_x: 0,
            cursor_y: 0,
            color: 0
        }
    }
}

impl Writer for VGAWriter {
    fn print(&mut self, _text: &str) {

    }
    fn set_color(&mut self, color: WriterColor) {
        self.color = color as u8;
    }
    fn print_with_color(&mut self, text: &str, color: WriterColor) {
        self.set_color(color);
        self.print(text);
    }
    fn is_valid(&self) -> bool {
        return true;
    }
}

impl fmt::Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
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
            WriterColor::Gray => b"90",
            WriterColor::BrightRed => b"91",
            WriterColor::BrightGreen => b"92",
            WriterColor::BrightBlue => b"94",
            WriterColor::BrightMagenta => b"95",
            WriterColor::BrightCyan => b"96",
            WriterColor::White => b"97",
            _ => b"37"
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
#[derive(Clone, Copy)]
pub enum WriterColor {
    Gray = 7,
    Red = 4,
    Yellow = 0xe,
    Green = 2,
    Blue = 1,
    Magenta = 5,
    Cyan = 3,
    Black = 0,
    Brown = 6,
    BrightRed = 0xc,
    BrightGreen = 0xa,
    BrightBlue = 0x9,
    BrightMagenta = 0xd,
    BrightCyan = 0xb,
    White = 0xf,
}

pub struct GlobalWriter {
    serial_writer: SerialWriter,
    vga_writer: VGAWriter,
}