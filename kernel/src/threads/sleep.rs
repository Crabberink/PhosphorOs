use alloc::boxed::Box;

use crate::threads::Thread;

/// A sleeping thread
/// Shorter sleeping threads are considered > longer
pub struct Sleeper {
    pub wake_at: u64,
    pub thread: Box<Thread>,
}

impl PartialEq for Sleeper {
    fn eq(&self, other: &Self) -> bool {
        other.wake_at == self.wake_at
    }
}

impl Eq for Sleeper {}

impl PartialOrd for Sleeper {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Sleeper {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // The ordering is inverted so that later times are not at the top of the heap
        other.wake_at.cmp(&self.wake_at)
    }
}