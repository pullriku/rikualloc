use core::ptr::NonNull;

use crate::{allocator::MutAllocator, source::MemorySource};

pub struct FreeList<S: MemorySource> {
    source: S,
    head: Option<NonNull<ListNode>>,
}

impl<S: MemorySource> MutAllocator for FreeList<S> {
    unsafe fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> Option<core::ptr::NonNull<[u8]>> {
        todo!()
    }

    unsafe fn dealloc(
        &mut self,
        ptr: core::ptr::NonNull<u8>,
        layout: core::alloc::Layout,
    ) {
        todo!()
    }
}

pub struct ListNode {
    size: usize,
    next: Option<NonNull<ListNode>>,
}
