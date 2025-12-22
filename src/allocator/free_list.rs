#![allow(deprecated)]

use core::{
    alloc::Layout,
    mem,
    ptr::{self, NonNull},
};

use crate::{allocator::MutAllocator, source::MemorySource};

#[deprecated(note = "雑に作った")]
pub struct FreeList<S: MemorySource> {
    source: S,
    head: Option<NonNull<ListNode>>,
}

#[repr(C)]
pub struct ListNode {
    size: usize, // このノードを含む空き領域の総バイト数
    next: Option<NonNull<ListNode>>,
}

// Send は source が Send のときだけに絞るのが筋
unsafe impl<S: MemorySource + Send> Send for FreeList<S> {}

impl<S: MemorySource> FreeList<S> {
    pub const fn new(source: S) -> Self {
        Self { source, head: None }
    }

    #[inline]
    fn node_layout() -> Layout {
        Layout::new::<ListNode>()
    }

    /// FreeList が内部的に使う「確保単位」を Layout で正規化する
    /// - align は ListNode の align 以上
    /// - size は ListNode の size 以上
    /// - size は align の倍数に丸める
    #[inline]
    fn normalized(layout: Layout) -> Option<Layout> {
        let node = Self::node_layout();
        let align = layout.align().max(node.align());
        let size = layout.size().max(node.size());

        let l = Layout::from_size_align(size, align).ok()?;
        Some(l.pad_to_align())
    }

    unsafe fn push_free(&mut self, start: NonNull<u8>, size: usize) {
        debug_assert!(size >= Self::node_layout().size());
        debug_assert!(
            start.as_ptr().align_offset(Self::node_layout().align()) == 0
        );

        let node_ptr = start.as_ptr() as *mut ListNode;
        unsafe {
            node_ptr.write(ListNode {
                size,
                next: self.head,
            })
        };
        self.head = Some(unsafe { NonNull::new_unchecked(node_ptr) });
    }

    /// prevを使って、nodeをfree listから外す
    unsafe fn unlink(
        &mut self,
        prev: Option<NonNull<ListNode>>,
        node: NonNull<ListNode>,
    ) -> ListNode {
        let v = unsafe { node.read() };
        match prev {
            None => self.head = v.next,
            Some(mut p) => unsafe { p.as_mut().next = v.next },
        }
        v
    }

    /// 空き領域 node から first-fit で割当を試す。
    /// 成功なら (alloc_ptr, alloc_size, prefix_size, suffix_size, next) を返す。
    fn try_take_from(node: NonNull<ListNode>, need: Layout) -> Option<Fit> {
        let node_ref = unsafe { node.as_ref() };

        let hole_start = node.as_ptr().cast::<u8>();
        let hole_start_addr = hole_start as usize;

        let hole_size = node_ref.size;
        let hole_end_addr = hole_start_addr.checked_add(hole_size)?;

        // アライン調整（ptr::align_offset を使う）
        let pad = hole_start.align_offset(need.align());
        if pad == usize::MAX {
            return None;
        }

        let alloc_start_addr = hole_start_addr.checked_add(pad)?;
        let alloc_end_addr = alloc_start_addr.checked_add(need.size())?;

        if alloc_end_addr > hole_end_addr {
            return None;
        }

        let prefix = alloc_start_addr - hole_start_addr;
        let suffix = hole_end_addr - alloc_end_addr;

        // 「結合しない」ので、残りを free list に戻すには ListNode を置ける必要がある
        let min_free = mem::size_of::<ListNode>();
        let prefix_ok = prefix == 0 || prefix >= min_free;
        let suffix_ok = suffix == 0 || suffix >= min_free;

        if !prefix_ok || !suffix_ok {
            return None;
        }

        let alloc_ptr = NonNull::new(alloc_start_addr as *mut u8)?;
        Some(Fit {
            alloc_ptr,
            alloc_size: need.size(),
            prefix_size: prefix,
            suffix_size: suffix,
        })
    }

    /// source から新チャンクを取って free list に追加
    unsafe fn grow(&mut self, need: Layout) -> Option<()> {
        // source に対して最低限の要求（大きめに取る）
        let request =
            Layout::from_size_align(need.size().max(4096), need.align())
                .ok()?;
        let chunk = unsafe { self.source.request_chunk(request) }?;

        // 返ってきた len を node_align で切り下げ（ノードを書けるように）
        let node_align = Self::node_layout().align();
        let usable = chunk.len() & !(node_align - 1);

        if usable < Self::node_layout().size() {
            return None;
        }

        let start = chunk.cast::<u8>();
        unsafe { self.push_free(start, usable) };
        Some(())
    }
}

impl<S: MemorySource> MutAllocator for FreeList<S> {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        // ZST は適当な non-null を返す
        if layout.size() == 0 {
            let p = ptr::without_provenance_mut::<u8>(layout.align());
            let nn = unsafe { NonNull::new_unchecked(p) };
            return Some(NonNull::slice_from_raw_parts(nn, 0));
        }

        let need = Self::normalized(layout)?;

        loop {
            // first-fit
            let mut prev: Option<NonNull<ListNode>> = None;
            let mut cur = self.head;

            while let Some(node) = cur {
                // if let Some((alloc_ptr, alloc_size, prefix, suffix, _next)) =
                if let Some(Fit {
                    alloc_ptr,
                    alloc_size,
                    prefix_size: prefix,
                    suffix_size: suffix,
                }) = Self::try_take_from(node, need)
                {
                    // node を list から外す
                    let _old = unsafe { self.unlink(prev, node) };

                    let hole_start = node.as_ptr().cast::<u8>() as usize;
                    let alloc_start = alloc_ptr.as_ptr() as usize;
                    let alloc_end = alloc_start + alloc_size;

                    // suffix を head に戻す
                    if suffix != 0 {
                        let suffix_ptr = unsafe {
                            NonNull::new_unchecked(alloc_end as *mut u8)
                        };
                        unsafe { self.push_free(suffix_ptr, suffix) };
                    }

                    // prefix を head に戻す
                    if prefix != 0 {
                        let prefix_ptr = unsafe {
                            NonNull::new_unchecked(hole_start as *mut u8)
                        };
                        unsafe { self.push_free(prefix_ptr, prefix) };
                    }

                    // alloc で返す長さは「要求サイズ」
                    return Some(NonNull::slice_from_raw_parts(
                        alloc_ptr,
                        layout.size(),
                    ));
                }

                prev = cur;
                cur = unsafe { node.as_ref().next };
            }

            // 見つからない → grow して再試行
            unsafe {
                self.grow(need)?;
            }
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() == 0 {
            return;
        }

        // alloc と同じ正規化サイズで free に戻す（結合なし、push するだけ）
        let need = match Self::normalized(layout) {
            Some(x) => x,
            None => return,
        };

        // ここが成立するのは alloc が need.align() で返すから
        debug_assert!(
            ptr.as_ptr().align_offset(Self::node_layout().align()) == 0
        );

        unsafe {
            self.push_free(ptr, need.size());
        }
    }
}

#[derive(Clone, Copy)]
struct Fit {
    /// ユーザに返す先頭。（内部サイズぶん確保）
    alloc_ptr: NonNull<u8>,
    /// 内部的に消費するサイズ。
    alloc_size: usize,
    /// 前に残る空き。
    prefix_size: usize,
    /// 後ろに残る空き。
    suffix_size: usize,
}
