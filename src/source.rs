use core::{alloc::Layout, ptr::NonNull};

/// A source of memory
pub trait MemorySource {
    /// Request a chunk of memory.
    ///
    /// # Safety
    ///
    /// The chunk must be released with `release_chunk`
    unsafe fn request_chunk(&self, size: usize) -> Option<NonNull<[u8]>>;

    /// Release a chunk of memory
    ///
    /// # Safety
    ///
    /// The chunk must have been requested with `request_chunk`
    unsafe fn release_chunk(&self, ptr: NonNull<u8>, layout: Layout);
}

pub mod static_buff;

#[cfg(feature = "std")]
pub mod os_heap;
