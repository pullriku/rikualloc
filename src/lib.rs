#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
extern crate alloc;

pub mod allocator;
pub mod mutex;
pub mod source;

mod align;
