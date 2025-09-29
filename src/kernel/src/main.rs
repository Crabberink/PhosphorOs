#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::fmt::Write;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;
mod serial;
mod writer;
mod interrupts;
mod gdt;
mod console;

use crate::console::Console;
use crate::gdt::setup_gdt;
use crate::interrupts::setup_idt;
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

    phosphoros_init();

    phosphoros_main();
    
    halt()
}

fn phosphoros_init() {
    setup_idt();
    setup_gdt();
}

fn phosphoros_main() {
    // let vga_buf = 0xb8000 as *mut u8;
    {
        let mut console = CONSOLE.lock();
        console.set_color(WriterColor::Yellow);
        console.print_str("\nPhosphor");
        console.set_color(WriterColor::White);
        console.print_str("OS");
    }

    breakpoint!();

    cause_a_fucking_stack_overflow(3);
}

fn halt() -> ! {
    loop { }
}

#[macro_export]
macro_rules! breakpoint {
    () => {
        x86_64::instructions::interrupts::int3();
    };
}

fn cause_a_fucking_stack_overflow(useless_param: u8) {
    cause_a_fucking_stack_overflow(useless_param);
}