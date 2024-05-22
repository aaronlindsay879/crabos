use core::ptr::NonNull;

use x86_64::{
    structures::{Frame, Page, PAGE_SIZE},
    PhysicalAddress, VirtualAddress,
};

use super::{
    entry::EntryFlags,
    table::{Level4, Table},
};
use crate::memory::{frame_alloc::FrameAllocator, paging::ENTRY_COUNT};

pub struct Mapper {
    table: NonNull<Table<Level4>>,
}

impl Mapper {
    /// Creates a new mapper with the given page 4 table
    ///
    /// # Safety
    /// This should only ever be called with a valid table
    pub unsafe fn new(table: *mut Table<Level4>) -> Self {
        Self {
            table: NonNull::new_unchecked(table),
        }
    }

    /// Returns a reference to the in-use level 4 table
    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.table.as_ref() }
    }

    /// Returns a mutable reference to the in-use level 4 table
    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.table.as_mut() }
    }

    /// Translates a given virtual address to its physical address
    pub fn translate(&self, virt_addr: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virt_addr % PAGE_SIZE;

        self.translate_page(Page::containing_address(virt_addr))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    /// Finds the frame that a given page points to
    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                // 1GiB page?
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        let frame = Some(Frame {
                            number: start_frame.number
                                + page.p2_index() * ENTRY_COUNT
                                + page.p1_index(),
                        });
                        return frame;
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page?
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                            // address must be 2MiB aligned
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number: start_frame.number + page.p1_index(),
                            });
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(huge_page)
    }

    /// Maps a given page to a given frame, using the provided flags
    pub fn map_to<A: FrameAllocator>(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());

        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    /// Maps a given page to any available frame, using the provided flags
    pub fn map<A: FrameAllocator>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A) {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    /// Identity maps a given frame, using the provided flags
    pub fn identity_map<A: FrameAllocator>(
        &mut self,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }
}
