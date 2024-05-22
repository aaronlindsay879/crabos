use multiboot::elf_symbols::ElfSectionHeader;

#[repr(C)]
#[derive(Debug)]
pub struct ElfHeader {
    pub ident: [u8; 16],
    pub file_type: u16,
    pub machine_version: u16,
    pub file_version: u32,
    pub entrypoint: usize,
    pub program_header_offset: usize,
    pub section_header_offset: usize,
    pub flags: u32,
    pub header_size: u16,
    pub program_header_size: u16,
    pub program_header_entries: u16,
    pub section_header_size: u16,
    pub section_header_entries: u16,
    pub string_table_index: u16,
}

pub struct ElfFile {
    data: &'static [u8],
}

impl ElfFile {
    pub fn new(data: &'static [u8]) -> Option<Self> {
        if &data[0..4] == b"\x7FELF" {
            Some(Self { data })
        } else {
            None
        }
    }

    pub fn header(&self) -> &ElfHeader {
        unsafe { &*(self.data.as_ptr() as *const ElfHeader) }
    }

    pub fn entrypoint(&self) -> usize {
        self.header().entrypoint
    }

    pub fn section_headers(&self) -> &[ElfSectionHeader] {
        let header = self.header();

        unsafe {
            core::slice::from_raw_parts(
                self.data.as_ptr().add(header.section_header_offset) as *const _,
                header.section_header_entries as usize,
            )
        }
    }

    pub fn string_header(&self) -> &ElfSectionHeader {
        let header = self.header();
        &self.section_headers()[header.string_table_index as usize]
    }
}
