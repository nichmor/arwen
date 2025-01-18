/// Aligns the given size to 4 bytes.
pub fn align_to_4(size: usize) -> usize {
    ((size + 1 + 3) / 4) * 4
}
