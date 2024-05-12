use core::ops::{Deref, DerefMut};

pub use entry::EntryFlags;
use multiboot::{elf_symbols::SectionFlags, Module};
use x86_64::{
    registers::CR3,
    structures::{Frame, Page},
};

use self::{mapper::Mapper, temporary_page::TemporaryPage};
use super::{FrameAllocator, PAGE_SIZE};
use crate::{println, serial_println, BootInfo};

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
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(
                table.p4_frame.clone(),
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
            let table = temporary_page.map_table_frame(frame.clone(), active_table);

            // zero and set up recursive mapping
            table.zero();
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);

        Self { p4_frame: frame }
    }
}

/// Remaps the kernel using provided frame allocator, removing the huge pages and reckless
/// identity mapping used in bootstrap assembly.
pub fn remap_kernel<A: FrameAllocator>(
    allocator: &mut A,
    bootinfo: &BootInfo,
    initrd: &Module,
) -> ActivePageTable {
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

            serial_println!(
                "mapping kernel section {:?} at addr: {:#x}, size: {:#x}",
                section.name(string_header),
                section.addr,
                section.size
            );

            let flags = EntryFlags::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.addr as usize);
            let end_frame = Frame::containing_address((section.addr + section.size - 1) as usize);

            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        // then map VGA buffer
        let framebuffer_info = bootinfo.framebuffer_info.unwrap();
        let buffer_start = framebuffer_info.buffer_addr as usize;
        let buffer_end = buffer_start + (framebuffer_info.pitch * framebuffer_info.height) as usize;
        serial_println!(
            "mapping framebuffer at addr: {:#x}, size: {:#x}",
            buffer_start,
            buffer_end - buffer_start
        );

        for frame in Frame::range_inclusive(
            Frame::containing_address(buffer_start),
            Frame::containing_address(buffer_end),
        ) {
            mapper.identity_map(frame, EntryFlags::WRITABLE, allocator);
        }

        // map multiboot info
        let multiboot_start = Frame::containing_address(bootinfo.addr);
        let multiboot_end = Frame::containing_address(bootinfo.addr + bootinfo.total_size - 1);
        serial_println!(
            "mapping multiboot info at addr: {:#x}, size: {:#x}",
            bootinfo.addr,
            bootinfo.total_size
        );

        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            serial_println!(
                "mapping page {:?} to frame {:?}",
                Page::containing_address(frame.start_address()),
                frame
            );
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }

        let initrd_start_page = Frame::containing_address(initrd.start as usize);
        let initrd_end_page = Frame::containing_address(initrd.end as usize);
        serial_println!(
            "mapping initrd at addr: {:#x}, size: {:#x}",
            initrd.start,
            initrd.end - initrd.start
        );

        for frame in Frame::range_inclusive(initrd_start_page, initrd_end_page) {
            serial_println!(
                "mapping page {:?} to frame {:?}",
                Page::containing_address(frame.start_address() | 0x00FF_FFFF_0000),
                frame
            );
            mapper.map_to(
                Page::containing_address(frame.start_address() | 0x00FF_FFFF_0000),
                frame,
                EntryFlags::PRESENT,
                allocator,
            );
        }
    });

    // finally switch tables to use new mappings
    let old_table = active_table.switch(new_table);
    serial_println!("new p4 table loaded");

    // now set old p4 table as a guard page to prevent stack overflows
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);
    serial_println!("guard page at {:#x}", old_p4_page.start_address());

    active_table
}
