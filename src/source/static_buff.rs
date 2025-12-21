use core::{
    alloc::Layout,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use crate::source::MemorySource;

pub struct StaticBuffer<const N: usize> {
    buffer: [MaybeUninit<u8>; N],
    offset: usize,
}

impl<const N: usize> MemorySource for StaticBuffer<N> {
    unsafe fn request_chunk(&mut self, size: usize) -> Option<NonNull<[u8]>> {
        let remaining = N - self.offset;

        if size > remaining {
            None
        } else {
            let start_ptr = unsafe {
                self.buffer.as_mut_ptr().cast::<u8>().add(self.offset)
            };
            self.offset += size;

            let slice_ptr = ptr::slice_from_raw_parts_mut(start_ptr, size);
            NonNull::new(slice_ptr)
        }
    }

    unsafe fn release_chunk(&mut self, _ptr: NonNull<u8>, _layout: Layout) {}
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
