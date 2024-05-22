use multiboot::{MemoryMapEntry, MemoryType};
use x86_64::{
    align_down_to_page, align_up,
    structures::{Frame, PAGE_SIZE},
};

use super::FrameAllocator;

/// Length of a bitmap array that fills one page
const BITMAP_LENGTH: usize = PAGE_SIZE / core::mem::size_of::<u64>();

/// Number of frames that each bitmap can store information about
const FRAMES_PER_BITMAP: usize = PAGE_SIZE * 8;

/// Frame allocator that works by storing a list of bits, with each one correlating to whether a specific frame is currently mapped.
pub struct BitmapFrameAllocator {
    memory_regions: &'static [MemoryMapEntry],
    bitmaps: &'static mut [u64],
}

impl BitmapFrameAllocator {
    pub fn frames_needed(memory_regions: &'static [MemoryMapEntry]) -> usize {
        memory_regions
            .iter()
            .filter(|region| region.mem_type == MemoryType::RAM)
            .map(|region| (region.length as usize).div_ceil(PAGE_SIZE * FRAMES_PER_BITMAP))
            .sum()
    }

    /// Creates a bitmap allocator from a given address, assuming it's already initialised
    ///
    /// # Safety
    /// `address` must point to a valid bitmap array
    pub unsafe fn from_address(memory_regions: &'static [MemoryMapEntry], address: usize) -> Self {
        let frames_needed = Self::frames_needed(memory_regions);

        let bitmaps: &mut [u64] =
            core::slice::from_raw_parts_mut(address as *mut u64, frames_needed * BITMAP_LENGTH);

        Self {
            memory_regions,
            bitmaps,
        }
    }

    /// Returns a new bitmap allocator, and the range of frames that need to be identity mapped
    pub fn new(
        start_addr: usize,
        memory_regions: &'static [MemoryMapEntry],
    ) -> (Self, (Frame, Frame)) {
        // first figure out how many frames we need for allocator, with at least one frame per region
        let frames_needed = Self::frames_needed(memory_regions);

        // now we "steal" that many frames starting from `start_addr`
        let bitmaps: &mut [u64] = unsafe {
            // zero memory and return a slice for it
            core::ptr::write_bytes(start_addr as *mut u64, 0, frames_needed * BITMAP_LENGTH);
            core::slice::from_raw_parts_mut(start_addr as *mut u64, frames_needed * BITMAP_LENGTH)
        };

        // find start and end frames where allocator will store data
        let frame_start = Frame::containing_address(start_addr);
        let frame_end = Frame {
            number: frame_start.number + frames_needed,
        };

        let mut bitmap_alloc = Self {
            memory_regions,
            bitmaps,
        };

        bitmap_alloc.set_ignored_frames(&frame_start, &frame_end);

        // now we need to mask out all
        let mut bitmap_index = 0;
        for region in bitmap_alloc.ram_regions() {
            // number of frames in memory region
            let frames = (region.length as usize).div_ceil(PAGE_SIZE);

            for index in 0..frames {
                if (index + 1) * 64 >= frames && index * 64 < frames {
                    // first frame that's not completely mapped - partially mask out first, and then fully mask remaining
                    // now we just need to mask all invalid frames to make sure they're not allocated
                    let valid_frames = frames - index * 64;
                    let mask = generate_mask(valid_frames, 64);

                    bitmap_alloc.bitmaps[bitmap_index + index] |= mask;

                    for bitmap in bitmap_alloc.bitmaps
                        [bitmap_index + index + 1..align_up(bitmap_index + index, BITMAP_LENGTH)]
                        .iter_mut()
                    {
                        *bitmap = !0;
                    }
                }
            }

            bitmap_index += align_up(frames / 64, BITMAP_LENGTH);
        }

        (bitmap_alloc, (frame_start, frame_end))
    }

    /// Set a region of memory as ignored by the allocator, so it can't allocate frames from that region
    pub fn set_ignored_frames(&mut self, frame_start: &Frame, frame_end: &Frame) {
        self.set_ignored_area(frame_start.start_address(), frame_end.start_address())
    }

    /// Set a region of memory as ignored by the allocator, so it can't allocate frames from that region,
    /// assuming `addr_start` and `addr_end` lie in the same memory map region
    pub fn set_ignored_area(&mut self, addr_start: usize, addr_end: usize) {
        // first make sure addresses are aligned to page boundaries
        let addr_start = align_down_to_page(addr_start);
        let addr_end = align_down_to_page(addr_end);

        // keep track of frames skipped while finding correct memory region
        let mut frames_skipped = 0;
        for region in self.ram_regions() {
            // skip regions until we find where address lies
            if addr_start > region.base_addr as usize + region.length as usize {
                // make sure we skip a multiple of `BITMAP_LENGTH` frames since each region starts
                // in a new frame
                frames_skipped +=
                    align_up((region.length as usize).div_ceil(PAGE_SIZE), BITMAP_LENGTH);
                continue;
            }

            // we now know we're in the right memory region to set as ignored

            // find addresses within the current region
            let addr_start = addr_start - region.base_addr as usize;
            let addr_end = addr_end - region.base_addr as usize;

            // shift to get index into bitmap array
            let addr_start_index = addr_start >> 18;
            let addr_end_index = addr_end >> 18;

            // start and end bits for first and last element of bitmap array we're setting
            let bit_start_index = (addr_start % 0x40000) >> 12;
            let bit_end_index = ((addr_end % 0x40000) >> 12) + 1;

            if bit_start_index == 0 && bit_end_index == 64 && addr_start_index == addr_end_index {
                // if all bits need to be set in one index, just do that
                self.bitmaps[frames_skipped + addr_start_index] = !0;
            } else if addr_start_index == addr_end_index {
                // otherwise set some bits in one index
                let mask = generate_mask(bit_start_index, bit_end_index);

                self.bitmaps[frames_skipped + addr_start_index] |= mask as u64;
            } else {
                // if we need to set multiple u64s in bitmap array, handle start and end cases separately
                let mask_a = generate_mask(bit_start_index, 64);
                self.bitmaps[frames_skipped + addr_start_index] |= mask_a as u64;

                let mask_b = generate_mask(0, bit_end_index);
                self.bitmaps[frames_skipped + addr_end_index] |= mask_b as u64;
            }

            // then handle between the two ranges
            if addr_end_index - addr_start_index >= 2 {
                let start_range = frames_skipped + addr_start_index + 1;
                let end_range = frames_skipped + addr_end_index;

                for bitmap in self.bitmaps[start_range..end_range].iter_mut() {
                    *bitmap = !0;
                }
            }

            break;
        }
    }

    /// Returns an iterator of all memory regions which are actually RAM
    fn ram_regions(&self) -> impl Iterator<Item = &'static multiboot::MemoryMapEntry> {
        self.memory_regions
            .iter()
            .filter(|region| region.mem_type == MemoryType::RAM)
    }
}

impl FrameAllocator for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        let mut bitmap_index = 0;
        for region in self.ram_regions() {
            // number of frames in memory region
            let frames = (region.length as usize).div_ceil(PAGE_SIZE * 64);

            for (index, bitmap) in self.bitmaps[bitmap_index..bitmap_index + frames]
                .iter_mut()
                .enumerate()
            {
                if *bitmap == !0 {
                    continue;
                }

                // we've found a bitmap with a free page! so now find the (lowest) unset bit, and then set it
                let unset_bit = bitmap.trailing_ones() as usize;
                *bitmap |= 1 << unset_bit;

                let addr = ((index << 6) | unset_bit) << 12;

                return Some(Frame::containing_address(addr + region.base_addr as usize));
            }

            bitmap_index += align_up(frames, BITMAP_LENGTH);
        }

        // all bitmaps are full
        None
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        let frame_addr = frame.start_address();

        let mut bitmap_index = 0;
        for region in self.ram_regions() {
            // skip regions until we find memory region that address lies in
            if (region.base_addr as usize + region.length as usize) < frame.start_address() {
                let frames = (region.length as usize).div_ceil(PAGE_SIZE * 64);
                bitmap_index += align_up(frames, BITMAP_LENGTH);
                continue;
            }

            // then find index into bitmaps & bit index for frame to deallocate
            let frame_index = bitmap_index + ((frame_addr - region.base_addr as usize) >> 18);
            let bit_index = (frame_addr % 0x40000) >> 12;

            self.bitmaps[frame_index] &= !generate_mask(bit_index, bit_index + 1);
            break;
        }
    }
}

/// Generates a mask with all bits set in the range start..end
fn generate_mask(start: usize, end: usize) -> u64 {
    !(!0u64 << (end - start)) << start
}
