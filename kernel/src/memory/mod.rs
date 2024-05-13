mod area_frame_allocator;
mod heap_allocator;
mod paging;

use core::sync::atomic::{AtomicBool, Ordering};

pub use area_frame_allocator::AreaFrameAllocator;
use multiboot::{ElfSymbols, Module};
pub use paging::*;
use x86_64::structures::{Frame, Page};

use self::heap_allocator::{HEAP_SIZE, HEAP_START};
use crate::BootInfo;

/// Size of a page in bytes
pub const PAGE_SIZE: usize = 4096;

pub trait FrameAllocator {
    /// Finds a free frame to allocate and return
    fn allocate_frame(&mut self) -> Option<Frame>;

    /// Frees a given frame
    #[allow(unused)]
    fn deallocate_frame(&mut self, frame: Frame);
}

static INIT_CALLED: AtomicBool = AtomicBool::new(false);

/// Initialises memory
pub fn init(bootinfo: &BootInfo, initrd: &Module) {
    if INIT_CALLED.swap(true, Ordering::Relaxed) {
        panic!("memory::init must only be called once")
    }

    let elf_symbols = bootinfo.elf_symbols.expect("Memory map tag required");
    let (kernel_start, kernel_end) = kernel_range(&elf_symbols);
    log::trace!(
        "kernel found in range {:#X} to {:#X}",
        kernel_start,
        kernel_end
    );

    let multiboot_start = bootinfo.addr;
    let multiboot_end = multiboot_start + bootinfo.total_size;
    log::trace!(
        "multiboot info found in range {:#X} to {:#X}",
        multiboot_start,
        multiboot_end
    );

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        multiboot_start,
        multiboot_end,
        bootinfo.memory_map.unwrap().entries,
    );

    let mut active_table = remap_kernel(&mut frame_allocator, bootinfo, initrd);

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }

    log::info!(
        "mapping heap at addr: {:#X}, size: {:#X}",
        HEAP_START,
        HEAP_START + HEAP_SIZE - 1
    );
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
