mod heap_allocator;
mod map;
mod paging;

use core::sync::atomic::{AtomicBool, Ordering};

use kernel_shared::memory::frame_alloc::bitmap::BitmapFrameAllocator;
use multiboot::{ElfSymbols, Module};
pub use paging::*;
use x86_64::{align_up_to_page, structures::Frame};

use crate::BootInfo;

macro_rules! log_mapping {
    ($string:expr, start: $start:expr, end: $end:expr $(, $args:expr)*) => {
        log::info!(
            concat!($string, " at addr: {:#X}, size: {:#X} ({:#X}-{:#X})"),
            $($args,)*
            $start,
            $end - $start,
            x86_64::align_down_to_page($start as usize),
            x86_64::align_up_to_page($end as usize),
        );
    };
    ($string:expr, start: $start:expr, len: $len:expr $(, $args:expr)*) => {
        crate::memory::log_mapping!($string, start: $start, end: ($start + $len) $(, $args)*)
    };
}

use log_mapping;

/// Size of a page in bytes
pub const PAGE_SIZE: usize = 4096;

static INIT_CALLED: AtomicBool = AtomicBool::new(false);

/// Initialises memory
pub fn init(bootinfo: &BootInfo, initrd: &Module) {
    if INIT_CALLED.swap(true, Ordering::Relaxed) {
        panic!("memory::init must only be called once")
    }

    log::info!("initialising memory");

    let elf_symbols = bootinfo.elf_symbols.expect("Memory map tag required");
    let (kernel_start, kernel_end) = kernel_range(&elf_symbols);
    log::trace!(
        "\t* kernel found in range {:#X}-{:#X}",
        kernel_start,
        kernel_end
    );

    let multiboot_start = bootinfo.addr;
    let multiboot_end = multiboot_start + bootinfo.total_size;
    log::trace!(
        "\t* multiboot info found in range {:#X}-{:#X}",
        multiboot_start,
        multiboot_end
    );

    // find end address of important data structures, and map frame allocator after them
    // TODO: smarter placement of allocator backing bitmaps
    let alloc_start_addr = align_up_to_page(
        multiboot_end
            .max(kernel_end as usize)
            .max(initrd.end as usize),
    );
    let (mut frame_allocator, (frame_start, frame_end)) =
        BitmapFrameAllocator::new(alloc_start_addr, bootinfo.memory_map.unwrap().entries);

    frame_allocator.set_ignored_area(kernel_start as usize, kernel_end as usize);
    frame_allocator.set_ignored_area(multiboot_start, multiboot_end);

    let (mut active_table, guard_page) =
        remap_kernel(&mut frame_allocator, bootinfo, frame_start, frame_end);

    frame_allocator.set_ignored_area(guard_page.start_address(), guard_page.start_address());

    // then map VGA buffer, initrd, and heap
    map::map_framebuffer(bootinfo, &mut active_table, &mut frame_allocator);
    map::map_initrd(initrd, &mut active_table, &mut frame_allocator);
    map::map_heap(&mut active_table, &mut frame_allocator);

    log::info!("memory initialised");
}

fn kernel_range(elf_symbols: &ElfSymbols) -> (u64, u64) {
    let kernel_start = elf_symbols
        .headers
        .iter()
        .filter(|header| header.is_loaded())
        .map(|header| header.addr)
        .min()
        .unwrap();

    let kernel_end = elf_symbols
        .headers
        .iter()
        .filter(|header| header.is_loaded())
        .map(|header| header.addr + header.size)
        .max()
        .unwrap();

    (kernel_start, kernel_end)
}
