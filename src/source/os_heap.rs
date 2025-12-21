use crate::source::MemorySource;

pub struct OsHeap;

impl MemorySource for OsHeap {
    unsafe fn request_chunk(
        &self,
        size: usize,
    ) -> Option<core::ptr::NonNull<[u8]>> {
        todo!()
    }

    unsafe fn release_chunk(
        &self,
        ptr: core::ptr::NonNull<u8>,
        layout: core::alloc::Layout,
    ) {
        todo!()
    }
}
