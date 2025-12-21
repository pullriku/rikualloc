#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
extern crate alloc;

pub mod allocator;
pub mod locked;
pub mod source;
pub mod mutex;
