use core::{alloc::Layout, ptr::NonNull};

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

    pub fn lock(&self) -> InnerGuard<'_, T> {
        self.inner.lock()
    }
}

impl<T: MemorySource> MemorySource for &Locked<T> {
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        unsafe { self.lock().request_chunk(layout) }
    }

    unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { self.lock().release_chunk(ptr, layout) }
    }
}

impl<T: MutAllocator> MutAllocator for &Locked<T> {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        unsafe { self.lock().alloc(layout) }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { self.lock().dealloc(ptr, layout) }
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
