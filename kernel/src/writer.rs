#![allow(dead_code)]

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::RasterizedChar;

use crate::{ serial::SerialPort, writer::noto_sans_mono_spacing::{BACKUP_CHAR, FONT_WEIGHT, RASTER_HEIGHT}};
use core::{fmt, ptr::{self}};


pub trait Writer: fmt::Write {
    fn print(&mut self, text: &str);
    fn set_color(&mut self, color: WriterColor);
    fn print_with_color(&mut self, text: &str, color: WriterColor) {
        self.set_color(color);
        self.print(text);
    }
    fn is_valid(&self) -> bool;
    fn write_string(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
    fn backspace(&mut self) { }
}

pub struct SerialWriter {
    port: SerialPort,
    valid: bool
}

impl SerialWriter {
    pub fn new(port: SerialPort) -> SerialWriter {
        SerialWriter {
            valid: port.is_valid(),
            port
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
        self.valid
    }
    fn backspace(&mut self) {
        self.port.write_serial(0x08); // Backspace character
        self.port.write_serial(b' ');   // Overwrite with space
        self.port.write_serial(0x08); // Move back again
    }
}
impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}

mod noto_sans_mono_spacing {
    use noto_sans_mono_bitmap::{get_raster_width, FontWeight};

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;

    pub const RASTER_HEIGHT: noto_sans_mono_bitmap::RasterHeight = noto_sans_mono_bitmap::RasterHeight::Size16;
    pub const RASTER_WIDTH: usize = get_raster_width(FONT_WEIGHT, RASTER_HEIGHT);

    pub const BACKUP_CHAR: char = '?';
}

const DEFAULT_PADDING: usize = 1;
const DEFAULT_CHAR_SPACING: usize = 0;
const DEFAULT_LINE_SPACING: usize = 2;

pub struct FrameBufferWriter {
    frame_buffer: &'static mut [u8],
    line_spacing: usize,
    char_spacing: usize,
    padding: usize,
    buffer_info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
    color_r: u8,
    color_g: u8,
    color_b: u8,
    is_valid: bool,
}

impl FrameBufferWriter {
    pub fn new_empty() -> FrameBufferWriter {
        FrameBufferWriter {
            frame_buffer: unsafe { &mut *(0x8000 as *mut [u8; 0]) },
            line_spacing: 0,
            char_spacing: 0,
            padding: 0,
            buffer_info: FrameBufferInfo { 
                byte_len: 0,
                width: 0, 
                height: 0, 
                pixel_format: PixelFormat::Rgb, 
                bytes_per_pixel: 0, 
                stride: 0
            },
            x_pos: 0,
            y_pos: 0,
            color_r: 0,
            color_g: 0,
            color_b: 0,
            is_valid: false,
        }
    }
    pub fn new(frame_buffer: &'static mut [u8], info: FrameBufferInfo) -> FrameBufferWriter {
        FrameBufferWriter {
            frame_buffer,
            buffer_info:  info,
            char_spacing: DEFAULT_CHAR_SPACING,
            line_spacing: DEFAULT_LINE_SPACING,
            padding: DEFAULT_PADDING,
            x_pos: 0,
            y_pos: 0,
            color_r: 255,
            color_g: 255,
            color_b: 255,
            is_valid: true,
        }
    }
    // Scroll later
    fn new_line(&mut self) {
        self.y_pos += noto_sans_mono_spacing::RASTER_HEIGHT.val() + self.char_spacing;
        self.carriage_return();
    }

    fn carriage_return(&mut self) {
        self.x_pos = self.padding;
    }

    pub fn clear(&mut self) {
        if !self.is_valid { return; }
        self.frame_buffer.fill(0);
    }

    fn write_char(&mut self, char: char) {
        fn get_raster_char(c: char) -> RasterizedChar {
            fn get(c: char) -> Option<RasterizedChar> {
                noto_sans_mono_bitmap::get_raster(c, FONT_WEIGHT, RASTER_HEIGHT)
            }

            get(c).unwrap_or_else(|| get(BACKUP_CHAR).unwrap())
        }

        match char {
            '\n' => self.new_line(),
            '\r' => self.carriage_return(),
            c => {
                let x_pos = self.x_pos + noto_sans_mono_spacing::RASTER_WIDTH + self.char_spacing;
                if x_pos >= self.buffer_info.width - self.padding {
                    self.new_line();
                }
                let y_pos = self.y_pos + noto_sans_mono_spacing::RASTER_HEIGHT.val() + self.line_spacing;
                if y_pos >= self.buffer_info.height - self.padding {
                    self.y_pos = self.padding;
                    self.clear();
                }
                self.display_raster_char(get_raster_char(c));
            }
        }
    }

    fn display_raster_char(&mut self, char: RasterizedChar) {
        for (y,row) in char.raster().iter().enumerate() {
            for (x,byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += self.char_spacing + char.width();
    }
    // Writes a greyscale pixel to the buffer at x y. The brightness is value. 
    fn write_pixel(&mut self, x:usize, y:usize, value:u8) {
        let color = match self.buffer_info.pixel_format {
            PixelFormat::Rgb => {
                [
                    ((self.color_r as u16 * value as u16) / 255) as u8,
                    ((self.color_g as u16 * value as u16) / 255) as u8,
                    ((self.color_b as u16 * value as u16) / 255) as u8,
                    0,
                ]
            },
            PixelFormat::Bgr => {
                [
                    ((self.color_b as u16 * value as u16) / 255) as u8,
                    ((self.color_g as u16 * value as u16) / 255) as u8,
                    ((self.color_r as u16 * value as u16) / 255) as u8,
                    0
                ]
            },
            PixelFormat::U8 => {
                [
                    if value > 200 { 0xf } else { 0 },
                    0,0,0
                ]
            },
            format => {
                self.buffer_info.pixel_format = PixelFormat::Rgb;
                panic!("Unknown frame buffer color format {:?}", format);
            }
        };
        let bytes_per_pixel = self.buffer_info.bytes_per_pixel;
        let offset = (y * self.buffer_info.stride + x) * bytes_per_pixel;
        self.frame_buffer[offset..(offset + bytes_per_pixel)].copy_from_slice(&color[..bytes_per_pixel]);

        let _ = unsafe { ptr::read_volatile(&self.frame_buffer[offset]) };
    }
}

impl Writer for FrameBufferWriter {
    fn is_valid(&self) -> bool {
        self.is_valid
    }
    fn print(&mut self, text: &str) {
        if !self.is_valid { return; }
        for char in text.chars() {
            self.write_char(char);
        }
    }
    fn set_color(&mut self, color: WriterColor) {
        let color: [u8; 3] = match color {
            WriterColor::Red => [128,0,0],
            WriterColor::Yellow => [128,128,0],
            WriterColor::Green => [0,128,0],
            WriterColor::Blue => [0,0,128],
            WriterColor::Magenta => [188,0,188],
            WriterColor::Cyan => [0,128,128],
            WriterColor::White => [255,255,255],
            WriterColor::Black => [0,0,0],
            WriterColor::Gray => [128,128,128],
            WriterColor::Brown => [100, 64, 0],
            WriterColor::BrightBlue => [0,0,255],
            WriterColor::BrightCyan => [0,255,255],
            WriterColor::BrightGreen => [0,255,0],
            WriterColor::BrightMagenta => [255,0,255],
            WriterColor::BrightRed => [255,0,0],
        };
        self.color_r = color[0];
        self.color_g = color[1];
        self.color_b = color[2];
    }
    fn backspace(&mut self) {
        if self.x_pos < noto_sans_mono_spacing::RASTER_WIDTH - self.char_spacing {
            return;
        }
        let x_pos = self.x_pos - noto_sans_mono_spacing::RASTER_WIDTH - self.char_spacing;
        self.x_pos = x_pos;
        self.write_char(' ');
        self.x_pos = x_pos;
    }
}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}