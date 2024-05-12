use core::fmt;

use super::{ParseTag, Tag};

/// Stores information about VBE
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#VBE-info
#[derive(Clone, Copy)]
#[repr(C)]
pub struct VBEInfo {
    pub mode: u16,
    pub interface_seg: u16,
    pub interface_off: u16,
    pub interface_len: u16,
    pub control_info: [u8; 512],
    pub mode_info: [u8; 256],
}

impl ParseTag for VBEInfo {
    unsafe fn parse(addr: *const u32, _size: usize) -> Option<Tag> {
        /*       +-------------------+
        u16     | vbe_mode          |
        u16     | vbe_interface_seg |
        u16     | vbe_interface_off |
        u16     | vbe_interface_len |
        u8[512] | vbe_control_info  |
        u8[256] | vbe_mode_info     |
                +-------------------+ */

        Some(Tag::VBEInfo(*(addr as *const _)))
    }
}

impl fmt::Debug for VBEInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VBEInfo")
            .field("mode", &self.mode)
            .field("interface_seg", &self.interface_seg)
            .field("interface_off", &self.interface_off)
            .field("interface_len", &self.interface_len)
            .finish()
    }
}
