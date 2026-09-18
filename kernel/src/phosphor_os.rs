use bootloader_api::BootInfo;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, Translate};
use x86_64::{PhysAddr, VirtAddr};
use crate::{gdt::setup_gdt, halt_loop, interrupts::{setup_idt, PICS}, writer::{FrameBufferWriter, WriterColor}, CONSOLE, cprint, cprintln, println, memory};

pub fn init(boot_info: &'static mut BootInfo) {
    without_interrupts(|| {
        let Some(framebuffer) = boot_info.framebuffer.take() else {
            return;
        };

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
    let Some(physical_memory_offset) = boot_info.physical_memory_offset.into_option() else {
        init(boot_info); // Still gotta init so we can print 😭🥀
        panic!("Bootloader did not map physical memory!");
    };

    init(boot_info);

    let phys_mem_offset = VirtAddr::new(physical_memory_offset);

    let mem_mapper = unsafe { memory::init(VirtAddr::new(physical_memory_offset)) };

    let addresses = [
        0x80001008,
        physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mem_mapper.translate_addr(virt);
        cprintln!(WriterColor::White, "{:?} -> {:?}", virt, phys);
    }

    cprint!(WriterColor::Yellow, "\nPhosphor");
    cprintln!(WriterColor::White, "OS");

    let (l4_page_table, _flags) = Cr3::read();

    println!("L4 Page Table at: {:?}", l4_page_table.start_address());

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