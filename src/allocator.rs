use core::{alloc::Layout, ptr::NonNull};

pub mod bump;
pub mod free_list;

/// `Layout` に基づいてメモリを確保/解放するための、低レベルなアロケータ。selfの可変参照を引数にとる。
///
/// # 契約
/// - `alloc` が返すポインタは `layout.align()` に従ってアラインされ、
///   少なくとも `layout.size()` バイト分の有効な領域を指していなければなりません。
/// - `alloc` が `Some` を返した場合、その領域は `dealloc` される（またはアロケータ自体が破棄される）まで、
///   他の確保に再利用されてはなりません。
/// - `dealloc` には、必ず対応する `alloc` 呼び出しで得た `ptr` と **同一の `layout`** を渡さなければなりません。
///
/// # スレッド安全性
/// このトレイト自体はスレッド安全性を保証しません。
/// 複数スレッドから同時に呼び出す場合は、外側で排他（Mutex 等）を行ってください。
pub trait MutAllocator {
    /// `layout` を満たすメモリ領域を確保し、確保した領域を `NonNull<[u8]>` として返します。
    ///
    /// 返されるスライスの長さは、`layout.size()`でなければなりません。
    /// `size`が0の場合はダングリングポインタを返します。
    ///
    /// # Safety
    /// - 呼び出し側は、返ってきた領域（`Some` の場合）が指すメモリについて、
    ///   そのアロケータの規約に従って使用しなければなりません。
    /// - 実装側は、返すポインタが `layout.align()` を満たし、少なくとも `layout.size()` バイト分が
    ///   有効であることを保証しなければなりません。
    /// - 実装側は、同じ領域を二重に返したり、解放前に別用途へ再利用したりしてはいけません。
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>>;

    /// `alloc` により確保されたメモリ領域を解放します。
    ///
    /// # Safety
    /// - `ptr` は同じアロケータの `alloc`が返したポインタでなければなりません。
    /// - `layout` は、その `ptr` を得たときに `alloc` に渡した同一の `Layout`でなければなりません。
    ///   （`size`/`align` のどちらも一致が必要）
    /// - すでに `dealloc` された領域を再度 `dealloc` してはいけません（二重解放は禁止）。
    /// - `dealloc` 呼び出し後、`ptr` が指していた領域へアクセスしてはいけません（use-after-free 禁止）。
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout);
}

impl<A: MutAllocator + ?Sized> MutAllocator for &mut A {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        unsafe { <A as MutAllocator>::alloc(&mut **self, layout) }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { <A as MutAllocator>::dealloc(&mut **self, ptr, layout) }
    }
}
