#[cfg(feature = "std")]
use std::sync::{Mutex as InnerMutex, MutexGuard as InnerGuard};

#[cfg(not(feature = "std"))]
use spin::{Mutex as InnerMutex, MutexGuard as InnerGuard};

pub struct Mutex<T> {
    inner: InnerMutex<T>,
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: InnerMutex::new(value),
        }
    }

    pub fn lock(&self) -> InnerGuard<'_, T> {
        #[cfg(feature = "std")]
        {
            self.inner.lock().expect("failed to lock mutex")
        }

        #[cfg(not(feature = "std"))]
        {
            self.inner.lock()
        }
    }
}
