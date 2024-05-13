use super::{
    table::{Level1, Table},
    ActivePageTable, Page, VirtualAddress,
};
use crate::memory::{EntryFlags, Frame, FrameAllocator};

/// Helper temporary page struct for changing loaded pages
pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    /// Creates a new temporary page with the given allocator
    pub fn new<A: FrameAllocator>(page: Page, allocator: &mut A) -> Self {
        Self {
            page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page.
    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        assert!(
            active_table.translate_page(self.page).is_none(),
            "temporary page is already mapped"
        );

        active_table.map_to(self.page, frame, EntryFlags::WRITABLE, &mut self.allocator);
        self.page.start_address()
    }

    /// Unmaps the temporary page in the active table.
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        // dont want to free unused tables here since we can accidentally "steal" tables
        // from other allocators and overflow buffer
        active_table.unmap(self.page, &mut self.allocator, false);
    }

    /// Maps the temporary page to the given page table frame in the active
    /// table. Returns a reference to the now mapped table.
    pub fn map_table_frame(
        &mut self,
        frame: Frame,
        active_table: &mut ActivePageTable,
    ) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }
}

/// Minimal frame allocator that just hands out one of a few saved frames
struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
    /// Construct a new tiny allocator, allocating three frames in the process
    fn new<A: FrameAllocator>(allocator: &mut A) -> Self {
        let mut f = || allocator.allocate_frame();
        let frames = [f(), f(), f()];

        Self(frames)
    }
}

impl FrameAllocator for TinyAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        // find and return one of the saved frames
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        // find a frame that's been handed out and save the unused frame there
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("Tiny allocator can hold only 3 frames.");
    }
}
