/// Rounds `x` up to the nearest multiple of `align`.
pub(crate) const fn align_up(x: usize, align: usize) -> usize {
    (x + align - 1) & !(align - 1)
}
