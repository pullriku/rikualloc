use core::{alloc::Layout, ptr::NonNull};

use crate::{allocator::MutAllocator, source::MemorySource};

pub struct BumpAllocator<S: MemorySource> {
    source: S,

    ptr: NonNull<u8>,
    end: NonNull<u8>,

    head: Option<NonNull<ChunkNode>>,
}

impl<S: MemorySource> MutAllocator for BumpAllocator<S> {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        // アラインメントに必要なバイト数のパディング
        // alignに合わせるためには何バイト必要か
        let align_offset = self.ptr.as_ptr().align_offset(layout.align());

        if align_offset == usize::MAX {
            return None;
        }
        // self.end.as_ptr()
        let alloc_start_ptr = unsafe { self.ptr.as_ptr().add(align_offset) };

        let alloc_end_ptr = unsafe { alloc_start_ptr.add(layout.size()) };

        if alloc_end_ptr <= self.end.as_ptr() {
            // バッファ内に空きがある
            self.ptr = unsafe { NonNull::new_unchecked(alloc_end_ptr) };

            let ptr = unsafe { NonNull::new_unchecked(alloc_start_ptr) };

            return Some(NonNull::slice_from_raw_parts(ptr, layout.size()));
        }

        // バッファ内に空きがない

        let head_layout = Layout::new::<ChunkNode>();
        let (request_layout, offset) = head_layout.extend(layout).ok()?;
        let final_layout = Layout::from_size_align(
            request_layout.size().max(4096),
            request_layout.align(),
        )
        .ok()?;

        let chunk_mem = unsafe { self.source.request_chunk(final_layout)? };

        let chunk_ptr = chunk_mem.cast::<u8>();
        let node_ptr = chunk_ptr.cast::<ChunkNode>();
        unsafe {
            node_ptr.write(ChunkNode {
                next: self.head,
                ptr: chunk_ptr,
                layout: final_layout,
            })
        };

        self.head = Some(node_ptr);
        let user_start_u8_ptr = unsafe {  chunk_ptr.add(offset) };
        let new_cursor_u8_ptr = unsafe { user_start_u8_ptr.add(layout.size()) };

        let chunk_len = chunk_mem.len();
        let chunk_end_u8_ptr = unsafe { chunk_ptr.add(chunk_len) };

        self.ptr = new_cursor_u8_ptr;
        self.end = chunk_end_u8_ptr;

        Some(NonNull::slice_from_raw_parts(
            user_start_u8_ptr,
            layout.size(),
        ))
    }

    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {}
}

impl<S: MemorySource> Drop for BumpAllocator<S> {
    fn drop(&mut self) {
        let mut current = self.head;

        while let Some(node_ptr) = current {
            unsafe {
                let node = node_ptr.read();
                current = node.next;

                self.source
                    .release_chunk(node.ptr.cast::<u8>(), node.layout);
            }
        }
    }
}

pub struct ChunkNode {
    next: Option<NonNull<ChunkNode>>,
    ptr: NonNull<u8>,
    layout: Layout,
}
