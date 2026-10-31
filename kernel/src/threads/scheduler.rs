use core::arch::naked_asm;

use alloc::{boxed::Box, collections::{BinaryHeap, VecDeque}, vec};
use conquer_once::spin::OnceCell;
use x86_64::instructions::interrupts;

use crate::{cprintln, halt_loop, interrupts::timer::{PIT_BASE_HZ, closest_pit_divisor_for, set_pit_divisor}, lock::IrqMutex, threads::{THREAD_STACK_SIZE, Thread, sleeper::Sleeper}, writer::WriterColor};

struct Scheduler {
    current: Box<Thread>,
    ready: VecDeque<Box<Thread>>,
    sleeping: BinaryHeap<Sleeper>,
    idle: Option<Box<Thread>>,
    pit_divisor: u32,
    accumulated_ticks: u64,
}

static SCHEDULER: OnceCell<IrqMutex<Scheduler>> = OnceCell::uninit();

fn scheduler() -> &'static IrqMutex<Scheduler> {
    SCHEDULER.get().expect("Scheduler uninitialized")
}

/// Requires the PIT to be set up
pub unsafe fn init() {
    cprintln!(WriterColor::White, "Initializing thread scheduler");

    let divisor = closest_pit_divisor_for(1000);
    unsafe { set_pit_divisor(divisor); }

    let divisor_u32 = if divisor == 0 { 65536 } else { divisor as u32 };

    SCHEDULER.init_once(|| {
        IrqMutex::new(Scheduler {
            // Thread 0 is kmain
            current: Box::new(Thread {
                rsp: 0, 
                entry: None, 
                _stack: None
            }),
            ready: VecDeque::new(),
            idle: Some(create_thread(|| { halt_loop() })),
            sleeping: BinaryHeap::new(),
            pit_divisor: divisor_u32,
            accumulated_ticks: 0,
        })
    });
}

fn create_thread(entry: fn()) -> Box<Thread> {
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

    Box::new(Thread {
        rsp: sp as usize,
        entry: Some(entry),
        _stack: Some(stack),
    })
}

pub fn spawn_thread(entry: fn()) {
    scheduler().lock().ready.push_back(create_thread(entry));
}

extern "C" fn thread_start() -> ! {
    interrupts::enable();
    let entry = scheduler().lock().current.entry.expect("Thread has no entry");
    entry();

    loop { yield_now(); }
}

/// Interrupts must be disabled
pub unsafe fn tick() {
    {
        let mut s = scheduler().lock();
        s.accumulated_ticks += 1;

        while let Some(sleeper) = s.sleeping.peek() {
            if sleeper.wake_at > s.accumulated_ticks { break; }

            let sleeper = s.sleeping.pop().unwrap();

            s.ready.push_back(sleeper.thread);
        }
    }

    unsafe { schedule() }
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

        if s.idle.is_none() {
            s.idle = Some(prev);
        } else {
            s.ready.push_back(prev);
        }
    
        (old_rsp_slot, new_rsp_slot)
    };
    
    unsafe { switch_context(old_rsp_slot, new_rsp_slot) };
}

fn ticks_for_interval_with_divisor(ms: u64, divisor: u32) -> u64 {
    let numerator = ms as u128 * PIT_BASE_HZ as u128; // No overflows allowed fuck you
    let denominator = 1000 * divisor as u128;
    
    u64::try_from(numerator.div_ceil(denominator)).unwrap_or(u64::MAX)
}

pub fn sleep(ms: u64) {
    let enabled = interrupts::are_enabled();
    interrupts::disable();

    let (old_rsp_slot, new_rsp_slot) = {
        let mut s = scheduler().lock();
    
        let next = match s.ready.pop_front() {
            Some(next) => next,
            None => s.idle.take().expect("Idle thread is missing?"),
        };

        let mut prev = core::mem::replace(&mut s.current, next);
    
        let old_rsp_slot = &mut prev.rsp as *mut usize;
        let new_rsp_slot = s.current.rsp;

        let target_ticks = s.accumulated_ticks + ticks_for_interval_with_divisor(ms, s.pit_divisor) + 1;
        let sleeper = Sleeper {
            thread: prev,
            wake_at: target_ticks,
        };

        s.sleeping.push(sleeper);
    
        (old_rsp_slot, new_rsp_slot)
    };

    unsafe { switch_context(old_rsp_slot, new_rsp_slot) };

    if enabled { interrupts::enable(); }
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