use super::{ParseTag, Tag};

/// Stores basic memory information
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Basic-memory-information
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Memory {
    pub lower: u32,
    pub upper: u32,
}

impl ParseTag for Memory {
    unsafe fn parse(addr: *const u32, _size: usize) -> Option<Tag> {
        /*      +-------------------+
        u32     | mem_lower         |
        u32     | mem_upper         |
                +-------------------+ */
        Some(Tag::Memory(*(addr as *const _)))
    }
}
