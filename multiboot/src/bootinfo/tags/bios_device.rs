use super::{ParseTag, Tag};

/// Stores information about the BIOS boot device
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#BIOS-Boot-device
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BiosDevice {
    pub biosdev: u32,
    pub partition: u32,
    pub sub_partition: u32,
}

impl ParseTag for BiosDevice {
    unsafe fn parse(addr: *const u32, _size: usize) -> Option<Tag> {
        /*      +-------------------+
        u32     | biosdev           |
        u32     | partition         |
        u32     | sub_partition     |
                +-------------------+ */
        Some(Tag::BiosDevice(*(addr as *const _)))
    }
}
