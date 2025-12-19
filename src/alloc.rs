use std::{alloc::Layout, ptr::NonNull};

pub trait MutAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>>;
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout);
}

pub mod bump;
pub mod free_list;
