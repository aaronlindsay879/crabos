use core::ffi::CStr;

use super::{ParseTag, Tag};

/// Stores information about a module loaded alongside the kernel
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Modules
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Module {
    pub start: u32,
    pub end: u32,
    pub string: &'static CStr,
}

impl ParseTag for Module {
    unsafe fn parse(addr: *const u32, _size: usize) -> Option<Tag> {
        /*      +-------------------+
        u32     | mod_start         |
        u32     | mod_end           |
        u8[n]   | string            |
                +-------------------+ */

        let start = *addr;
        let end = *addr.add(1);
        let string = CStr::from_ptr(addr.add(2) as *const i8);

        Some(Tag::Module(Self { start, end, string }))
    }
}
