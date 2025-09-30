#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::fmt::Write;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;
mod serial;
mod writer;
mod interrupts;
mod gdt;
mod phosphor_os;
mod console;

use crate::console::Console;
use crate::gdt::setup_gdt;
use crate::interrupts::setup_idt;
use crate::interrupts::PICS;
use crate::serial::*;
use crate::writer::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut console = CONSOLE.lock();
    console.set_color(WriterColor::BrightRed);
    let _ = write!(console, "\n{}", _info);
    loop { }
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new(
        VGAWriter::new(40, 40),
        SerialWriter::new(SerialPort::new(0x3F8))
    ));
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {

    phosphor_os::init();
    
    phosphor_os::main();

    halt_loop()
}

fn halt_loop() -> ! {
    // Wait for interrupts indefinitely
    loop { 
        x86_64::instructions::hlt();
    }
}

// Runs at every timer interrupt interval
pub fn timer_loop() {
    
}

#[macro_export]
macro_rules! breakpoint {
    () => {
        x86_64::instructions::interrupts::int3();
    };
}