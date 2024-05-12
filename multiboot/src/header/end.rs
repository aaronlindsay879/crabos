use crabstd::cursor::Cursor;

use crate::HeaderTag;

/// End tag for header
pub struct End;

impl const HeaderTag for End {
    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(0);
        buffer.write_u16(0);
        buffer.write_u32(8);
    }
}
