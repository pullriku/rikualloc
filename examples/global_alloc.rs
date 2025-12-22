#![allow(unused)]

use std::{
    alloc::{Layout, alloc},
    hint,
};

#[allow(unused_imports)]
use rikualloc::{
    allocator::{bump::BumpAllocator, free_list::FreeList},
    mutex::Locked,
    source::{os_heap::OsHeap, static_buff::StaticBuffer},
};

// static BUFFER: StaticBuffer<{ 1024 * 1024 }> = StaticBuffer::new();

// #[global_allocator]
// static BUMP: Locked<BumpAllocator<&StaticBuffer<{ 1024 * 1024 }>>> =
//     Locked::new(BumpAllocator::new(&BUFFER));

#[global_allocator]
static BUMP: Locked<BumpAllocator<OsHeap>> =
    Locked::new(BumpAllocator::new(OsHeap));

fn main() {
    println!("global allocator demo start");

    // --- 1) Box ---
    let b = Box::new(1234_u64);
    hint::black_box(*b);

    // --- 2) Vec: 再確保が起きるように push ---
    let mut v: Vec<u32> = Vec::new();
    let mut last_cap = v.capacity();
    println!("Vec: initial cap={}", last_cap);

    for i in 0..10_000u32 {
        v.push(i);
        let cap = v.capacity();
        if cap != last_cap {
            // realloc タイミングが見える（FreeList/Bump の挙動確認に便利）
            println!("Vec: cap grew {} -> {} (len={})", last_cap, cap, v.len());
            last_cap = cap;
        }
    }

    for i in 0..10_000u32 {
        v.push(i);
    }
    println!("after grow: len={} cap={}", v.len(), v.capacity());

    // --- shrink phase ---
    v.truncate(100);
    println!("after truncate: len={} cap={}", v.len(), v.capacity());

    v.shrink_to_fit();
    println!("after shrink:   len={} cap={}", v.len(), v.capacity());

    // --- regrow phase ---
    for i in 0..10_000u32 {
        v.push(i);
    }
    println!("after regrow: cap={}", v.capacity());

    hint::black_box(v.as_ptr());
    hint::black_box(v.len());

    // --- 3) String ---
    let mut s = String::new();
    for _ in 0..10_000 {
        s.push_str("abc");
    }
    hint::black_box(s.len());

    // --- 4) 手動 alloc: Layout を直接叩く（unsafeだけど検証に便利） ---
    unsafe {
        let l = Layout::from_size_align(1024, 64).unwrap();
        let p = alloc(l);
        assert!(!p.is_null(), "alloc returned null");
        assert_eq!((p as usize) % 64, 0, "alignment broken");
        // 触ってみる（最適化防止）
        core::ptr::write_bytes(p, 0xA5, 1024);
        hint::black_box(p);
    }

    // --- 5) Drop が動くか（スコープ終了でまとめて解放される） ---
    drop(s);
    drop(v);
    drop(b);

    println!("global allocator demo done");
}
