use alloc::alloc::GlobalAlloc;
use core::sync::atomic::{AtomicUsize, Ordering};

pub const HEAP_START: usize = 0x40000000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// A simple allocator that allocates memory linearly and ignores freed memory.
pub struct BumpAllocator {
    heap_end: usize,
    next: AtomicUsize,
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);

impl BumpAllocator {
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_end,
            next: AtomicUsize::new(heap_start),
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        loop {
            // load current state of the `next` field
            let current_next = self.next.load(Ordering::Relaxed);
            let alloc_start = align_up(current_next, layout.align());
            let alloc_end = alloc_start.saturating_add(layout.size());

            if alloc_end <= self.heap_end {
                // update the `next` pointer if it still has the value `current_next`
                let next_now = self
                    .next
                    .compare_exchange(
                        current_next,
                        alloc_end,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .unwrap();

                if next_now == current_next {
                    return alloc_start as *mut u8;
                }
            } else {
                return core::ptr::null_mut();
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // do nothing
    }
}

/// Align downwards - returns the greatest _x_ with alignment `align`
/// such that _x_ <= addr. `align` must be power of 2
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be power of two")
    }
}

/// Align upwards - returns the smallest _x_ with alignment `align`
/// such that _x_ >= addr. `align` must be power of 2
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
