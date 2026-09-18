use bootloader_api::BootInfo;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::registers::control::Cr3;
use crate::{breakpoint, gdt::setup_gdt, halt_loop, interrupts::{setup_idt, PICS}, writer::{FrameBufferWriter, WriterColor}, CONSOLE, cprint, cprintln, println};

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

        pics.write_masks(0, 0); // Unmask all interrupts
    }
    x86_64::instructions::interrupts::enable();
}

pub fn main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    cprint!(WriterColor::Yellow, "\nPhosphor");
    cprintln!(WriterColor::White, "OS");

    breakpoint!();

    let (l4_page_table, _flags) = Cr3::read();

    println!("L4 Page Table at: {:?}", l4_page_table.start_address());

    // let ptr = 0xb00bf01d as *mut u8;
    // unsafe { *ptr = 69; };

    // cause_a_fucking_stack_overflow(0);

    halt_loop()
}

#[allow(unconditional_recursion)]
#[allow(dead_code)]
fn cause_a_fucking_stack_overflow(arg: u32) -> ! {
    if arg == 0 {
        without_interrupts(|| {
            let mut console = CONSOLE.lock();
            console.set_color(WriterColor::BrightGreen);
            console.print_str("\nCausing a stack overflow hooray!");
        });
    }
    cause_a_fucking_stack_overflow(arg.wrapping_add(1));
}