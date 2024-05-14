use multiboot::{MemoryMapEntry, MemoryType};

use super::super::{Frame, FrameAllocator};

/// Simple bump allocator for handing out free frames
pub struct BumpFrameAllocator {
    /// Next free frame
    next_free_frame: Frame,
    /// Memory area containing [Self::next_free_frame]
    current_area: Option<&'static MemoryMapEntry>,
    /// List of memory areas we can use
    areas: &'static [MemoryMapEntry],
    /// Frame containing start of kernel, used to avoid handing out kernel frames
    kernel_start: Frame,
    /// Frame containing end of kernel, used to avoid handing out kernel frames
    kernel_end: Frame,
    /// Frame containing start of multiboot info, used to avoid handing out multiboot frames
    multiboot_start: Frame,
    /// Frame containing end of multiboot info, used to avoid handing out multiboot frame
    multiboot_end: Frame,
}

impl FrameAllocator for BumpFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        // if area is none, then we're out of free frames
        let area = match self.current_area {
            Some(area) => area,
            None => return None,
        };

        // "clone" the frame to return if free
        let frame = unsafe { self.next_free_frame.clone() };

        // last frame in the current area
        let current_area_last_frame = {
            let address = area.base_addr + area.length - 1;
            Frame::containing_address(address as usize)
        };

        if frame > current_area_last_frame {
            // all frames in area used, choose a new area
            self.choose_next_area();
        } else if frame >= self.kernel_start && frame <= self.kernel_end {
            // frame lies within kernel space, move to end of kernel and try again
            self.next_free_frame = Frame {
                number: self.kernel_end.number + 1,
            }
        } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
            // frame lies within multiboot info, move to end of multiboot info and try again
            self.next_free_frame = Frame {
                number: self.multiboot_end.number + 1,
            }
        } else {
            // otherwise frame is unused, return and increment next_free_frame
            self.next_free_frame.number += 1;
            return Some(frame);
        }

        // try again if no valid frame found
        self.allocate_frame()
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        // do nothing, this is a simple bump allocator
    }
}

impl BumpFrameAllocator {
    /// Construcs a new area frame allocator
    pub fn new(
        kernel_start: usize,
        kernel_end: usize,
        multiboot_start: usize,
        multiboot_end: usize,
        memory_areas: &'static [MemoryMapEntry],
    ) -> Self {
        let mut allocator = Self {
            next_free_frame: Frame::containing_address(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        };
        allocator.choose_next_area();

        allocator
    }

    /// Chooses the next memory area to allocate from
    fn choose_next_area(&mut self) {
        // find the area with lowest base address that has frames free
        // (next_free_frame < end frame in area)
        self.current_area = self
            .areas
            .iter()
            .filter(|area| area.mem_type == MemoryType::RAM)
            .filter(|area| {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(address as usize) >= self.next_free_frame
            })
            .min_by_key(|area| area.base_addr);

        // if next_free_frame < start frame in area, make sure to update it
        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.base_addr as usize);

            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
