use crabstd::cursor::Cursor;

use crate::{HeaderTag, HeaderTagValue};

/// Options for pixel-based framebuffer
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#The-framebuffer-tag-of-Multiboot2-header
pub struct Framebuffer {
    pub width: HeaderTagValue<u32>,
    pub height: HeaderTagValue<u32>,
    pub depth: HeaderTagValue<u32>,
}

impl const HeaderTag for Framebuffer {
    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(5);
        buffer.write_u16(0);
        buffer.write_u32(20);
        buffer.write_u32(self.width.unwrap_or_default(0));
        buffer.write_u32(self.height.unwrap_or_default(0));
        buffer.write_u32(self.depth.unwrap_or_default(0));
    }
}
