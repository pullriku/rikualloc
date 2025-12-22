use core::{alloc::Layout, ptr, ptr::NonNull};

use crate::{allocator::MutAllocator, source::MemorySource};

pub struct BumpAllocator<S: MemorySource> {
    source: S,

    ptr: NonNull<u8>,
    end: NonNull<u8>,

    head: Option<NonNull<ChunkNode>>,
}

unsafe impl<S: MemorySource + Send> Send for BumpAllocator<S> {}

impl<S: MemorySource> MutAllocator for BumpAllocator<S> {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        debug_assert!(self.ptr.as_ptr().addr() <= self.end.as_ptr().addr());

        if layout.size() == 0 {
            let ptr = ptr::without_provenance_mut::<u8>(layout.align());
            let nn = unsafe { NonNull::new_unchecked(ptr) };

            return Some(NonNull::slice_from_raw_parts(nn, 0));
        }

        let base = self.ptr.as_ptr();
        let end = self.end.as_ptr();

        let base_addr = base.addr();
        let end_addr = end.addr();

        // アラインメントに必要なバイト数のパディング
        // alignに合わせるためには何バイト必要か
        let pad = self.ptr.as_ptr().align_offset(layout.align());
        if pad == usize::MAX {
            return None;
        }

        let start_addr = base_addr.checked_add(pad)?;
        if start_addr >= end_addr {
            return self.new_chunk(layout);
        }

        let remaining = end_addr - start_addr;
        if remaining < layout.size() {
            return self.new_chunk(layout);
        }

        let alloc_start_ptr = unsafe { self.ptr.as_ptr().add(pad) };

        let alloc_end_ptr = unsafe { alloc_start_ptr.add(layout.size()) };

        // バッファ内に空きがある
        self.ptr = unsafe { NonNull::new_unchecked(alloc_end_ptr) };

        let ptr = unsafe { NonNull::new_unchecked(alloc_start_ptr) };

        Some(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {}
}

impl<S: MemorySource> BumpAllocator<S> {
    pub const fn new(source: S) -> Self {
        Self {
            source,
            ptr: NonNull::dangling(),
            end: NonNull::dangling(),
            head: None,
        }
    }
    /// バッファ内に空きがない場合、新しいチャンクを作成し、データ部のポインタを返す
    fn new_chunk(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        debug_assert!(self.ptr.as_ptr().addr() <= self.end.as_ptr().addr());

        let head_layout = Layout::new::<ChunkNode>();
        let (request_layout, header_size) = head_layout.extend(layout).ok()?;

        let request_layout = Layout::from_size_align(
            request_layout.size().max(4096), // 4096以上
            request_layout.align(),
        )
        .ok()?;

        let chunk_mem = unsafe { self.source.request_chunk(request_layout)? };

        debug_assert!(header_size + layout.size() <= chunk_mem.len());

        let need = header_size.checked_add(layout.size())?;
        if need > chunk_mem.len() {
            return None;
        }

        let actual_layout =
            Layout::from_size_align(chunk_mem.len(), request_layout.align())
                .ok()?;

        let chunk_ptr = chunk_mem.cast::<u8>();
        let node_ptr = chunk_ptr.cast::<ChunkNode>();
        unsafe {
            node_ptr.write(ChunkNode {
                next: self.head,
                ptr: chunk_ptr,
                layout: actual_layout,
            })
        };

        self.head = Some(node_ptr);
        let user_start_ptr = unsafe { chunk_ptr.add(header_size) };
        let new_cursor_ptr = unsafe { user_start_ptr.add(layout.size()) };

        let chunk_end_ptr = unsafe { chunk_ptr.add(chunk_mem.len()) };

        self.ptr = new_cursor_ptr;
        self.end = chunk_end_ptr;

        Some(NonNull::slice_from_raw_parts(user_start_ptr, layout.size()))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;
    use core::ptr::NonNull;
    use std::alloc::{alloc, dealloc};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::vec;
    use std::vec::Vec;

    #[derive(Default, Debug)]
    struct Stats {
        requested: usize,
        released: usize,
    }

    struct MockSource {
        stats: Rc<RefCell<Stats>>,
        /// 生きてるチャンク
        live: Vec<(NonNull<u8>, Layout)>,
    }

    impl MockSource {
        fn new(stats: Rc<RefCell<Stats>>) -> Self {
            Self {
                stats,
                live: vec![],
            }
        }
    }

    impl MemorySource for MockSource {
        unsafe fn request_chunk(
            &mut self,
            layout: Layout,
        ) -> Option<NonNull<[u8]>> {
            // Layout が無効だと alloc が UB
            if layout.size() == 0 {
                // 0サイズのチャンクは実用上いらないので拒否
                return None;
            }

            let ptr = unsafe { alloc(layout) };
            let nn = NonNull::new(ptr)?;
            self.live.push((nn, layout));
            self.stats.borrow_mut().requested += 1;

            Some(NonNull::slice_from_raw_parts(nn, layout.size()))
        }

        unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout) {
            // 該当チャンクを探して解放
            let idx = self
                .live
                .iter()
                .position(|(p, l)| {
                    p == &ptr
                        && l.size() == layout.size()
                        && l.align() == layout.align()
                })
                .expect("release_chunk called with unknown ptr/layout");

            let (p, l) = self.live.swap_remove(idx);
            unsafe { dealloc(p.as_ptr(), l) };
            self.stats.borrow_mut().released += 1;
        }
    }

    fn make_allocator_with_initial_chunk(
        stats: Rc<RefCell<Stats>>,
        chunk_size: usize,
    ) -> BumpAllocator<MockSource> {
        let mut source = MockSource::new(stats);
        let head_layout = Layout::new::<ChunkNode>();

        // 少なくとも ChunkNode を置けるサイズにする
        let size = chunk_size.max(head_layout.size());
        let layout =
            Layout::from_size_align(size, head_layout.align()).unwrap();

        let chunk_mem = unsafe { source.request_chunk(layout).unwrap() };
        let chunk_ptr = chunk_mem.cast::<u8>();
        let node_ptr = chunk_ptr.cast::<ChunkNode>();

        unsafe {
            node_ptr.write(ChunkNode {
                next: None,
                ptr: chunk_ptr,
                layout,
            });
        }

        let user_start = unsafe { chunk_ptr.add(head_layout.size()) };
        let chunk_end = unsafe { chunk_ptr.add(chunk_mem.len()) };

        BumpAllocator {
            source,
            ptr: user_start,
            end: chunk_end,
            head: Some(node_ptr),
        }
    }

    fn addr(p: NonNull<[u8]>) -> usize {
        // NonNull<[u8]> -> NonNull<u8> にしてアドレスを見る
        p.cast::<u8>().as_ptr().addr()
    }

    #[test]
    fn alloc_respects_alignment() {
        let stats = Rc::new(RefCell::new(Stats::default()));
        let mut a = make_allocator_with_initial_chunk(stats, 4096);

        let layout = Layout::from_size_align(24, 64).unwrap();
        let p = unsafe { a.alloc(layout).unwrap() };

        assert_eq!(addr(p) % 64, 0);
        assert_eq!(p.len(), 24);
    }

    #[test]
    fn bump_allocates_monotonically_when_it_fits() {
        let stats = Rc::new(RefCell::new(Stats::default()));
        let mut a = make_allocator_with_initial_chunk(stats, 4096);

        let l1 = Layout::from_size_align(16, 8).unwrap();
        let l2 = Layout::from_size_align(32, 8).unwrap();

        let p1 = unsafe { a.alloc(l1).unwrap() };
        let p2 = unsafe { a.alloc(l2).unwrap() };

        // bump なので、後の割当は前の割当より後ろになる
        // 同一チャンク内で収まる前提
        assert!(addr(p2) >= addr(p1) + p1.len());
    }

    #[test]
    fn alloc_grows_into_new_chunks_when_out_of_space() {
        let stats = Rc::new(RefCell::new(Stats::default()));
        let mut a = make_allocator_with_initial_chunk(stats.clone(), 128);

        // 128だとヘッダ + ちょっとで埋まるので、数回 alloc で増えるはず
        let l = Layout::from_size_align(80, 8).unwrap();
        let _ = unsafe { a.alloc(l).unwrap() };
        let _ = unsafe { a.alloc(l).unwrap() };

        let st = stats.borrow();
        assert!(
            st.requested >= 2,
            "should have requested at least 2 chunks, got {}",
            st.requested
        );
    }

    #[test]
    fn drop_releases_all_chunks() {
        let stats = Rc::new(RefCell::new(Stats::default()));

        {
            let mut a = make_allocator_with_initial_chunk(stats.clone(), 128);
            let l = Layout::from_size_align(80, 8).unwrap();

            // 複数チャンクを作る
            let _ = unsafe { a.alloc(l).unwrap() };
            let _ = unsafe { a.alloc(l).unwrap() };
            let _ = unsafe { a.alloc(l).unwrap() };
        } // drop で release_chunk が走る

        let st = stats.borrow();
        assert_eq!(
            st.released, st.requested,
            "all requested chunks should be released (requested={}, released={})",
            st.requested, st.released
        );
    }

    #[test]
    fn zst_allocation_returns_len_zero_slice() {
        let stats = Rc::new(RefCell::new(Stats::default()));
        let mut a = make_allocator_with_initial_chunk(stats, 4096);

        let l = Layout::from_size_align(0, 8).unwrap();
        let p = unsafe { a.alloc(l).unwrap() };
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn alloc_huge_object() {
        let stats = Rc::new(RefCell::new(Stats::default()));
        // 最初は小さいチャンクしか持ってないアロケーターを作る
        let mut a = make_allocator_with_initial_chunk(stats.clone(), 128);

        let huge_layout = Layout::from_size_align(10000, 16).unwrap();
        let p = unsafe { a.alloc(huge_layout).unwrap() };

        assert_eq!(p.len(), 10000);
        // ちゃんと確保できてること
        assert!(addr(p) > 0);

        let st = stats.borrow();
        assert!(st.requested >= 2);
    }

    #[test]
    fn alloc_fits_exact_remaining_space() {
        let stats = Rc::new(RefCell::new(Stats::default()));
        // わかりやすく、ユーザー領域がぴったり 64バイト ある状態を作る
        // (ヘッダサイズ + 64バイト)
        let head_size = Layout::new::<ChunkNode>().size();
        let total_size = head_size + 64;

        let mut a =
            make_allocator_with_initial_chunk(stats.clone(), total_size);

        // 32バイト確保 (残り32)
        let l32 = Layout::from_size_align(32, 1).unwrap();
        let _ = unsafe { a.alloc(l32).unwrap() };

        // 32バイト確保 (残り0 -> ジャストフィット)
        let _ = unsafe { a.alloc(l32).unwrap() };

        // ここまででチャンク追加は発生してないはず (requested == 1)
        assert_eq!(
            stats.borrow().requested,
            1,
            "Should fit exactly without new chunk"
        );

        let l1 = Layout::from_size_align(1, 1).unwrap();
        let _ = unsafe { a.alloc(l1).unwrap() };

        assert_eq!(
            stats.borrow().requested,
            2,
            "Should allocate new chunk now"
        );
    }

    #[test]
    fn zst_with_large_alignment() {
        let stats = Rc::new(RefCell::new(Stats::default()));
        let mut a = make_allocator_with_initial_chunk(stats, 4096);

        // サイズ0 だが アラインメント128
        let layout = Layout::from_size_align(0, 128).unwrap();
        let p = unsafe { a.alloc(layout).unwrap() };

        assert_eq!(p.len(), 0);
        assert_eq!(addr(p) % 128, 0);
    }
}
