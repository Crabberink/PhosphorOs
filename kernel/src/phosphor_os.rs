use bootloader_api::BootInfo;
use x86_64::instructions::interrupts::without_interrupts;

use crate::{breakpoint, gdt::setup_gdt, halt_loop, interrupts::{setup_idt, PICS}, writer::WriterColor, CONSOLE};

pub fn init() {
    setup_gdt();
    setup_idt();
    
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
    }
    x86_64::instructions::interrupts::enable();
}

pub fn main(_boot_info: &'static mut BootInfo) -> ! {
    init();

    without_interrupts(|| {
        let mut console = CONSOLE.lock();
        console.set_color(WriterColor::Yellow);
        console.print_str("\nPhosphor");
    });

    without_interrupts(|| {
        let mut console = CONSOLE.lock();
        console.set_color(WriterColor::White);
        console.print_str("OS");
    });

    breakpoint!();

    halt_loop()
}