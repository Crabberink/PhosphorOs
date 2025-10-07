#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::fmt::Write;
use core::panic::PanicInfo;
use bootloader_api::entry_point;
use lazy_static::lazy_static;
use spin::Mutex;
mod serial;
mod vga_buffer;
mod writer;
mod interrupts;
mod gdt;
mod phosphor_os;
mod console;

use crate::console::Console;
use crate::serial::*;
use crate::writer::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut console = CONSOLE.lock();
    console.set_color(WriterColor::BrightRed);
    let _ = write!(console, "\n{}", _info);
    
    halt_loop();
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new(
        VGAWriter::new(unsafe { &mut *(0xb8000 as *mut vga_buffer::Text)}),
        SerialWriter::new(SerialPort::new(0x3F8))
    ));
}

entry_point!(phosphor_os::main);

fn halt_loop() -> ! {
    // Wait for interrupts indefinitely
    loop { 
        x86_64::instructions::hlt();
    }
}

#[macro_export]
macro_rules! breakpoint {
    () => {
        x86_64::instructions::interrupts::int3();
    };
}