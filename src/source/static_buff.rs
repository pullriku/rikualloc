use core::{
    alloc::Layout,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use crate::{align::align_up, source::MemorySource};

pub struct StaticBuffer<const N: usize> {
    buffer: [MaybeUninit<u8>; N],
    offset: usize,
}

impl<const N: usize> MemorySource for StaticBuffer<N> {
    unsafe fn request_chunk(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        let start_ptr = unsafe {
            self.buffer.as_mut_ptr().cast::<u8>().add(self.offset)
        };
        let start = start_ptr as usize;

        let aligned_start = align_up(start, layout.align());
        let padding = aligned_start - start;

        let alloc_size = layout.size() + padding;
        
        if self.offset + alloc_size > N {
            return None;
        }
        
        self.offset += alloc_size;

        let ptr =  NonNull::new(aligned_start as *mut u8)?;
        Some(NonNull::slice_from_raw_parts(ptr, layout.size()))
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
