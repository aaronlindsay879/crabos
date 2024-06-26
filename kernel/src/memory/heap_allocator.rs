use alloc::alloc::GlobalAlloc;
use core::sync::atomic::{AtomicUsize, Ordering};

use kernel_shared::HEAP_SIZE;
use x86_64::align_up;

pub const HEAP_START: usize = 0xFFFFFFFF20000000;

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
