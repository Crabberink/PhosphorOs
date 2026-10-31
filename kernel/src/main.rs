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

mod serial;
mod writer;
mod interrupts;
mod gdt;
mod kernel;
mod keyboard;
mod lock;
mod memory;
mod threads;
mod task;

use crate::writer::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        WRITER.force_unlock();
    }

    let mut console = WRITER.lock();
    console.set_color(WriterColor::BrightRed);
    let _ = write!(console, "\n{}", _info);
    
    halt_loop();
}

const COM1: u16 = 0x3F8;

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