use core::{
    alloc::Layout,
    ptr::{self, NonNull},
};

use crate::{align::align_up, source::MemorySource};

pub struct OsHeap {
    page_size: usize,
}

impl MemorySource for OsHeap {
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<core::ptr::NonNull<[u8]>> {
        let page_size = self.page_size;

        let alloc_size = align_up(layout.size(), page_size);

        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                alloc_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return None;
        }

        let slice_ptr =
            ptr::slice_from_raw_parts_mut(ptr.cast::<u8>(), alloc_size);

        NonNull::new(slice_ptr)
    }

    unsafe fn release_chunk(
        &mut self,
        ptr: core::ptr::NonNull<u8>,
        layout: core::alloc::Layout,
    ) {
        let alloc_size = align_up(layout.size(), self.page_size);

        unsafe {
            libc::munmap(ptr.as_ptr().cast::<libc::c_void>(), alloc_size);
        }
    }
}

impl OsHeap {
    pub fn new() -> Self {
        Self {
            page_size: get_page_size(),
        }
    }
}

impl Default for OsHeap {
    fn default() -> Self {
        Self::new()
    }
}

fn get_page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize }
}
