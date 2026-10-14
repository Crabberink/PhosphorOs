use core::{mem::ManuallyDrop, ops::{Deref, DerefMut}};

use ::spin::{Mutex, MutexGuard};
use x86_64::instructions::interrupts;

pub struct IrqMutex<T> {
    inner: Mutex<T>
}

pub struct IrqGuard<'a, T> {
    gaurd: ManuallyDrop<MutexGuard<'a, T>>,
    were_enabled: bool,
}

impl<T> IrqMutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    pub fn lock(&self) -> IrqGuard<'_, T> {
        let were_enabled = interrupts::are_enabled();
        interrupts::disable();
        IrqGuard {
            gaurd: ManuallyDrop::new(self.inner.lock()),
            were_enabled
        }
    }

    pub unsafe fn force_unlock(&self) {
        unsafe { self.inner.force_unlock() }
    }
}

impl<T> Deref for IrqGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.gaurd
    }
}

impl<T> DerefMut for IrqGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.gaurd
    }
}

impl<T> Drop for IrqGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.gaurd); }
        if self.were_enabled {
            interrupts::enable();
        }
    }
}