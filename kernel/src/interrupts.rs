use core::fmt::Write;

use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::gdt::DOUBLE_FAULT_STACK_INDEX;
use crate::writer::WriterColor;
use crate::CONSOLE;
use crate::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Interrupts
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_STACK_INDEX);
        }

        // Hardware interrupts
        idt[HardwareInterruptIndex::Timer as u8].set_handler_fn(timer_handler);
        idt[HardwareInterruptIndex::Keyboard as u8].set_handler_fn(keyboard_handler);

        idt
    };
}

pub fn setup_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    let mut console = CONSOLE.lock();
    console.set_color(WriterColor::BrightBlue);
    let _ = write!(console, "\nBREAKPOINT:\n{:#?}",stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _err_code: u64) -> ! {
    let mut console = CONSOLE.lock();
    console.set_color(WriterColor::Yellow);
    let _ = write!(console, "\nPhosphorOS has experienced a ");
    console.set_color(WriterColor::Red);
    let _ = write!(console, "FATAL ERROR!\n");
    console.set_color(WriterColor::BrightRed);
    let _ = write!(console, "A DOUBLE FAULT EXCEPTION HAS OCCURRED\n");
    let _ = write!(console, "The following information is available:\n{:#?}",stack_frame);
    loop { }
}

pub const PIC_1_OFFSET: u8 = 32;                // All offsets need to be multiples of 8 because
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;  // Lower 3 bits are reserved and not counted

pub static PICS: spin::Mutex<ChainedPics> = Mutex::new(
unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HardwareInterruptIndex {
    Timer = PIC_1_OFFSET + 0,
    Keyboard,
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    let mut console = CONSOLE.lock();
    console.print_str(".");

    unsafe {
        let mut pics = PICS.lock();
        pics.notify_end_of_interrupt(HardwareInterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    let mut console = CONSOLE.lock();
    console.print_str("\nKeyboard interrupt raised");

    unsafe {
        let mut pics = PICS.lock();
        pics.notify_end_of_interrupt(HardwareInterruptIndex::Keyboard as u8);
    }
}