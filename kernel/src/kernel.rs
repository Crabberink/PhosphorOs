use bootloader_api::BootInfo;
use bootloader_api::info::{FrameBuffer, Optional};
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::{VirtAddr};
use crate::interrupts::timer::{self, setup_pit};
use crate::keyboard::process_keypresses;
use crate::memory::allocator::init_heap;
use crate::memory::BootInfoFrameAllocator;
use crate::threads::scheduler::{self, sleep};
use crate::{threads};
use crate::task::{Task};
use crate::task::executor::Executor;
use crate::{gdt::setup_gdt, interrupts::{setup_idt, PICS}, writer::{FrameBufferWriter, WriterColor}, WRITER, cprint, cprintln, memory};

fn init() {
    setup_gdt();
    setup_idt();
    
    cprintln!(WriterColor::White, "Initializing pics...");
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
    }
    
    cprintln!(WriterColor::White, "Setting up pit...");
    setup_pit();
    unsafe { timer::set_pit_freq(5000) }

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

    cprintln!(WriterColor::White, "Initializing heap...");
    init_heap(&mut mem_mapper, &mut frame_allocator).expect("Failed to initialize heap");

    cprintln!(WriterColor::White, "Initializing thread scheduler...");
    unsafe { threads::scheduler::init(); }

    // Unmask all interrupts now that everything is set up
    unsafe { PICS.lock().write_masks(0, 0); }

    cprint!(WriterColor::BrightYellow, "\nPhosphor");
    cprintln!(WriterColor::White, "OS");

    let mut executor = Executor::new();
    executor.spawn(Task::new(process_keypresses()));

    scheduler::spawn_thread(|| { loop { cprintln!(WriterColor::White, "A"); sleep(1000); }});
    scheduler::spawn_thread(|| { loop { cprintln!(WriterColor::White, "B"); sleep(1000); }});

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