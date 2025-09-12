#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;
mod serial;
mod writer;

use crate::serial::*;
use crate::writer::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut writer = SERIALWRITER.lock();
    writer.set_color(WriterColor::BrightRed);
    // if let Some(location) = _info.location() {
    //     if let Some(message) = _info.message().as_str() {
    //         let _ = write!(writer, "\nPanicked at {}:{}:{}\n\t'{}'", location.file(), location.line(), location.column(), message);
    //     } else {
    //         let _ = write!(writer, "\nPanicked at {}:{}:{}", location.file(), location.line(), location.column());
    //     }
    // } else {
    //     if let Some(message) = _info.location() {
    //         let _ = write!(writer, "\nThread Panicked:\n\t'{}'", message);
    //     } else {
    //         let _ = write!(writer, "\nThread Panicked! No location or message available!");
    //     }
    // }
    let _ = write!(writer, "\n{}", _info);
    loop { }
}

lazy_static! {
    pub static ref SERIALWRITER: Mutex<SerialWriter> = Mutex::new(SerialWriter::new(
        SerialPort::new(0x3F8)
    ));
}
static HELLO: &str = "Hello";
static WORLD: &str = " World!";

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // let vga_buf = 0xb8000 as *mut u8;


    {
        let mut terminal = SERIALWRITER.lock();
        terminal.set_color(WriterColor::Cyan);
        terminal.print(HELLO);
        terminal.set_color(WriterColor::BrightCyan);
        terminal.print(WORLD);
    }

    panic!("This is a test panic...");    

    halt()
}

fn halt() -> ! {
    loop { }
}