use crate::{structures::PAGE_SIZE, PhysicalAddress};

/// Represents an individual frame of memory
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    /// Returns the frame that contains the specified address
    pub fn containing_address(address: usize) -> Self {
        Self {
            number: address / PAGE_SIZE,
        }
    }

    /// Returns the physical start address of the frame
    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    /// Private clone method since should not be a part of the public api
    pub fn clone(&self) -> Frame {
        Frame {
            number: self.number,
        }
    }

    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter { start, end }
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}
