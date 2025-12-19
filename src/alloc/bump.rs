use std::{alloc::Layout, ptr::NonNull};

use crate::{alloc::MutAllocator, source::MemorySource};

pub struct BumpAllocator<S: MemorySource> {
    source: S,
    current_chunk: Option<ChunkNode>,
}

impl<S: MemorySource> MutAllocator for BumpAllocator<S> {
    unsafe fn alloc(&mut self, layout: std::alloc::Layout) -> Option<NonNull<[u8]>> {
        todo!()
    }

    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: std::alloc::Layout) {}
}

impl<S: MemorySource> Drop for BumpAllocator<S> {
    fn drop(&mut self) {
        todo!()
    }
}

pub struct ChunkNode {
    next: Option<NonNull<ChunkNode>>,
    ptr: NonNull<u8>,
    layout: Layout,
}
