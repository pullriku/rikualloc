use core::{
    alloc::Layout,
    cell::UnsafeCell,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::source::MemorySource;

pub struct StaticBuffer<const N: usize> {
    buffer: UnsafeCell<[MaybeUninit<u8>; N]>,
    taken: AtomicBool,
}

unsafe impl<const N: usize> Sync for StaticBuffer<N> {}

impl<const N: usize> MemorySource for &StaticBuffer<N> {
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        unsafe { self.request_chunk_impl(layout) }
    }

    unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.release_chunk_impl(ptr, layout)
    }
}

impl<const N: usize> StaticBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buffer: UnsafeCell::new([MaybeUninit::uninit(); N]),
            taken: AtomicBool::new(false),
        }
    }

    unsafe fn request_chunk_impl(
        &self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        if self.taken.swap(true, Ordering::AcqRel) {
            return None;
        }

        let base = unsafe { (*self.buffer.get()).as_mut_ptr().cast::<u8>() };

        let pad = base.align_offset(layout.align());
        if pad == usize::MAX || pad > N {
            return None;
        }

        let avail = N - pad;
        if avail < layout.size() {
            return None;
        }

        let start = unsafe { base.add(pad) };
        let nn = NonNull::new(start)?;

        Some(NonNull::slice_from_raw_parts(nn, avail))
    }

    fn release_chunk_impl(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

impl<const N: usize> Default for StaticBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}
