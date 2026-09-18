#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]

use core::fmt::Write;
use core::panic::PanicInfo;
use bootloader_api::{entry_point, BootloaderConfig};
use bootloader_api::config::Mapping;
use lazy_static::lazy_static;
use spin::Mutex;

mod serial;
mod writer;
mod interrupts;
mod gdt;
mod phosphor_os;
mod console;
mod keyboard;
mod memory;

use crate::console::Console;
use crate::serial::*;
use crate::writer::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut serial_writer = SerialWriter::new(SerialPort::new(0x3F8));
    // Backup message in case the CONSOLE is causing the error
    serial_writer.set_color(WriterColor::BrightRed);
    let _ = write!(serial_writer, "\n{}", _info);

    let mut console = CONSOLE.lock();
    console.set_color(WriterColor::BrightRed);
    let _ = write!(console, "\n{}", _info);
    
    halt_loop();
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new(
        SerialWriter::new(SerialPort::new(0x3F8)),
        FrameBufferWriter::new_empty(),
    ));
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(Mapping::Dynamic);

    config
};

entry_point!(phosphor_os::main, config = &BOOTLOADER_CONFIG);

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