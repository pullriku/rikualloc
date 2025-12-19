use std::{cell::UnsafeCell, mem::MaybeUninit};

pub struct StaticBuffer<const N: usize> {
    buffer: UnsafeCell<[MaybeUninit<u8>; N]>,
    used: UnsafeCell<bool>,
}
