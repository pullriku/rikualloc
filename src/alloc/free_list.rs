use std::ptr::NonNull;

use crate::{alloc::MutAllocator, source::MemorySource};

pub struct FreeList<S: MemorySource> {
    source: S,
    head: Option<NonNull<ListNode>>,
}

impl<S: MemorySource> MutAllocator for FreeList<S> {
    unsafe fn alloc(&mut self, layout: std::alloc::Layout) -> Option<std::ptr::NonNull<[u8]>> {
        todo!()
    }

    unsafe fn dealloc(&mut self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        todo!()
    }
}

pub struct ListNode {
    size: usize,
    next: Option<NonNull<ListNode>>,
}
