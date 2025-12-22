use core::{
    alloc::Layout,
    mem::MaybeUninit,
    ptr::NonNull,
};

use crate::source::MemorySource;

pub struct StaticBuffer<const N: usize> {
    buffer: [MaybeUninit<u8>; N],
    offset: usize,
}

impl<const N: usize> MemorySource for &mut StaticBuffer<N> {
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        let size = layout.size();
        let align = layout.align();

        if self.offset > N {
            return None;
        }

        let base = self.buffer.as_mut_ptr().cast::<u8>();
        let start_ptr = unsafe { base.add(self.offset) };

        // paddingを「ポインタから」計算（provenanceを壊さない）
        let padding = start_ptr.align_offset(align);
        if padding == usize::MAX {
            return None;
        }

        let alloc_size = padding.checked_add(size)?;
        let new_offset = self.offset.checked_add(alloc_size)?;
        if new_offset > N {
            return None;
        }
        self.offset = new_offset;

        let aligned_ptr = unsafe { start_ptr.add(padding) };
        let nn = NonNull::new(aligned_ptr)?;
        Some(NonNull::slice_from_raw_parts(nn, size))
    }

    unsafe fn release_chunk(&mut self, _ptr: NonNull<u8>, _layout: Layout) {}
}

impl<const N: usize> StaticBuffer<N> {
    pub const fn new() -> Self {
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
