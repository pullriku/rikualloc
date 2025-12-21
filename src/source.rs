use core::{alloc::Layout, ptr::NonNull};

pub mod static_buff;

#[cfg(feature = "std")]
pub mod os_heap;

/// A source of memory
pub trait MemorySource {
    /// Request a chunk of memory.
    ///
    /// # Safety
    ///
    /// The chunk must be released with `release_chunk`
    unsafe fn request_chunk(&mut self, layout: Layout)
    -> Option<NonNull<[u8]>>;

    /// Release a chunk of memory
    ///
    /// # Safety
    ///
    /// The chunk must have been requested with `request_chunk`
    unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout);
}

impl<S: MemorySource + ?Sized> MemorySource for &mut S {
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        unsafe { <S as MemorySource>::request_chunk(&mut **self, layout) }
    }

    unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { <S as MemorySource>::release_chunk(&mut **self, ptr, layout) }
    }
}
