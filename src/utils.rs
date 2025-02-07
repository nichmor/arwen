/// Aligns the given size to 4 bytes.
pub fn align_to_arch(size: usize) -> usize {
    if cfg!(target_pointer_width = "32") {
        // ((size + 1 + 3) / 4) * 4
        size.next_multiple_of(4)
    } else if cfg!(target_pointer_width = "64") {
        // ((size + 1 + 3) / 4) * 4
        size.next_multiple_of(8)
    } else {
        // TODO: remove the pannic and return a Result
        panic!("Unsupported architecture")
    }
}
