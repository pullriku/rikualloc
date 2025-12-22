#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use libc::printf;
use rikualloc::{
    allocator::bump::BumpAllocator, mutex::Locked,
    source::static_buff::StaticBuffer,
};

const BUFFER_SIZE: usize = 1024 * 1024;
static BUFFER: StaticBuffer<BUFFER_SIZE> = StaticBuffer::new();

#[global_allocator]
static BUMP: Locked<BumpAllocator<&StaticBuffer<BUFFER_SIZE>>> =
    Locked::new(BumpAllocator::new(&BUFFER));

fn main() {
    let v: Vec<usize> = (0..100000).filter(|x| x % 13 == 0).collect();

    unsafe {
        libc::printf(c"vec len=%zu\n".as_ptr() as *const _, v.len());
    }

    unsafe { printf(c"[".as_ptr()) };
    for x in v.iter() {
        unsafe {
            libc::printf(c"%u, ".as_ptr(), *x);
        }
    }
    unsafe { printf(c"]\n".as_ptr()) };
}
