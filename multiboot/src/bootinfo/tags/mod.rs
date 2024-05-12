#![allow(unused)]

use core::ffi::CStr;

pub mod bios_device;
pub mod elf_symbols;
pub mod framebuffer_info;
pub mod memory;
pub mod memory_map;
pub mod module;
pub mod vbe_info;

pub use bios_device::BiosDevice;
pub use elf_symbols::ElfSymbols;
pub use framebuffer_info::*;
pub use memory::Memory;
pub use memory_map::*;
pub use module::Module;
pub use vbe_info::VBEInfo;

// each of these represents the tags found at https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format
pub const END: u32 = 0;
pub const BOOT_COMMAND_LINE: u32 = 1;
pub const BOOTLOADER_NAME: u32 = 2;
pub const MODULE: u32 = 3;
pub const MEMORY: u32 = 4;
pub const BIOS_DEVICE: u32 = 5;
pub const MEMORY_MAP: u32 = 6;
pub const VBE_INFO: u32 = 7;
pub const FRAMEBUFFER_INFO: u32 = 8;
pub const ELF_SYMBOLS: u32 = 9;
pub const APM_TABLE: u32 = 10;
pub const EFI_TABLE_32BIT: u32 = 11;
pub const EFI_TABLE_64BIT: u32 = 12;
pub const SMBIOS_TABLE: u32 = 13;
pub const ACPI_RSDP_OLD: u32 = 14;
pub const ACPI_RSDP_NEW: u32 = 15;
pub const NETWORKING: u32 = 16;
pub const EFI_MEMORY_MAP: u32 = 17;
pub const EFI_BOOT_SERVICES_NOT_TERMINATED: u32 = 18;
pub const EFI_HANDLE_32BIT: u32 = 19;
pub const EFI_HANDLE_64BIT: u32 = 20;
pub const IMAGE_LOAD_ADDRESS: u32 = 21;

/// Represents a type that can be parsed into a tag
trait ParseTag: Sized {
    unsafe fn parse(addr: *const u32, size: usize) -> Option<Tag>;
}

/// Stores data about an individual multiboot tag
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format
#[derive(Debug)]
pub enum Tag {
    End,
    BootCommandLine(&'static CStr),
    BootloaderName(&'static CStr),
    Module(Module),
    Memory(Memory),
    BiosDevice(BiosDevice),
    MemoryMap(MemoryMap),
    VBEInfo(VBEInfo),
    FramebufferInfo(FramebufferInfo),
    ElfSymbols(ElfSymbols),
}

impl Tag {
    /// Parses a tag at the given address, returning the parsed tag and the size of it
    pub unsafe fn parse_tag(mut addr: *const u32) -> (Option<Self>, usize) {
        /* all tags start with following format
                +-------------------+
        u32     | type              |
        u32     | size              |
                +-------------------+
        */
        let tag_type = *addr;
        let size = *addr.add(1) as usize;

        // then skip past first two entries to make parsing easier
        addr = addr.add(2);

        let tag = match tag_type {
            END => Some(Tag::End),
            BOOT_COMMAND_LINE => {
                /*      +-------------------+
                u8[n]   | string            |
                        +-------------------+ */

                Some(Tag::BootCommandLine(CStr::from_ptr(addr as *const i8)))
            }
            BOOTLOADER_NAME => {
                /*      +-------------------+
                u8[n]   | string            |
                        +-------------------+ */

                Some(Tag::BootloaderName(CStr::from_ptr(addr as *const i8)))
            }
            MODULE => Module::parse(addr, size),
            MEMORY => Memory::parse(addr, size),
            BIOS_DEVICE => BiosDevice::parse(addr, size),
            MEMORY_MAP => MemoryMap::parse(addr, size),
            VBE_INFO => VBEInfo::parse(addr, size),
            FRAMEBUFFER_INFO => FramebufferInfo::parse(addr, size),
            ELF_SYMBOLS => ElfSymbols::parse(addr, size),
            _ => None,
        };

        (tag, size)
    }
}
