use volatile::Volatile;

use crate::writer::WriterColor;

pub const HEIGHT: usize = 25;
pub const WIDTH: usize = 80;

#[repr(transparent)]
pub struct Text {
    pub chars: [[Volatile<VGAChar>; WIDTH]; HEIGHT]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct VGAChar {
    pub ascii: u8,
    pub color: VGAColor
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct VGAColor(u8);

impl VGAColor {
    pub fn new(foreground: WriterColor, background: WriterColor) -> VGAColor {
        VGAColor((background as u8) << 4 | foreground as u8)
    }
    pub fn set_foreground(&mut self, color: WriterColor) {
        self.0 = (self.0 | 0xF0) | (color as u8);
    }
    pub fn set_background(&mut self, color: WriterColor) {
        self.0 = (self.0 | 0x0F) | (color as u8) << 4;
    }
}