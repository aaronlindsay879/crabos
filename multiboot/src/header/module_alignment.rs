use crabstd::cursor::Cursor;

use crate::HeaderTag;

/// Requires that modules are aligned to pages
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Module-alignment-tag
pub struct ModuleAlignment;

impl const HeaderTag for ModuleAlignment {
    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(6);
        buffer.write_u16(0);
        buffer.write_u32(8);
    }
}
