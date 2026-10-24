use core::arch::naked_asm;

use alloc::{boxed::Box, collections::VecDeque, vec};
use conquer_once::spin::OnceCell;
use x86_64::instructions::interrupts;

use crate::{lock::IrqMutex, task::threading::{THREAD_STACK_SIZE, Thread}};

struct Scheduler {
    current: Box<Thread>,
    ready: VecDeque<Box<Thread>>,
}

static SCHEDULER: OnceCell<IrqMutex<Scheduler>> = OnceCell::uninit();

fn scheduler() -> &'static IrqMutex<Scheduler> {
    SCHEDULER.get().expect("Scheduler uninitialized")
}


pub fn init() {
    SCHEDULER.init_once(|| {
        IrqMutex::new(Scheduler {
            // Thread 0 is kmain
            current: Box::new(Thread {
                rsp: 0, 
                entry: None, 
                _stack: None
            }),
            ready: VecDeque::new(),
        })
    });
}

pub fn spawn_thread(entry: fn()) {
    let mut stack = vec![0u8; THREAD_STACK_SIZE].into_boxed_slice();
    let top = (stack.as_mut_ptr() as usize + THREAD_STACK_SIZE) & !0xF; // 16-byte aligned ig

    let mut sp = top as *mut usize;
    unsafe {
        sp = sp.sub(1);
        sp.write(0); // bs return addr
        sp = sp.sub(1);
        sp.write(thread_start as *const () as usize); // switch context ret location
        for _ in 0..6 { sp = sp.sub(1); sp.write(0); }
    }

    let thread = Box::new(Thread {
        rsp: sp as usize,
        entry: Some(entry),
        _stack: Some(stack),
    });

    scheduler().lock().ready.push_back(thread);
}

extern "C" fn thread_start() -> ! {
    interrupts::enable();
    let entry = scheduler().lock().current.entry.expect("Thread has no entry");
    entry();

    loop { yield_now(); }
}

/// Interrupts must be disabled
pub unsafe fn schedule() {
    let (old_rsp_slot, new_rsp_slot) = {
        let mut s = scheduler().lock();
    
        let Some(next) = s.ready.pop_front() else { 
            return;
        };
    
        let mut prev = core::mem::replace(&mut s.current, next);
        let old_rsp_slot = &mut prev.rsp as *mut usize;
        let new_rsp_slot = s.current.rsp;
        s.ready.push_back(prev);
    
        (old_rsp_slot, new_rsp_slot)
    };
    
    unsafe { switch_context(old_rsp_slot, new_rsp_slot) };
}

pub fn yield_now() {
    let enabled = interrupts::are_enabled();
    interrupts::disable();

    unsafe { schedule(); }

    if enabled { interrupts::enable(); }
}

pub fn has_ready_threads() -> bool {
    !scheduler().lock().ready.is_empty()
}

#[unsafe(naked)]
// Fyi this bullshit has arg 1 in rdi and arg 2 in rsi
unsafe extern "sysv64" fn switch_context(old_rsp_slot: *mut usize, new_rsp: usize) {
    naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        "mov [rdi], rsp", // Put stack pointer into old_rsp_slot
        "mov rsp, rsi",   // Change stack pointer to new_rsp

        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",

        "ret"
    );
}