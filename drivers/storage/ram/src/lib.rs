#![no_std]

use crabstd::fs::StorageDevice;

pub struct Ram {
    virt_mask: usize,
}

impl Ram {
    pub fn new(virt_mask: usize) -> Self {
        Self { virt_mask }
    }
}

impl StorageDevice for Ram {
    fn read(&mut self, start: usize, count: usize, buf: &mut [u8]) -> usize {
        let count = count.min(buf.len());
        unsafe {
            core::ptr::copy(
                (start | self.virt_mask) as *const u8,
                buf.as_mut_ptr(),
                count,
            );
        }

        count
    }
}
