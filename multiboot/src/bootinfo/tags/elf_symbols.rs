use core::{ffi::CStr, fmt::Display};

use bitflags::bitflags;

use super::{ParseTag, Tag};

bitflags! {
    /// Flags for a specific section
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SectionFlags: u64 {
        /// Whether the section is writeable
        const WRITE = 0x1;
        /// Whether the section is allocated to memory
        const ALLOC = 0x2;
        /// Whether the section is executable
        const EXECUTABLE = 0x4;
    }
}

impl Display for SectionFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for string in self.iter_names().map(|(a, _)| a).intersperse(" | ") {
            f.write_str(string)?;
        }

        Ok(())
    }
}

/// Stores information about the ELF sections for the kernel
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#ELF_002dSymbols
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ElfSymbols {
    /// Number of sections
    pub num: u32,
    /// Size of each section
    pub entry_size: u32,
    /// Index into [Self::headers] for the string table
    pub string_table_index: u32,
    /// List of headers for each section
    pub headers: &'static [ElfSectionHeader],
}

impl ElfSymbols {
    /// Returns the header for the string table
    pub const fn string_header(&self) -> &ElfSectionHeader {
        &self.headers[self.string_table_index as usize]
    }
}

/// Header for a single ELF section
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ElfSectionHeader {
    pub name: u32,
    pub section_type: u32,
    pub flags: SectionFlags,
    pub addr: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub addralign: u64,
    pub entry_size: u64,
}

impl ElfSectionHeader {
    /// Returns the name of the header using the provided string table
    pub fn name(&self, string_header: &ElfSectionHeader) -> &'static CStr {
        let location = string_header.addr as *const i8;

        unsafe { CStr::from_ptr(location.add(self.name as usize)) }
    }

    /// Checks if the section is loaded in memory
    pub fn is_loaded(&self) -> bool {
        self.flags.contains(SectionFlags::ALLOC)
    }
}

impl ParseTag for ElfSymbols {
    unsafe fn parse(addr: *const u32, _size: usize) -> Option<Tag> {
        /*      +-------------------+
        u32     | num               |
        u32     | entsize           |
        u32     | shndx             |
        varies  | section headers   |
                +-------------------+ */

        let num = *addr;
        let entry_size = *addr.add(1);
        let string_table_index = *addr.add(2);

        let headers =
            core::slice::from_raw_parts(addr.add(3) as *const ElfSectionHeader, num as usize);

        Some(Tag::ElfSymbols(ElfSymbols {
            num,
            entry_size,
            string_table_index,
            headers,
        }))
    }
}
