use bitflags::bitflags;
use crabstd::cursor::Cursor;

use crate::HeaderTag;

bitflags! {
    /// Flags for OS console
    ///
    /// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Console-header-tags
    pub struct ConsoleFlags: u32 {
        /// OS image has support for EGA text mode
        const EGA_TEXT_SUPPORT = 1;
        /// At least one of the supported consoles must be present to boot
        const SUPPORTED_MUST_BE_PRESENT = 1 << 1;
    }
}

impl const HeaderTag for ConsoleFlags {
    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(4);
        buffer.write_u16(0);
        buffer.write_u32(12);

        buffer.write_u32(self.bits());
    }
}
