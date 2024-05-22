#![no_std]
#![feature(const_mut_refs, const_trait_impl, effects)]

mod elf;

use core::{arch::asm, ffi::CStr, ops::DerefMut, panic::PanicInfo};

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
use multiboot::{elf_symbols::SectionFlags, prelude::*};
use x86_64::{
    align_down_to_page, align_up_to_page,
    structures::{Frame, Page, PAGE_SIZE},
};

use crate::elf::ElfFile;

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

    log::trace!("initialised frame allocator");

    // dont overwrite any existing data
    frame_alloc.set_ignored_area(bootinfo_start, bootinfo_end);
    frame_alloc.set_ignored_area(loader_start, loader_end);
    frame_alloc.set_ignored_area(kernel_start, kernel_end);
    frame_alloc.set_ignored_area(initrd_start, initrd_end);

    let table_frame = frame_alloc
        .allocate_frame()
        .expect("failed to allocate a frame for level 4 table");
    let mut table = unsafe { InactivePageTable::new(table_frame) };

    // make sure to map loader, initrd and bootinfo
    let start_page = Page::containing_address(0);
    let end_page = Page::containing_address(bootinfo_end - bootinfo_start);

    let start_frame = Frame::containing_address(bootinfo_start);
    let end_frame = Frame::containing_address(bootinfo_end);

    log::trace!(
        "mapping bootinfo at {:#X}-{:#X}",
        start_page.start_address(),
        end_page.start_address()
    );

    for (page, frame) in Page::range_inclusive(start_page, end_page)
        .zip(Frame::range_inclusive(start_frame, end_frame))
    {
        table.map_to(page, frame, EntryFlags::WRITABLE, &mut frame_alloc);
    }

    identity_map(
        "initrd",
        &mut frame_alloc,
        &mut table,
        initrd_start,
        initrd_end,
    );
    identity_map(
        "loader",
        &mut frame_alloc,
        &mut table,
        loader_start,
        loader_end,
    );

    // then map frame allocator and frame buffer
    map_frame_allocator(&mut frame_alloc, &mut table, alloc_start, alloc_end);

    let framebuffer = bootinfo.framebuffer_info.unwrap();
    map_framebuffer(
        &mut frame_alloc,
        &mut table,
        framebuffer.buffer_addr as usize,
        (framebuffer.pitch * framebuffer.height) as usize,
    );

    let elf_data = unsafe {
        core::slice::from_raw_parts(kernel_start as *const u8, kernel_end - kernel_start)
    };
    let kernel_elf = ElfFile::new(elf_data).expect("kernel was not a valid ELF file.");
    let string_header = kernel_elf.string_header();

    for kernel_section in kernel_elf.section_headers() {
        // only map sections that need allocating
        if !kernel_section.flags.contains(SectionFlags::ALLOC) {
            continue;
        }

        // ensure section is aligned to page
        assert!(
            kernel_section.addr as usize % PAGE_SIZE == 0,
            "sections need to be page aligned, addr {:#X}",
            kernel_section.addr
        );

        let flags = EntryFlags::from_elf_section_flags(kernel_section);

        let start_phys = kernel_section.offset as usize + kernel_start;
        let end_phys = (kernel_section.offset + kernel_section.size - 1) as usize + kernel_start;

        let start_virt = kernel_section.addr as usize;
        let end_virt = (kernel_section.addr + kernel_section.size - 1) as usize;

        let location = (string_header.offset + kernel_start as u64) as *const i8;
        let name = unsafe { CStr::from_ptr(location.add(kernel_section.name as usize)) };

        log::trace!(
            "mapping kernel section {:?} at {:#X}-{:#X} with flags `{}`",
            name,
            align_down_to_page(start_virt),
            align_down_to_page(end_virt),
            flags
        );

        // if SHT_NOBITS, make sure to zero
        if kernel_section.section_type == 8 {
            unsafe {
                core::slice::from_raw_parts_mut(
                    align_down_to_page(start_phys) as *mut u8,
                    kernel_section.size as usize,
                )
                .fill(0);
            }
        }

        table.map_range(
            (start_phys, end_phys),
            (start_virt, end_virt),
            flags,
            &mut frame_alloc,
            true,
        );
    }

    map_heap(&mut frame_alloc, &mut table, kernel_shared::HEAP_SIZE);
    map_phys_memory(&mut frame_alloc, &mut table, &bootinfo.memory_map.unwrap());

    // set up stack at final 16MiB of kernel space
    log::trace!("setting up stack at {:#X}", usize::MAX);
    let start_page = Page::containing_address(usize::MAX - kernel_shared::STACK_SIZE + 1);
    let end_page = Page::containing_address(usize::MAX);

    for page in Page::range_inclusive(start_page, end_page) {
        table.map(page, EntryFlags::WRITABLE, &mut frame_alloc);
    }

    // finally switch to new table
    let entrypoint = kernel_elf.entrypoint();

    drop(bootinfo);
    let mut active_table = unsafe { ActivePageTable::new() };
    active_table.switch(table);

    log::trace!("switched active table!");

    log::trace!("jumping to kernel at {entrypoint:#X}");
    unsafe {
        asm!(
            "mov rsp, 0xFFFFFFFFFFFFFFFF",
            "jmp {}",
            in(reg) entrypoint,
            in("rdi") addr as usize - align_down_to_page(addr as usize),
            in("rsi") loader_start,
            in("rdx") loader_end
        )
    }
}

/// Identity maps loader
fn identity_map<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    log_str: &'static str,
    alloc: &mut A,
    table: &mut T,
    start_addr: usize,
    end_addr: usize,
) {
    let start_frame = Frame::containing_address(start_addr);
    let end_frame = Frame::containing_address(end_addr);

    log::trace!(
        "mapping {log_str} at {:#X}-{:#X}",
        start_frame.start_address(),
        end_frame.start_address()
    );

    for frame in Frame::range_inclusive(start_frame, end_frame) {
        table.identity_map(frame, EntryFlags::WRITABLE, alloc);
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

    table.map_range(
        (start_frame.start_address(), end_frame.start_address()),
        (0xFFFFFFFF00000000, 0xFFFFFFFF1FFFFFFF),
        EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
        alloc,
        true,
    );
}

/// Maps framebuffer to 0xFFFFFFFF40000000
fn map_framebuffer<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    start_addr: usize,
    size: usize,
) {
    log::trace!("mapping framebuffer");

    table.map_range(
        (start_addr, start_addr + size - 1),
        (0xFFFFFFFF40000000, 0xFFFFFFFF7FFFFFFF),
        EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
        alloc,
        true,
    );
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
        table.map(page, EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE, alloc);
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

    table.map_range(
        (0, highest_address),
        (0xFFFF800000000000, 0xFFFFBFFFFFFFFFFF),
        EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
        alloc,
        true,
    );
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
        ModuleAlignment,
    ]
}
