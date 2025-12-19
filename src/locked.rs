use std::sync::Mutex;

use crate::alloc::MutAllocator;

pub struct LockedAllocator<A: MutAllocator> {
    inner: Mutex<A>,
}
