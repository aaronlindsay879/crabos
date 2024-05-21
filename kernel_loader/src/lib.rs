#![no_std]
#![feature(const_mut_refs)]

use core::panic::PanicInfo;

use kernel_shared::{
    logger::Logger,
    memory::{
        frame_alloc::{bitmap::BitmapFrameAllocator, FrameAllocator},
        paging::{
            active_table::{self, ActivePageTable},
            entry::EntryFlags,
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

    let bootinfo = unsafe { BootInfo::<4>::new(addr) };

    let kernel = bootinfo.get_module(c"kernel").expect("no kernel module!");
    let initrd = bootinfo.get_module(c"initrd").expect("no initrd module!");

    let (loader_start, loader_end) = (bootinfo.addr, bootinfo.addr + bootinfo.total_size);
    let (kernel_start, kernel_end) = (kernel.start as usize, kernel.end as usize);
    let (initrd_start, initrd_end) = (initrd.start as usize, initrd.end as usize);

    let first_free_addr = align_up_to_page(loader_end.max(kernel_end).max(initrd_end));

    let (mut frame_alloc, (alloc_start, alloc_end)) =
        BitmapFrameAllocator::new(first_free_addr, bootinfo.memory_map.unwrap().entries);

    let mut active_table = unsafe { ActivePageTable::new() };

    map_frame_allocator(&mut frame_alloc, &mut active_table, alloc_start, alloc_end);

    // parse kernel module and map at 0xffffffff80000000
    // map framebuffer at 0xffffffff40000000
    // map heap at 0xffffffff20000000 but dont initialise
    // map remaining physical memory at 0xffff800000000000 (or shrink existing mappings)

    x86_64::hlt_loop();
}

fn map_frame_allocator<A: FrameAllocator>(
    alloc: &mut A,
    active_table: &mut ActivePageTable,
    start_frame: Frame,
    end_frame: Frame,
) {
    log::trace!("mapping frame allocator");

    let start_page = Page::containing_address(0xFFFFFFFF00000000);
    let end_page = Page::containing_address(0xFFFFFFFF1FFFFFFF);

    for (page, frame) in Page::range_inclusive(start_page, end_page)
        .zip(Frame::range_inclusive(start_frame, end_frame))
    {
        active_table.map_to(page, frame, EntryFlags::WRITABLE, alloc);
    }
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
