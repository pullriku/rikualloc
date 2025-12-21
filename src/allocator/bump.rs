use core::{alloc::Layout, ptr::NonNull};

use crate::{allocator::MutAllocator, source::MemorySource};

pub struct BumpAllocator<S: MemorySource> {
    source: S,

    ptr: NonNull<u8>,
    end: NonNull<u8>,

    head: Option<NonNull<ChunkNode>>,
}

impl<S: MemorySource> MutAllocator for BumpAllocator<S> {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        todo!()
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {}
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
