use goblin::container::Ctx;

/// Pad the given size to 4 bytes.
pub fn padding_size(size: usize) -> usize {
    size.next_multiple_of(4)
}

/// Aligns the given size to 4 bytes.
pub fn align_to_arch(size: usize, ctx: Ctx) -> usize {
    if ctx.container.is_big() {
        size.next_multiple_of(8)
    } else {
        size.next_multiple_of(4)
    }
}
