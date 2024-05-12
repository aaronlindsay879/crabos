use super::{ParseTag, Tag};

/// Stores a list of memory regions available to the kernel
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Memory-map
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryMap {
    pub version: u32,
    pub entries: &'static [MemoryMapEntry],
}

/// Stores information about a single region of memory
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub mem_type: MemoryType,
    _reserved: u32,
}

/// The type of memory region
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryType {
    RAM = 1,
    RESERVED,
    ACPI_INFO,
    PRESERVED_ON_HIBERNATION,
    DEFECTIVE,
}

impl ParseTag for MemoryMap {
    unsafe fn parse(addr: *const u32, size: usize) -> Option<Tag> {
        /*       +-------------------+
        u32     | entry_size        |
        u32     | entry_version     |
        varies  | entries           |
                +-------------------+ */

        /* where entries has the following format
                +-------------------+
        u64     | base_addr         |
        u64     | length            |
        u32     | type              |
        u32     | reserved          |
                +-------------------+ */

        let entry_size = *addr as usize;
        let version = *addr.add(1);

        // calculate number of entries from given info, and then construct a slice with that many entries
        let entries_size = size - 16;
        let num_entries = entries_size / entry_size;

        let entries = core::slice::from_raw_parts(addr.add(2) as *const _, num_entries);

        Some(Tag::MemoryMap(MemoryMap { version, entries }))
    }
}
