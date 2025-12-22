use core::alloc::Layout;
use core::mem;
use core::ptr::NonNull;

use crate::{allocator::MutAllocator, source::MemorySource};

pub struct FreeList<S: MemorySource> {
    source: S,
    head: Option<NonNull<ListNode>>,
}

pub struct ListNode {
    size: usize,
    next: Option<NonNull<ListNode>>,
}

unsafe impl<S: MemorySource + Send> Send for FreeList<S>{}

impl<S: MemorySource> FreeList<S> {
    pub const fn new(source: S) -> Self {
        Self {
            source,
            head: None,
        }
    }

    /// 指定されたポインタとサイズで新しいListNodeを作成し、リストの先頭に追加する
    unsafe fn add_free_region(&mut self, ptr: NonNull<u8>, size: usize) {
        // 確保領域がListNodeを格納できるか確認（念の為）
        if size < mem::size_of::<ListNode>() {
            return; 
        }

        // ポインタをListNode型にキャスト
        let mut node_ptr = ptr.cast::<ListNode>();
        
        // ListNodeを書き込む
        unsafe { node_ptr.as_mut().size = size };
        unsafe { node_ptr.as_mut().next = self.head };

        // headを更新
        self.head = Some(node_ptr);
    }
}

impl<S: MemorySource> MutAllocator for FreeList<S> {
    unsafe fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> Option<core::ptr::NonNull<[u8]>> {
        // 1. 要求サイズを調整（ListNodeが入るサイズ以上にする）
        let min_size = mem::size_of::<ListNode>();
        let size = layout.size().max(min_size);
        let align = layout.align();

        // 2. Free Listを走査して適切なブロックを探す (First-Fit)
        let mut prev_link = &mut self.head;

        while let Some(mut node_ptr) = *prev_link {
            let node = unsafe { node_ptr.as_mut() };

            // アライメントチェック
            // (単純化のため、ブロックの先頭アドレスがアライメントを満たすかだけ確認します)
            let addr = node_ptr.as_ptr() as usize;
            let is_aligned = addr.is_multiple_of(align);
            
            if is_aligned && node.size >= size {
                // --- 確保成功 ---
                
                // リストからこのノードを取り除く
                *prev_link = node.next;

                // 残りの領域がListNodeを作れるほど大きければ分割(Split)する
                let remaining_size = node.size - size;
                if remaining_size >= min_size {
                    // 新しいノードの開始位置を計算
                    let next_ptr = unsafe { (node_ptr.as_ptr() as *mut u8).add(size) };
                    let next_node_ptr = unsafe { NonNull::new_unchecked(next_ptr) };
                    
                    // リスト（今回は先頭）に戻す
                    // ※最適化するなら prev_link の位置に挿入したほうが断片化しにくいですが、
                    //   実装を簡単にするため add_free_region を使って先頭に戻します。
                    unsafe { self.add_free_region(next_node_ptr, remaining_size) };
                }

                // スライスへのポインタを作成して返す
                let ptr = unsafe { NonNull::new_unchecked(node_ptr.as_ptr() as *mut u8) };
                return Some(NonNull::slice_from_raw_parts(ptr, size));
            }

            // 次のノードへ進む
            prev_link = unsafe { &mut node_ptr.as_mut().next };
        }

        // 3. リストに見つからなかった場合、Sourceから新しいチャンクをもらう
        //    ここでは単純に要求サイズ分（あるいはもっと大きな固定サイズ）を要求します。
        //    効率化のため、通常は4KBなどのページ単位で要求するのが一般的です。
        let request_size = size.max(4096); // 例: 最低4KB要求する
        // アライメント要件を満たすレイアウトを作成
        let request_layout = Layout::from_size_align(request_size, align).ok()?;
        
        if let Some(chunk) = unsafe { self.source.request_chunk(request_layout) } {
            let chunk_ptr = chunk.cast();
            let chunk_len = chunk.len();

            // もらったチャンクを使って再帰的に確保、あるいは手動で分割
            // ここではもらったチャンクを一度FreeListに入れてから、再度allocを呼ぶ形で実装します
            unsafe { self.add_free_region(chunk_ptr, chunk_len) };
            
            // 再帰呼び出し（無限ループ防止のため、実際は回数制限などを入れると良い）
            return unsafe { self.alloc(layout) };
        }

        None
    }

    unsafe fn dealloc(
        &mut self,
        ptr: core::ptr::NonNull<u8>,
        layout: core::alloc::Layout,
    ) {
        // 解放された領域をFree Listの先頭に追加するだけ
        // (マージ処理/Coalescing は含んでいません)
        
        // 実際に管理していたサイズを計算（alloc時に調整したサイズ）
        let min_size = mem::size_of::<ListNode>();
        let size = layout.size().max(min_size);

        unsafe { self.add_free_region(ptr, size) };
    }
}
