use object::{
    build::elf::{Builder, Segment},
    elf,
};

/// A trait to extend the `object::Builder` struct with additional methods.
pub trait BuilderExt<'data> {
    /// Find and return the mutable reference to the segment containing the stack exec.
    ///
    /// This uses the `PT_GNU_STACK` program header to find the executable.
    fn gnu_stack_mut(&mut self) -> Option<&mut Segment<'data>>;

    /// Find the segment containing the stack exect.
    ///
    /// This uses the `PT_GNU_STACK` program header to find the executable.
    fn gnu_stack(&self) -> Option<&Segment<'data>>;

    /// Add a new `PT_GNU_STACK` segment
    /// with all segment permission
    fn add_gnu_stack(&mut self) -> &mut Segment<'data>;
}

impl<'data> BuilderExt<'data> for Builder<'data> {
    fn gnu_stack(&self) -> Option<&Segment<'data>> {
        let segment = self
            .segments
            .iter()
            .find(|segment| segment.p_type == elf::PT_GNU_STACK)?;

        Some(segment)
    }

    fn add_gnu_stack(&mut self) -> &mut Segment<'data> {
        // add a new program header entry
        let segment = self.segments.add();
        segment.p_type = elf::PT_GNU_STACK;
        // set elf::PF_X  to 0 as we are clearing the executable stack
        segment.p_flags = elf::PF_R | elf::PF_W | elf::PF_X;
        segment.p_align = 0x10;

        segment
    }

    fn gnu_stack_mut(&mut self) -> Option<&mut Segment<'data>> {
        let segment = self
            .segments
            .iter_mut()
            .find(|segment| segment.p_type == elf::PT_GNU_STACK)?;

        Some(segment)
    }
}
