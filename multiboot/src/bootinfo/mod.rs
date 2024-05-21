pub mod tags;

use core::ffi::CStr;

pub use tags::*;

/// Contains information returned by multiboot during the booting process
#[derive(Default, Debug)]
pub struct BootInfo<const MAX_MODULES: usize>
where
    [Option<Module>; MAX_MODULES]: Default,
{
    pub addr: usize,
    pub total_size: usize,
    pub boot_command_line: Option<&'static CStr>,
    pub bootloader_name: Option<&'static CStr>,
    pub modules: [Option<Module>; MAX_MODULES],
    pub memory: Option<Memory>,
    pub bios_device: Option<BiosDevice>,
    pub memory_map: Option<MemoryMap>,
    pub vbe_info: Option<VBEInfo>,
    pub framebuffer_info: Option<FramebufferInfo>,
    pub elf_symbols: Option<ElfSymbols>,
}

impl<const MAX_MODULES: usize> BootInfo<MAX_MODULES>
where
    [Option<Module>; MAX_MODULES]: Default,
{
    /// Creates a new bootinfo struct from the given address
    ///
    /// # Safety
    /// This is **very** unsafe and must only ever be called with the address returned by multiboot2
    pub unsafe fn new(mut bootinfo: *const u32) -> Self {
        let mut info = Self::default();

        let total_size = *bootinfo as usize;
        info.addr = bootinfo as usize;
        info.total_size = total_size;

        let mut advanced = 0; // keep track of how far into bootinfo we've advanced
        assert_eq!(*bootinfo.add(1), 0); // second element in bootinfo is reserved - must equal 0

        // now advance to first tag
        bootinfo = bootinfo.add(2);
        advanced += 8;

        // keep track of module index
        let mut module_index = 0;

        // then iterate through each tag, setting each one as it's found
        while advanced < total_size {
            let (tag, size) = Tag::parse_tag(bootinfo);
            // round size up to next multiple of 8 and advance pointers
            let size = (size + 7) & !7;
            bootinfo = bootinfo.byte_add(size);
            advanced += size;

            if let Some(tag) = tag {
                match tag {
                    Tag::End => break,
                    Tag::BootCommandLine(boot_command_line) => {
                        info.boot_command_line = Some(boot_command_line)
                    }
                    Tag::BootloaderName(bootloader_name) => {
                        info.bootloader_name = Some(bootloader_name)
                    }
                    Tag::Module(module) => {
                        if module_index == MAX_MODULES {
                            panic!("too many modules");
                        }

                        info.modules[module_index] = Some(module);
                        module_index += 1;
                    }
                    Tag::Memory(memory) => info.memory = Some(memory),
                    Tag::BiosDevice(bios_device) => info.bios_device = Some(bios_device),
                    Tag::MemoryMap(memory_map) => info.memory_map = Some(memory_map),
                    Tag::VBEInfo(vbe_info) => info.vbe_info = Some(vbe_info),
                    Tag::FramebufferInfo(framebuffer_info) => {
                        info.framebuffer_info = Some(framebuffer_info)
                    }
                    Tag::ElfSymbols(elf_symbols) => info.elf_symbols = Some(elf_symbols),
                }
            }
        }

        info
    }

    pub fn get_module(&self, module_str: &CStr) -> Option<&Module> {
        self.modules
            .iter()
            .flatten()
            .find(|module| module.string == module_str)
    }
}
