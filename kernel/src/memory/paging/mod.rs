use core::ops::{Deref, DerefMut};

pub use entry::EntryFlags;
use kernel_shared::memory::frame_alloc::FrameAllocator;
use multiboot::elf_symbols::SectionFlags;
use x86_64::{
    registers::CR3,
    structures::{Frame, Page},
};

use self::{mapper::Mapper, temporary_page::TemporaryPage};
use super::PAGE_SIZE;
use crate::BootInfo;

mod entry;
mod mapper;
mod table;
mod temporary_page;

/// Number of entries per page (4KiB / 8 bytes)
const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

/// Stores information about the page tables currently loaded by the CPU
pub struct ActivePageTable {
    mapper: Mapper,
}

impl ActivePageTable {
    /// Reads active page table
    pub unsafe fn new() -> Self {
        Self {
            mapper: Mapper::new(),
        }
    }

    /// Executes a function with recursive mapping based on provided inactive table
    pub fn with<F: FnOnce(&mut Mapper)>(
        &mut self,
        table: &mut InactivePageTable,
        temporary_page: &mut temporary_page::TemporaryPage,
        f: F,
    ) {
        {
            let backup = CR3::read().0;
            let p4_table = temporary_page.map_table_frame(unsafe { backup.clone() }, self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(
                unsafe { table.p4_frame.clone() },
                EntryFlags::PRESENT | EntryFlags::WRITABLE,
            );
            CR3::flush_tlb();

            // execute f in the new context
            f(self);

            // restore backup
            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            CR3::flush_tlb();
        }

        temporary_page.unmap(self);
    }

    /// Switches the currently loaded table to the provided inactive table
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let (frame, flags) = CR3::read();
        let old_table = InactivePageTable { p4_frame: frame };

        unsafe { CR3::write(new_table.p4_frame, flags) }

        old_table
    }
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

/// Stores information about a page table which is not currently loaded by the CPU
pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    /// Creates an inactive page table
    pub fn new(
        frame: Frame,
        active_table: &mut ActivePageTable,
        temporary_page: &mut TemporaryPage,
    ) -> Self {
        {
            let table = temporary_page.map_table_frame(unsafe { frame.clone() }, active_table);

            // zero and set up recursive mapping
            table.zero();
            table[511].set(
                unsafe { frame.clone() },
                EntryFlags::PRESENT | EntryFlags::WRITABLE,
            );
        }
        temporary_page.unmap(active_table);

        Self { p4_frame: frame }
    }
}

/// Remaps the kernel using provided frame allocator, removing the huge pages and reckless
/// identity mapping used in bootstrap assembly. Identity maps frames between `frame_start`
/// and `frame_end` and returns the new active page table & the page to set as guard page.
pub fn remap_kernel<A: FrameAllocator>(
    allocator: &mut A,
    bootinfo: &BootInfo,
    frame_start: Frame,
    frame_end: Frame,
) -> (ActivePageTable, Page) {
    log::info!("\t* remapping kernel");
    // create temporary page at arbitrary (but unused) page
    let mut temporary_page = TemporaryPage::new(Page { number: 0xDEADBEEF }, allocator);

    // load active table and create an inactive table we'll use to set up mappings
    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    // set up mappings for new table
    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_symbols = bootinfo.elf_symbols.expect("no memory map tag");
        let string_header = elf_symbols.string_header();

        // map all elf sections
        for section in elf_symbols.headers {
            // skip entries that don't need to be mapped
            if !section.flags.contains(SectionFlags::ALLOC) {
                continue;
            }

            // ensure section is aligned to page
            assert!(
                section.addr as usize % PAGE_SIZE == 0,
                "sections need to be page aligned"
            );
            super::log_mapping!(
                "\t\t* mapping kernel section {:?}",
                start: section.addr,
                len: section.size,
                section.name(string_header)
            );

            let flags = EntryFlags::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.addr as usize);
            let end_frame = Frame::containing_address((section.addr + section.size - 1) as usize);

            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        // also map multiboot info here, since held reference needs to stay valid
        let multiboot_start = Frame::containing_address(bootinfo.addr);
        let multiboot_end = Frame::containing_address(bootinfo.addr + bootinfo.total_size - 1);
        super::log_mapping!(
            "\t\t* mapping multiboot info",
            start: bootinfo.addr,
            len: bootinfo.total_size
        );

        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }

        // allocator frames need to be mapped here to prevent GPF when new mappings are loaded
        for frame in Frame::range_inclusive(frame_start, frame_end) {
            mapper.identity_map(frame, EntryFlags::WRITABLE, allocator);
        }
    });

    // switch tables to use new mappings
    let old_table = active_table.switch(new_table);
    log::trace!("\t\t* new p4 table loaded");

    // now set old p4 table as a guard page to prevent stack overflows
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator, true);

    log::trace!("\t\t* guard page at {:#X}", old_p4_page.start_address());

    log::info!("\t* kernel remapped");

    (active_table, old_p4_page)
}
