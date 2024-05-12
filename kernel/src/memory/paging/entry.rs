use bitflags::bitflags;
use multiboot::elf_symbols::{ElfSectionHeader, SectionFlags};
use x86_64::structures::Frame;

bitflags! {
    /// Stores possible flags for a page entry
    #[derive(Clone, Copy)]
    pub struct EntryFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const NO_EXECUTE = 1 << 63;
    }
}

impl EntryFlags {
    /// Set flags based on the flags used in ELF header
    pub fn from_elf_section_flags(section: &ElfSectionHeader) -> Self {
        let mut flags = EntryFlags::NO_EXECUTE;

        for flag in section.flags {
            match flag {
                SectionFlags::ALLOC => flags.insert(EntryFlags::PRESENT),
                SectionFlags::WRITE => flags.insert(EntryFlags::WRITABLE),
                SectionFlags::EXECUTABLE => flags.remove(EntryFlags::NO_EXECUTE),
                _ => continue,
            }
        }

        flags
    }
}

/// Stores address and flags for a page entry
pub struct Entry(u64);

impl Entry {
    /// Checks if PRESENT flag is set
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// Removes address and flags
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Sets the entry to the given frame and flags
    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        // ensure address is page aligned and smaller than 2^52
        assert!(frame.start_address() & !0x000FFFFF_FFFFF000 == 0);

        self.0 = (frame.start_address() as u64) | flags.bits();
    }

    /// Returns the flags
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Returns the frame the entry points to, if it exists
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(Frame::containing_address(
                self.0 as usize & 0x000FFFFF_FFFFF000,
            ))
        } else {
            None
        }
    }
}
