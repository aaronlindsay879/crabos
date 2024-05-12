pub struct Cursor {
    buf: *mut u8,
    cursor: usize,
}

impl Cursor {
    pub const fn new(buffer: &mut [u8]) -> Self {
        Self {
            buf: buffer.as_mut_ptr(),
            cursor: 0,
        }
    }

    pub const fn default() -> Self {
        Self {
            buf: core::ptr::null_mut(),
            cursor: 0,
        }
    }

    pub const fn write_u8(&mut self, data: u8) {
        unsafe {
            core::ptr::write(self.buf.add(self.cursor), data);
        }
        self.cursor += core::mem::size_of::<u8>();
    }

    pub const fn write_u16(&mut self, data: u16) {
        const U16_SIZE: usize = core::mem::size_of::<u16>();

        unsafe {
            core::ptr::copy(
                data.to_ne_bytes().as_ptr(),
                self.buf.add(self.cursor),
                U16_SIZE,
            );
        }
        self.cursor += U16_SIZE;
    }

    pub const fn write_u32(&mut self, data: u32) {
        const U32_SIZE: usize = core::mem::size_of::<u32>();

        unsafe {
            core::ptr::copy(
                data.to_ne_bytes().as_ptr(),
                self.buf.add(self.cursor),
                U32_SIZE,
            );
        }
        self.cursor += U32_SIZE;
    }

    pub const fn write_u64(&mut self, data: u64) {
        const U64_SIZE: usize = core::mem::size_of::<u64>();

        unsafe {
            core::ptr::copy(
                data.to_ne_bytes().as_ptr(),
                self.buf.add(self.cursor),
                U64_SIZE,
            );
        }
        self.cursor += U64_SIZE;
    }

    pub const fn write_usize(&mut self, data: usize) {
        const USIZE_SIZE: usize = core::mem::size_of::<usize>();

        unsafe {
            core::ptr::copy(
                data.to_ne_bytes().as_ptr(),
                self.buf.add(self.cursor),
                USIZE_SIZE,
            );
        }
        self.cursor += USIZE_SIZE;
    }

    pub const fn write_slice(&mut self, data: &[u8]) {
        unsafe {
            core::ptr::copy(data.as_ptr(), self.buf.add(self.cursor), data.len());
        }
        self.cursor += data.len();
    }

    pub const fn position(&self) -> usize {
        self.cursor
    }

    pub const fn reset_position(&mut self) {
        self.cursor = 0;
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.buf
    }
}
