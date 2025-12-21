use core::mem::MaybeUninit;

use crate::source::MemorySource;

pub struct StaticBuffer<const N: usize> {
    buffer: [MaybeUninit<u8>; N],
    offset: usize,
}

impl<const N: usize> MemorySource for StaticBuffer<N> {
    unsafe fn release_chunk(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        todo!()
    }

    unsafe fn request_chunk(&self, size: usize) -> Option<core::ptr::NonNull<[u8]>> {
        todo!()
    }
}

impl<const N: usize> StaticBuffer<N> {
    pub fn new() -> Self {
        Self {
            buffer: [MaybeUninit::uninit(); N],
            offset: 0,
        }
    }
}

impl<const N: usize> Default for StaticBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}
