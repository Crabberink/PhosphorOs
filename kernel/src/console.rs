#![allow(dead_code)]

use core::fmt::{self, Write};

use crate::{breakpoint, writer::{SerialWriter, VGAWriter, Writer, WriterColor}};

pub struct Console {
    vga_writer: VGAWriter,
    serial_writer: SerialWriter,
}

impl Console {
    pub fn new(vga_writer: VGAWriter, serial_writer: SerialWriter) -> Console{
        Console {
            vga_writer: vga_writer,
            serial_writer: serial_writer,
        }
    }
    pub fn print_str(&mut self, string: &str) {
        let _ = self.serial_writer.write_str(string);
        // let _ = self.vga_writer.write_str(string);
    }
    pub fn set_color(&mut self, color: WriterColor) {
        self.serial_writer.set_color(color);
        // self.vga_writer.set_color(color);
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print_str(s);
        Ok(())
    }
}