use bootloader_api::BootInfo;
use x86_64::instructions::interrupts::without_interrupts;
use core::fmt::Write;

use crate::{breakpoint, gdt::setup_gdt, halt_loop, interrupts::{setup_idt, PICS}, writer::{FrameBufferWriter, WriterColor}, CONSOLE};

pub fn init(boot_info: &'static mut BootInfo) {
    without_interrupts(|| {
        // I honestly have no clue what the fuck is going on here but the borrow checker stopped screaming
        let framebuffer_option = boot_info.framebuffer.take();

        if framebuffer_option.is_none() {
            return;
        }

        let framebuffer = framebuffer_option.unwrap();

        let info = framebuffer.info();
        let buffer = framebuffer.into_buffer();

        let mut console = CONSOLE.lock();

        let framebuffer_writer = FrameBufferWriter::new(buffer, info);

        console.set_framebuffer_writer(framebuffer_writer);
    });
    setup_gdt();
    setup_idt();
    
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
    }
    x86_64::instructions::interrupts::enable();
}

pub fn main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    without_interrupts(|| {
        let mut console = CONSOLE.lock();
        console.set_color(WriterColor::Yellow);
        console.print_str("\nPhosphor");
    });

    without_interrupts(|| {
        let mut console = CONSOLE.lock();
        console.set_color(WriterColor::White);
        console.print_str("OS\n");
    });

    breakpoint!();

    halt_loop()
}