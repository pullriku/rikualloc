use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{self, NonNull},
};

#[cfg(feature = "std")]
use std::sync::{Mutex as InnerMutex, MutexGuard as InnerGuard};

#[cfg(not(feature = "std"))]
use spin::{Mutex as InnerMutex, MutexGuard as InnerGuard};

use crate::{allocator::MutAllocator, source::MemorySource};

pub struct Locked<T> {
    inner: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    fn lock(&self) -> InnerGuard<'_, T> {
        self.inner.lock()
    }

    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut g = self.lock();
        f(&mut *g)
    }
}

unsafe impl<T: MutAllocator> GlobalAlloc for Locked<T>
where
    for<'a> &'a mut T: MutAllocator,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.with_lock(|value| match unsafe { value.alloc(layout) } {
            Some(ptr) => ptr.as_ptr().cast::<u8>(),
            None => ptr::null_mut(),
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ptr) = NonNull::new(ptr) {
            self.with_lock(|value| {
                unsafe { value.dealloc(ptr, layout) };
            })
        }
    }
}

impl<T> MemorySource for &Locked<T>
where
    for<'a> &'a mut T: MemorySource,
{
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        self.with_lock(|mut value| unsafe { value.request_chunk(layout) })
    }

    unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.with_lock(|mut value| unsafe { value.release_chunk(ptr, layout) })
    }
}

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
