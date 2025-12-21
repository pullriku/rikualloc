use core::{alloc::Layout, ptr::NonNull};

pub mod bump;
pub mod free_list;

pub trait MutAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>>;
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout);
}

impl<A: MutAllocator + ?Sized> MutAllocator for &mut A {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        unsafe { <A as MutAllocator>::alloc(&mut **self, layout) }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { <A as MutAllocator>::dealloc(&mut **self, ptr, layout) }
    }
}
