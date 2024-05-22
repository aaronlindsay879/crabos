use core::ops::{Deref, DerefMut};

use x86_64::structures::Frame;

use super::{
    mapper::Mapper,
    table::{Level4, Table},
};

pub struct InactivePageTable {
    mapper: Mapper,
    frame: Frame,
}

impl InactivePageTable {
    /// Creates a new mapper in the given frame
    ///
    /// # Safety
    /// This should only ever be called with a valid frame
    pub unsafe fn new(frame: Frame) -> Self {
        core::ptr::write_bytes(frame.start_address() as *mut u64, 0, 512);
        let table = frame.start_address() as *mut Table<Level4>;

        Self {
            mapper: Mapper::new(table),
            frame,
        }
    }

    pub fn frame(self) -> Frame {
        self.frame
    }
}

impl Deref for InactivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}

impl DerefMut for InactivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}
