use core::{
    alloc::Layout,
    ptr::{self, NonNull},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{align::align_up, source::MemorySource};

pub struct OsHeap;

impl MemorySource for OsHeap {
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<core::ptr::NonNull<[u8]>> {
        let page_size = page_size();

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
        let alloc_size = align_up(layout.size(), page_size());

        unsafe {
            libc::munmap(ptr.as_ptr().cast::<libc::c_void>(), alloc_size);
        }
    }
}

static PAGE_SIZE: AtomicUsize = AtomicUsize::new(0);

fn page_size() -> usize {
    let page_size = PAGE_SIZE.load(Ordering::Relaxed);
    if page_size != 0 {
        return page_size;
    }
    let result = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    let page_size = if result > 0 { result as usize } else { 4096 };
    PAGE_SIZE.store(page_size, Ordering::Relaxed);
    page_size
}
