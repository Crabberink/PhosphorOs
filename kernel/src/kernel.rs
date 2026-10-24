use bootloader_api::BootInfo;
use bootloader_api::info::{FrameBuffer, Optional};
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::{VirtAddr};
use crate::keyboard::process_keypresses;
use crate::memory::allocator::init_heap;
use crate::memory::BootInfoFrameAllocator;
use crate::println;
use crate::task::threading::scheduler;
use crate::task::{Task, threading};
use crate::task::executor::Executor;
use crate::{gdt::setup_gdt, interrupts::{setup_idt, PICS}, writer::{FrameBufferWriter, WriterColor}, WRITER, cprint, cprintln, memory};

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
        
        let mut console = WRITER.lock();
        
        let framebuffer_writer = FrameBufferWriter::new(buffer, info);
        
        console.set_framebuffer_writer(framebuffer_writer);
    });
}

pub fn main(boot_info: &'static mut BootInfo) -> ! {
    // So apparently we gotta execute a heist on this bitch and steal it from boot_info
    let framebuffer = core::mem::replace(&mut boot_info.framebuffer, Optional::None).into_option();

    // If it exists we initialize a writer for it
    if let Some(framebuffer) = framebuffer {
        init_framebuffer_writer(framebuffer);
    }

    init();

    let Some(physical_memory_offset) = boot_info.physical_memory_offset.into_option() else {
        panic!("Bootloader did not map physical memory!");
    };

    let mut mem_mapper = unsafe { memory::init(VirtAddr::new(physical_memory_offset)) };
    
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) }; 

    init_heap(&mut mem_mapper, &mut frame_allocator).expect("Failed to initialize heap");

    threading::scheduler::init();

    cprint!(WriterColor::BrightYellow, "\nPhosphor");
    cprintln!(WriterColor::White, "OS");

    scheduler::spawn_thread(|| loop { println!("A"); });
    scheduler::spawn_thread(|| loop { println!("B"); });

    let mut executor = Executor::new();
    executor.spawn(Task::new(process_keypresses()));
    // executor.spawn(Task::new(test_long_task()));

    executor.run();
}

pub async fn test_long_task() {
    let val = test_uncooperative_number().await;

    cprintln!(WriterColor::BrightOrange, "Num {}", val);
}

async fn test_uncooperative_number() -> f64 { 
    let mut n = 1000.0;
    for i in 0..5000 {
        n += i as f64;
        for b in 0..100 {
            n -= b as f64;
        }
    }

    n
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