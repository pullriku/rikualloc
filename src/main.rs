use std::{
    alloc::{Layout, alloc},
    hint,
};

use rikualloc::{
    allocator::{bump::BumpAllocator, free_list::FreeList},
    mutex::Locked,
    source::{os_heap::OsHeap, static_buff::StaticBuffer},
};

static BUFFER: StaticBuffer<{ 1024 * 1024 }> = StaticBuffer::new();

static BUMP: Locked<BumpAllocator<&StaticBuffer<{ 1024 * 1024 }>>> =
    Locked::new(BumpAllocator::new(&BUFFER));

#[global_allocator]
// static HEAP: Locked<FreeList<OsHeap>> = Locked::new(FreeList::new(OsHeap));
static HEAP: Locked<FreeList<&StaticBuffer<{ 1024 * 1024 }>>> =
    Locked::new(FreeList::new(&BUFFER));

fn main() {
    let vec: Vec<usize> = (0..10000).filter(|x| x % 13 == 0).collect();

    println!("{vec:?}");

    println!("{:?}", unsafe { alloc(Layout::new::<usize>()) });
    hint::black_box((0..10000).collect::<Vec<_>>());
    println!("{:?}", unsafe { alloc(Layout::new::<usize>()) });
}
