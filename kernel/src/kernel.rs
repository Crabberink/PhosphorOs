use bootloader_api::BootInfo;
use bootloader_api::info::{FrameBuffer, Optional};
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::Translate;
use x86_64::{VirtAddr};
use crate::memory::BootInfoFrameAllocator;
use crate::{gdt::setup_gdt, halt_loop, interrupts::{setup_idt, PICS}, writer::{FrameBufferWriter, WriterColor}, CONSOLE, cprint, cprintln, println, memory};

fn init() {
    setup_gdt();
    setup_idt();
    
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();

        pics.write_masks(0, 0); // Unmask all interrupts
    }
    x86_64::instructions::interrupts::enable();
}

fn init_framebuffer_writer(framebuffer: FrameBuffer) {
    without_interrupts(|| {
        let info = framebuffer.info();
        let buffer = framebuffer.into_buffer();
        
        let mut console = CONSOLE.lock();
        
        let framebuffer_writer = FrameBufferWriter::new(buffer, info);
        
        console.set_framebuffer_writer(framebuffer_writer);
    });
}

pub fn main(boot_info: &'static mut BootInfo) -> ! {
    // So apparently we gotta execute heist on this bitch and steal it from boot_info
    let framebuffer = core::mem::replace(&mut boot_info.framebuffer, Optional::None).into_option();

    // If it exists we initialize a writer for it
    if let Some(framebuffer) = framebuffer {
        init_framebuffer_writer(framebuffer);
    }

    init();

    let Some(physical_memory_offset) = boot_info.physical_memory_offset.into_option() else {
        panic!("Bootloader did not map physical memory!");
    };

    let _phys_mem_offset = VirtAddr::new(physical_memory_offset);

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

    let _frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) }; 

    halt_loop()
}


#[allow(unconditional_recursion)]
#[allow(dead_code)]
fn cause_a_fucking_stack_overflow(arg: u32) -> ! {
    // if arg == 0 {
    //     without_interrupts(|| {
    //         let mut console = CONSOLE.lock();
    //         console.set_color(WriterColor::BrightGreen);
    //         console.print_str("\nCausing a stack overflow hooray!");
    //     });
    // }
    cause_a_fucking_stack_overflow(arg.wrapping_add(1));
}