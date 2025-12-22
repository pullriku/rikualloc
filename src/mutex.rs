use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{self, NonNull},
};
use std::alloc::{AllocError, Allocator};

use spin::{Mutex, MutexGuard};

use crate::{allocator::MutAllocator, source::MemorySource};

pub struct Locked<T> {
    inner: spin::Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    fn lock(&self) -> MutexGuard<'_, T> {
        self.inner.lock()
    }

    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.lock();
        f(&mut *guard)
    }
}

unsafe impl<T: MutAllocator> GlobalAlloc for Locked<T>
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

impl<T: MemorySource> MemorySource for &Locked<T>
{
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        self.with_lock(|value| unsafe { value.request_chunk(layout) })
    }

    unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.with_lock(|value| unsafe { value.release_chunk(ptr, layout) })
    }
}

unsafe impl<T: MutAllocator> Allocator for &Locked<T> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.with_lock(|value| unsafe { value.alloc(layout) }).ok_or(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.with_lock(|value| unsafe { value.dealloc(ptr, layout) })
    }
}
