use core::marker::PhantomData;

/// Helper struct representing a mutable buffer with a stored cursor position,
/// for writing sequential binary data.
pub struct Cursor<'a> {
    buf: *mut u8,
    cursor: usize,
    capacity: usize,
    phantom: PhantomData<&'a u8>,
}

impl<'a> Cursor<'a> {
    /// Constructs a new cursor from the provided buffer
    pub const fn new(buffer: &mut [u8]) -> Self {
        Self {
            buf: buffer.as_mut_ptr(),
            cursor: 0,
            capacity: buffer.len(),
            phantom: PhantomData {},
        }
    }

    /// Constructs a new cursor with an empty backing buffer
    pub const fn default() -> Self {
        Self {
            buf: core::ptr::null_mut(),
            cursor: 0,
            capacity: 0,
            phantom: PhantomData {},
        }
    }

    /// Tries to write a u8 to the buffer, returning `false` if end of buffer reached.
    pub const fn write_u8(&mut self, data: u8) -> bool {
        const U8_SIZE: usize = core::mem::size_of::<u8>();

        if self.cursor + U8_SIZE > self.capacity {
            return false;
        }

        unsafe {
            core::ptr::write(self.buf.add(self.cursor), data);
        }
        self.cursor += U8_SIZE;

        true
    }

    /// Tries to write a u16 to the buffer, returning `false` if end of buffer reached.
    pub const fn write_u16(&mut self, data: u16) -> bool {
        const U16_SIZE: usize = core::mem::size_of::<u16>();

        if self.cursor + U16_SIZE > self.capacity {
            return false;
        }

        unsafe {
            core::ptr::copy(
                data.to_ne_bytes().as_ptr(),
                self.buf.add(self.cursor),
                U16_SIZE,
            );
        }
        self.cursor += U16_SIZE;

        true
    }

    /// Tries to write a u32 to the buffer, returning `false` if end of buffer reached.
    pub const fn write_u32(&mut self, data: u32) -> bool {
        const U32_SIZE: usize = core::mem::size_of::<u32>();

        if self.cursor + U32_SIZE > self.capacity {
            return false;
        }

        unsafe {
            core::ptr::copy(
                data.to_ne_bytes().as_ptr(),
                self.buf.add(self.cursor),
                U32_SIZE,
            );
        }
        self.cursor += U32_SIZE;

        true
    }

    /// Tries to write a u64 to the buffer, returning `false` if end of buffer reached.
    pub const fn write_u64(&mut self, data: u64) -> bool {
        const U64_SIZE: usize = core::mem::size_of::<u64>();

        if self.cursor + U64_SIZE > self.capacity {
            return false;
        }

        unsafe {
            core::ptr::copy(
                data.to_ne_bytes().as_ptr(),
                self.buf.add(self.cursor),
                U64_SIZE,
            );
        }
        self.cursor += U64_SIZE;

        true
    }

    /// Tries to write a usize to the buffer, returning `false` if end of buffer reached.
    pub const fn write_usize(&mut self, data: usize) -> bool {
        const USIZE_SIZE: usize = core::mem::size_of::<usize>();

        if self.cursor + USIZE_SIZE > self.capacity {
            return false;
        }

        unsafe {
            core::ptr::copy(
                data.to_ne_bytes().as_ptr(),
                self.buf.add(self.cursor),
                USIZE_SIZE,
            );
        }
        self.cursor += USIZE_SIZE;

        true
    }

    /// Tries to write a slice to the buffer, returning `false` if end of buffer reached.
    pub const fn write_slice(&mut self, data: &[u8]) -> bool {
        if self.cursor + data.len() > self.capacity {
            return false;
        }

        unsafe {
            core::ptr::copy(data.as_ptr(), self.buf.add(self.cursor), data.len());
        }
        self.cursor += data.len();

        true
    }

    /// Returns the current position into the file (cursor).
    pub const fn position(&self) -> usize {
        self.cursor
    }

    /// Resets current position (cursor) to 0.
    pub const fn reset_position(&mut self) {
        self.cursor = 0;
    }

    /// Returns a pointer to the underlying buffer
    pub const fn as_ptr(&self) -> *const u8 {
        self.buf
    }
}
