use crate::source::MemorySource;

pub struct OsHeap;

impl MemorySource for OsHeap {
    unsafe fn request_chunk(&self, size: usize) -> Option<std::ptr::NonNull<[u8]>> {
        todo!()
    }
    unsafe fn release_chunk(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        todo!()
    }
}
