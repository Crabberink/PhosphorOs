#![allow(dead_code)]

use core::fmt::{self, Write};

use crate::{writer::{FrameBufferWriter, SerialWriter, Writer, WriterColor}};

pub struct Console {
    serial_writer: SerialWriter,
    framebuffer_writer: FrameBufferWriter,
    is_initialized: bool,
}

impl Console {
    pub fn new(serial_writer: SerialWriter, framebuffer_writer: FrameBufferWriter) -> Console{
        Console {
            serial_writer,
            framebuffer_writer,
            is_initialized: false,
        }
    }
    pub fn print_char(&mut self, character: char) {
        let _ = self.serial_writer.write_char(character);
        let _ = self.framebuffer_writer.write_char(character);
    }
    pub fn print_str(&mut self, string: &str) {
        let _ = self.serial_writer.write_str(string);
        let _ = self.framebuffer_writer.write_str(string);
    }
    pub fn set_color(&mut self, color: WriterColor) {
        self.serial_writer.set_color(color);
        self.framebuffer_writer.set_color(color);
    }
    pub fn set_framebuffer_writer(&mut self, writer: FrameBufferWriter) {
        self.framebuffer_writer = writer;
        self.framebuffer_writer.clear();
    }
    pub fn backspace(&mut self) {
        self.serial_writer.backspace();
        self.framebuffer_writer.backspace();
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print_str(s);
        Ok(())
    }
}