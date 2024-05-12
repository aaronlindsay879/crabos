use crabstd::cursor::Cursor;

pub mod console_flags;
pub mod end;
pub mod framebuffer;
pub mod information_request;

pub use console_flags::ConsoleFlags;
pub use end::End;
pub use framebuffer::Framebuffer;
pub use information_request::InformationRequest;

/// Essentially an [Option] implementation with const unwrap method
#[derive(Copy, Clone)]
pub enum HeaderTagValue<T> {
    Value(T),
    NoPreference,
}

impl<T: Default + Copy> HeaderTagValue<T> {
    /// Returns either contained value or a provided default
    pub const fn unwrap_or_default(&self, default: T) -> T {
        match self {
            HeaderTagValue::Value(val) => *val,
            HeaderTagValue::NoPreference => default,
        }
    }
}

/// Helper struct for constructing a multiboot header with provided architecture and flags
#[repr(C)]
pub struct MultibootHeader {
    arch: u32,
    out: [u8; Self::SIZE as usize],
    out_cursor: Cursor<'static>,
    buffer: [u8; Self::SIZE as usize],
    buffer_cursor: Cursor<'static>,
}

impl MultibootHeader {
    /// Multiboot2 header magic value
    const MAGIC: u32 = 0xE85250D6;

    /// Slightly hacky workaround to dynamically sized flags - just write a 4096 long block of bytes, padded with 0s
    pub const SIZE: u32 = 4096;

    /// Constructs a new header **without** initialising pointers
    ///
    /// # Safety
    /// To ensure safe usage, [Self::set_cursors] must be called before any other methods
    pub const unsafe fn new() -> Self {
        Self {
            arch: 0,
            out: [0; Self::SIZE as usize],
            out_cursor: Cursor::default(),
            buffer: [0; Self::SIZE as usize],
            buffer_cursor: Cursor::default(),
        }
    }

    /// Sets the pointers within the cursors to point at the buffers
    pub const fn set_cursors(&mut self) -> &mut Self {
        self.out_cursor = Cursor::new(&mut self.out);
        self.buffer_cursor = Cursor::new(&mut self.buffer);

        self
    }

    /// Writes the start of multiboot header
    /// (actually only sets magic number and arch, size and checksum are set during [Self::as_bytes]
    pub const fn write_header(&mut self, arch: u32) -> &mut Self {
        // write initial header (magic, arch, and 0 for size/checksum)
        self.out_cursor.write_u32(Self::MAGIC);
        self.out_cursor.write_u32(arch);
        self.out_cursor.write_u32(0);
        self.out_cursor.write_u32(0);

        self.arch = arch;

        self
    }

    /// Writes a given tag to the multiboot header
    pub const fn write_tag(&mut self, tag: &impl ~const HeaderTag) -> &mut Self {
        // write the byte using saved buffer, resetting cursor position afterwards so buffer can be re-used
        tag.write_bytes(&mut self.buffer_cursor, &mut self.out_cursor);
        self.buffer_cursor.reset_position();

        self
    }

    /// Return the bytes representing multiboot header, setting the size and checksum fields
    pub const fn as_bytes(&mut self) -> [u8; Self::SIZE as usize] {
        let written = self.out_cursor.position() as u32;

        // write new size and checksum
        unsafe {
            core::ptr::copy_nonoverlapping(
                written.to_ne_bytes().as_ptr(),
                self.out.as_mut_ptr().add(8),
                4,
            );

            core::ptr::copy_nonoverlapping(
                ((0x100000000 - (Self::MAGIC + self.arch + written) as u64) as u32)
                    .to_ne_bytes()
                    .as_ptr(),
                self.out.as_mut_ptr().add(12),
                4,
            );
        }
        self.out
    }
}

/// Trait which represents a tag which can be written into the multiboot header
#[const_trait]
pub trait HeaderTag {
    /// Write the tag to an intermediary buffer
    fn write_to_buffer(&self, buffer: &mut Cursor);

    /// Write the tag to the output buffer, by initially writing to intermediary buffer,
    /// copying to output buffer, and performing alignment
    fn write_bytes(&self, buffer: &mut Cursor, out: &mut Cursor) {
        self.write_to_buffer(buffer);
        let end_tag = buffer.position();

        let slice = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), end_tag) };
        out.write_slice(slice);

        // align to 8 byte boundary, write 0s if needed
        let mut alignment = ((end_tag + 7) & !7) - end_tag;
        while alignment > 0 {
            out.write_u8(0);
            alignment -= 1;
        }
    }
}

/// Constructs a multiboot header with the given architecture and (optionally) tags.
///
/// Creates a static `HEADER` variable in the `.multiboot` section
#[macro_export]
macro_rules! multiboot_header {
    (arch: $arch:expr) => {
        #[no_mangle]
        #[used]
        #[link_section = ".multiboot"]
        static HEADER: [u8; multiboot::MultibootHeader::SIZE as usize] = unsafe {
            multiboot::MultibootHeader::new()
                .set_cursors()
                .write_header($arch)
                .write_tag(&multiboot::End)
                .as_bytes()
        };
    };
    (
        arch: $arch:expr,
        tags: [
            $( $tag:expr, )*
        ]
    ) => {
        #[no_mangle]
        #[used]
        #[link_section = ".multiboot"]
        static HEADER: [u8; multiboot::MultibootHeader::SIZE as usize] = unsafe {
            multiboot::MultibootHeader::new()
                .set_cursors()
                .write_header($arch)
                $(
                    .write_tag(&$tag)
                )*
                .write_tag(&multiboot::End)
                .as_bytes()
        };
    };
}
