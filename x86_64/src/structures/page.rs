use crate::VirtualAddress;

/// Size of a normal page in bytes
pub const PAGE_SIZE: usize = 0x1000;

/// Size of a huge L2 page in bytes
pub const HUGE_L2_PAGE_SIZE: usize = 0x200000;

/// Size of a huge L3 page in bytes
pub const HUGE_L3_PAGE_SIZE: usize = 0x40000000;

/// Similar to [crate::memory::Frame] but for virtual memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    pub number: usize,
}

impl Page {
    /// Returns the page that contains the specified address
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(
            !(0x0000_8000_0000_0000..0xFFFF_8000_0000_0000).contains(&address),
            "invalid address: 0x{:x}",
            address
        );

        Page {
            number: address / PAGE_SIZE,
        }
    }

    /// Returns the start address of the page
    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    /// Returns index into p4 table
    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    /// Returns index into p3 table
    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    /// Returns index into p2 table
    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    /// Returns index into p1 table
    #[allow(clippy::identity_op)]
    pub fn p1_index(&self) -> usize {
        // identity operation is allowed because it means this matches the pattern of other index functions
        (self.number >> 0) & 0o777
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter { start, end }
    }
}

pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}
