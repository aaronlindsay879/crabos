use multiboot::Module;
use x86_64::structures::{Frame, Page};

use super::{ActivePageTable, FrameAllocator};
use crate::{
    memory::{
        heap_allocator::{HEAP_SIZE, HEAP_START},
        EntryFlags,
    },
    BootInfo,
};

/// Identity maps framebuffer using data provided in `bootinfo`
pub(super) fn map_framebuffer<A: FrameAllocator>(
    bootinfo: &BootInfo,
    active_table: &mut ActivePageTable,
    frame_allocator: &mut A,
) {
    let framebuffer_info = bootinfo.framebuffer_info.unwrap();
    let buffer_start = framebuffer_info.buffer_addr as usize;
    let buffer_end = buffer_start + (framebuffer_info.pitch * framebuffer_info.height) as usize;

    log::info!(
        "\t* mapping framebuffer at addr: {:#X}, size: {:#X} ({:#X}-{:#X})",
        buffer_start,
        buffer_end - buffer_start,
        buffer_start,
        buffer_end
    );

    for frame in Frame::range_inclusive(
        Frame::containing_address(buffer_start),
        Frame::containing_address(buffer_end),
    ) {
        active_table.identity_map(frame, EntryFlags::WRITABLE, frame_allocator);
    }
}

/// Maps initrd to [RAMFS_ADDR](crate::RAMFS_ADDR)
pub(super) fn map_initrd<A: FrameAllocator>(
    initrd: &Module,
    active_table: &mut ActivePageTable,
    frame_allocator: &mut A,
) {
    let initrd_offset = initrd.start as usize;
    let initrd_len = (initrd.end - initrd.start) as usize;
    assert!(
        initrd_len <= 0xFFFF_FFFF,
        "initrd cannot be longer than 32 bits"
    );

    let initrd_start = crate::RAMFS_ADDR;
    let initrd_end = initrd_len | crate::RAMFS_ADDR;

    let initrd_start_page = Page::containing_address(initrd_start);
    let initrd_end_page = Page::containing_address(initrd_end);
    log::info!(
        "\t* mapping initrd at addr: {:#X}, size: {:#X} ({:#X}-{:#X})",
        initrd_start,
        initrd_len,
        initrd_start,
        initrd_end
    );

    for page in Page::range_inclusive(initrd_start_page, initrd_end_page) {
        active_table.map_to(
            page,
            Frame::containing_address((page.start_address() ^ crate::RAMFS_ADDR) + initrd_offset),
            EntryFlags::PRESENT,
            frame_allocator,
        );
    }
}

/// Maps heap to [HEAP_START](super::heap_allocator::HEAP_START)
pub(super) fn map_heap<A: FrameAllocator>(
    active_table: &mut ActivePageTable,
    frame_allocator: &mut A,
) {
    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);
    log::info!(
        "\t* mapping heap at addr: {:#X}, size: {:#X} ({:#X}-{:#X})",
        HEAP_START,
        HEAP_SIZE,
        HEAP_START,
        HEAP_START + HEAP_SIZE
    );

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, EntryFlags::WRITABLE, frame_allocator);
    }
}
