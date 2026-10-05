#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use core::fmt::Write;
use core::panic::PanicInfo;
use bootloader_api::{entry_point, BootloaderConfig};
use bootloader_api::config::Mapping;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

mod serial;
mod writer;
mod interrupts;
mod gdt;
mod kernel;
mod console;
mod keyboard;
mod memory;
mod allocator;

use crate::console::Console;
use crate::writer::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        CONSOLE.force_unlock();
    }

    let mut console = CONSOLE.lock();
    console.set_color(WriterColor::BrightRed);
    let _ = write!(console, "\n{}", _info);
    
    halt_loop();
}

const SERIAL_OUT: u16 = 0x3F8;

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = {
        let port: Port<u8> = Port::new(SERIAL_OUT);
        Mutex::new(Console::new(
            SerialWriter::new(port),
            FrameBufferWriter::new_empty(),
        ))
    };
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(Mapping::Dynamic);

    config
};

entry_point!(kernel::main, config = &BOOTLOADER_CONFIG);

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