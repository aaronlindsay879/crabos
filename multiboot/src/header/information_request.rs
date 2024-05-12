use crabstd::cursor::Cursor;

use crate::HeaderTag;

/// Requests that information about specific tags should be present in returned information table
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Information-request-header-tag
pub struct InformationRequest {
    pub requests: &'static [u32],
}

impl const HeaderTag for InformationRequest {
    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(1);
        buffer.write_u16(0);
        buffer.write_u32(8 + 4 * self.requests.len() as u32);

        let mut index = 0;
        while index < self.requests.len() {
            buffer.write_u32(self.requests[index]);
            index += 1;
        }
    }
}
