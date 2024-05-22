#![no_std]
#![feature(const_mut_refs)]

use core::{ops::DerefMut, panic::PanicInfo};

use kernel_shared::{
    logger::Logger,
    memory::{
        frame_alloc::{bitmap::BitmapFrameAllocator, FrameAllocator},
        paging::{
            active_table::ActivePageTable, entry::EntryFlags, inactive_table::InactivePageTable,
            mapper::Mapper,
        },
    },
};
use multiboot::prelude::*;
use x86_64::{
    align_up_to_page,
    structures::{Frame, Page},
};

static LOGGER: Logger = Logger::new(log::LevelFilter::Trace);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{info}");
    x86_64::hlt_loop()
}

#[no_mangle]
extern "C" fn loader_main(addr: *const u32) {
    LOGGER.init();
    log::trace!("jumped to loader_main!");

    // find info about kernel loader, and kernel/initrd modules
    let bootinfo = unsafe { BootInfo::<4>::new(addr) };

    let kernel = bootinfo.get_module(c"kernel").expect("no kernel module!");
    let initrd = bootinfo.get_module(c"initrd").expect("no initrd module!");

    let (bootinfo_start, bootinfo_end) = (bootinfo.addr, bootinfo.addr + bootinfo.total_size);
    let (loader_start, loader_end) = loader_range(&bootinfo.elf_symbols.unwrap());
    let (kernel_start, kernel_end) = (kernel.start as usize, kernel.end as usize);
    let (initrd_start, initrd_end) = (initrd.start as usize, initrd.end as usize);

    // find first free phys address and initialise frame allocator
    let first_free_addr =
        align_up_to_page(bootinfo_end.max(loader_end).max(kernel_end).max(initrd_end));

    let (mut frame_alloc, (alloc_start, alloc_end)) =
        BitmapFrameAllocator::new(first_free_addr, bootinfo.memory_map.unwrap().entries);

    // dont overwrite any existing data
    frame_alloc.set_ignored_area(bootinfo_start, bootinfo_end);
    frame_alloc.set_ignored_area(loader_start, loader_end);
    frame_alloc.set_ignored_area(kernel_start, kernel_end);
    frame_alloc.set_ignored_area(initrd_start, initrd_end);

    let table_frame = frame_alloc
        .allocate_frame()
        .expect("failed to allocate a frame for level 4 table");
    let mut table = unsafe { InactivePageTable::new(table_frame) };

    // make sure to map loader in new table
    map_loader(&mut frame_alloc, &mut table, loader_start, loader_end);

    // then map frame allocator and frame buffer
    map_frame_allocator(&mut frame_alloc, &mut table, alloc_start, alloc_end);

    let framebuffer = bootinfo.framebuffer_info.unwrap();
    map_framebuffer(
        &mut frame_alloc,
        &mut table,
        framebuffer.buffer_addr as usize,
        (framebuffer.pitch * framebuffer.height) as usize,
    );

    // TODO: parse kernel module and map at 0xffffffff80000000

    map_heap(&mut frame_alloc, &mut table, kernel_shared::HEAP_SIZE);
    map_phys_memory(&mut frame_alloc, &mut table, &bootinfo.memory_map.unwrap());

    // finally switch to new table
    drop(bootinfo);
    let mut active_table = unsafe { ActivePageTable::new() };
    active_table.switch(table);

    log::trace!("switched active table!");

    x86_64::hlt_loop();
}

/// Identity maps loader
fn map_loader<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    start_addr: usize,
    end_addr: usize,
) {
    let start_frame = Frame::containing_address(start_addr);
    let end_frame = Frame::containing_address(end_addr);

    for frame in Frame::range_inclusive(start_frame, end_frame) {
        table.identity_map(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE, alloc);
    }
}

/// Maps frame allocator to 0xFFFFFFFF00000000
fn map_frame_allocator<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    start_frame: Frame,
    end_frame: Frame,
) {
    log::trace!("mapping frame allocator");

    let start_page = Page::containing_address(0xFFFFFFFF00000000);
    let end_page = Page::containing_address(0xFFFFFFFF1FFFFFFF);

    for (page, frame) in Page::range_inclusive(start_page, end_page)
        .zip(Frame::range_inclusive(start_frame, end_frame))
    {
        table.map_to(
            page,
            frame,
            EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
            alloc,
        );
    }
}

/// Maps framebuffer to 0xFFFFFFFF40000000
fn map_framebuffer<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    start_addr: usize,
    size: usize,
) {
    log::trace!("mapping framebuffer");

    let start_page = Page::containing_address(0xFFFFFFFF40000000);
    let end_page = Page::containing_address(0xFFFFFFFF7FFFFFFF);

    let start_frame = Frame::containing_address(start_addr);
    let end_frame = Frame::containing_address(start_addr + size - 1);

    for (page, frame) in Page::range_inclusive(start_page, end_page)
        .zip(Frame::range_inclusive(start_frame, end_frame))
    {
        table.map_to(
            page,
            frame,
            EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
            alloc,
        );
    }
}

/// Maps heap to 0xFFFFFFFF20000000
fn map_heap<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    size: usize,
) {
    log::trace!("mapping heap");

    let end_addr = (0xFFFFFFFF20000000 + size).min(0xFFFFFFFF3FFFFFFF);

    let start_page = Page::containing_address(0xFFFFFFFF20000000);
    let end_page = Page::containing_address(end_addr);

    for page in Page::range_inclusive(start_page, end_page) {
        table.map(
            page,
            EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
            alloc,
        );
    }
}

/// Maps physical memory to 0xFFFF800000000000
fn map_phys_memory<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    memory_map: &MemoryMap,
) {
    let highest_address = memory_map
        .entries
        .iter()
        .filter(|entry| entry.mem_type == MemoryType::RAM)
        .map(|entry| entry.base_addr + entry.length)
        .max()
        .unwrap() as usize;

    let start_page = Page::containing_address(0xFFFF800000000000);
    let end_page = Page::containing_address(0xFFFFBFFFFFFFFFFF);

    let start_frame = Frame::containing_address(0);
    let end_frame = Frame::containing_address(highest_address);

    for (page, frame) in Page::range_inclusive(start_page, end_page)
        .zip(Frame::range_inclusive(start_frame, end_frame))
    {
        table.map_to(
            page,
            frame,
            EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
            alloc,
        );
    }
}

/// Finds where loader lies in physical memory
fn loader_range(elf_symbols: &ElfSymbols) -> (usize, usize) {
    let loader_start = elf_symbols
        .headers
        .iter()
        .filter(|header| header.is_loaded())
        .map(|header| header.addr)
        .min()
        .unwrap();

    let loader_end = elf_symbols
        .headers
        .iter()
        .filter(|header| header.is_loaded())
        .map(|header| header.addr + header.size)
        .max()
        .unwrap();

    (loader_start as usize, loader_end as usize)
}

multiboot_header! {
    arch: 0,
    tags: [
        InformationRequest {
            requests: &[ELF_SYMBOLS, MEMORY_MAP]
        },
        ConsoleFlags::all(),
        Framebuffer {
            width: Value(1920),
            height: Value(1080),
            depth: NoPreference
        },
    ]
}
