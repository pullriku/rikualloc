use crate::{allocator::MutAllocator, mutex::Mutex};

pub struct LockedAllocator<A: MutAllocator> {
    inner: Mutex<A>,
}
