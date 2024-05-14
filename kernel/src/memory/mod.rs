mod frame_allocator;
mod heap_allocator;
mod map;
mod paging;

use core::sync::atomic::{AtomicBool, Ordering};

use multiboot::{ElfSymbols, Module};
pub use paging::*;
use x86_64::structures::Frame;

use crate::{memory::frame_allocator::BitmapFrameAllocator, BootInfo};

macro_rules! log_mapping {
    ($string:expr, start: $start:expr, end: $end:expr $(, $args:expr)*) => {
        log::info!(
            concat!($string, " at addr: {:#X}, size: {:#X} ({:#X}-{:#X})"),
            $($args,)*
            $start,
            $end - $start,
            crate::memory::align_down($start as usize, crate::memory::PAGE_SIZE),
            crate::memory::align_up($end as usize, crate::memory::PAGE_SIZE),
        );
    };
    ($string:expr, start: $start:expr, len: $len:expr $(, $args:expr)*) => {
        crate::memory::log_mapping!($string, start: $start, end: ($start + $len) $(, $args)*)
    };
}

use log_mapping;

/// Size of a page in bytes
pub const PAGE_SIZE: usize = 4096;

pub trait FrameAllocator {
    /// Finds a free frame to allocate and return
    fn allocate_frame(&mut self) -> Option<Frame>;

    /// Frees a given frame
    fn deallocate_frame(&mut self, frame: Frame);
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

    let alloc_start_addr = align_up(multiboot_end.max(kernel_end as usize), PAGE_SIZE);
    let (mut frame_allocator, (frame_start, frame_end)) =
        BitmapFrameAllocator::new(alloc_start_addr, bootinfo.memory_map.unwrap().entries);

    frame_allocator.set_ignored_frames(&frame_start, &frame_end);
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
