use std::{cell::UnsafeCell, mem::MaybeUninit};

use crate::source::MemorySource;

pub struct StaticBuffer<const N: usize> {
    buffer: UnsafeCell<[MaybeUninit<u8>; N]>,
    used: UnsafeCell<bool>,
}

impl<const N: usize> MemorySource for StaticBuffer<N> {
    unsafe fn request_chunk(&self, size: usize) -> Option<std::ptr::NonNull<[u8]>> {
        todo!()
    }

    unsafe fn release_chunk(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {}
}
