use alloc::{boxed::Box};

pub mod scheduler;
mod sleep;

pub struct Thread {
    rsp: usize,
    entry: Option<fn()>,
    _stack: Option<Box<[u8]>>
}

const THREAD_STACK_SIZE: usize = 32 * 1024;