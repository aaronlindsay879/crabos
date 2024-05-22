mod heap_allocator;

use alloc::{boxed::Box, vec::Vec};
use core::sync::atomic::{AtomicBool, Ordering};

use kernel_shared::memory::{
    frame_alloc::bitmap::BitmapFrameAllocator, paging::active_table::ActivePageTable,
};
use x86_64::structures::Page;

use crate::BootInfo;

/// Initialises memory
pub fn init(
    bootinfo: &BootInfo,
    loader_start: usize,
    loader_end: usize,
) -> (BitmapFrameAllocator, ActivePageTable) {
    static INIT_CALLED: AtomicBool = AtomicBool::new(false);

    if INIT_CALLED.swap(true, Ordering::Relaxed) {
        panic!("memory::init must only be called once")
    }

    log::info!("initialising memory");

    // clone and leak memory map to make sure we have a reference that doesnt live in old loader memory space
    let memory_map = Box::new(bootinfo.memory_map.unwrap().entries.to_vec());
    let mut frame_alloc =
        unsafe { BitmapFrameAllocator::from_address(Box::leak(memory_map), 0xFFFFFFFF00000000) };

    let mut active_table = unsafe { ActivePageTable::new() };

    unsafe {
        free_region(
            &mut active_table,
            &mut frame_alloc,
            loader_start,
            loader_end,
        );
    }
    log::trace!("\t* loader memory freed");

    log::info!("memory initialised");

    (frame_alloc, active_table)
}

pub unsafe fn free_region(
    active_table: &mut ActivePageTable,
    frame_alloc: &mut BitmapFrameAllocator,
    addr_start: usize,
    addr_end: usize,
) {
    let start_page = Page::containing_address(addr_start);
    let end_page = Page::containing_address(addr_end);

    for page in Page::range_inclusive(start_page, end_page) {
        active_table.unmap(page, frame_alloc, false);
    }
}
