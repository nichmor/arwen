use core::panic;

use goblin::{
    container,
    mach::{
        header::{Header, SIZEOF_HEADER_32, SIZEOF_HEADER_64},
        parse_magic_and_ctx, MachO,
    },
};

pub struct MachoContainer<'a> {
    /// The parsed Mach-O file.
    pub inner: MachO<'a>,

    /// The raw bytes of the Mach-O file.
    pub data: &'a [u8],
    /// The context of the container.
    /// This is used to determine the architecture of the Mach-O file.
    pub ctx: container::Ctx,
}

impl<'a> MachoContainer<'a> {
    pub fn parse(bytes_of_file: &'a [u8]) -> Self {
        let parsed_macho = MachO::parse(bytes_of_file, 0).unwrap();

        let (_, maybe_ctx) = parse_magic_and_ctx(bytes_of_file, 0).unwrap();
        let ctx = if let Some(ctx) = maybe_ctx {
            ctx
        } else {
            panic!("Could not determine the architecture of the Mach-O file");
        };

        MachoContainer {
            inner: parsed_macho,
            data: bytes_of_file,
            ctx,
        }
    }
}

pub struct HeaderContainer {
    pub inner: Header,
    pub ctx: container::Ctx,
}

impl HeaderContainer {
    pub fn new(header: Header, ctx: container::Ctx) -> Self {
        HeaderContainer { inner: header, ctx }
    }

    pub fn size(&self) -> usize {
        if self.ctx.container.is_big() {
            SIZEOF_HEADER_64
        } else {
            SIZEOF_HEADER_32
        }
    }
}
