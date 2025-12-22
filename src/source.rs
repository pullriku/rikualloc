use core::{alloc::Layout, ptr::NonNull};

pub mod static_buff;

#[cfg(feature = "std")]
pub mod os_heap;

/// まとまったメモリ領域（チャンク）を供給/回収する。
/// アロケータ（例: bump allocator / free-list allocator）が内部で使うために、
/// 大きめのメモリ領域（チャンク）を確保して提供します。
///
/// # 契約
/// - `request_chunk(layout)` は、少なくとも `layout.size()` バイト分の領域を返し、
///   その先頭は `layout.align()` に従ってアラインされていなければなりません。
/// - `release_chunk` には、対応する `request_chunk` で得たポインタと同一の `layout`を渡さなければなりません。
///
/// # スレッド安全性
/// このトレイト自体はスレッド安全性を保証しません。
/// 複数スレッドから同時に呼び出す場合は、外側で排他してください。
pub trait MemorySource {
    /// `layout` を満たすチャンク（連続メモリ領域）を要求し、成功すればその領域を返します。
    /// 「要求より大きい領域」を返す可能性があります。
    ///
    /// # Safety
    /// - 実装側は、返すポインタが `layout.align()` を満たし、少なくとも `layout.size()` バイト分が
    ///   有効であることを保証しなければなりません。
    /// - 返した領域は `release_chunk` されるまで有効でなければなりません。
    unsafe fn request_chunk(&mut self, layout: Layout)
    -> Option<NonNull<[u8]>>;

    /// `request_chunk` により取得したチャンクを回収（解放）します。
    /// layoutの渡すサイズは、確保時に要求したサイズではなく、実際のサイズであることに注意してください。
    ///
    /// # Safety
    /// - `ptr` は **この同じ `MemorySource` の `request_chunk`** が返したポインタでなければなりません。
    /// - `layout` は、その `ptr` を得たときに `request_chunk` に渡した **同一の `Layout`** でなければなりません。
    /// - すでに解放したチャンクを再度 `release_chunk` してはいけません（二重解放は禁止）。
    /// - `release_chunk` 呼び出し後、`ptr` が指していた領域へアクセスしてはいけません。
    unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout);
}

impl<S: MemorySource + ?Sized> MemorySource for &mut S {
    unsafe fn request_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        unsafe { <S as MemorySource>::request_chunk(&mut **self, layout) }
    }

    unsafe fn release_chunk(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { <S as MemorySource>::release_chunk(&mut **self, ptr, layout) }
    }
}
