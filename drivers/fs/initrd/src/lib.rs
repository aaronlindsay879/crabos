#![no_std]

use alloc::vec;
use core::{ffi::CStr, marker::PhantomData};

use crabstd::fs::{self, Path, StorageDevice};
use ram::Ram;

extern crate alloc;

#[derive(Debug)]
pub struct Initrd<S: StorageDevice> {
    header: Header,
    data: &'static [u8],
    phantom: PhantomData<S>,
}

#[derive(Debug)]
struct Header {
    entries: &'static [TableEntry],
    string_table: &'static [u8],
}

impl Header {
    pub fn get_path(&self, path_index: usize) -> Option<&Path> {
        let string = CStr::from_bytes_until_nul(&self.string_table[path_index..]).ok();

        string
            .and_then(|string| string.to_str().ok())
            .map(|string| Path::new(string))
    }
}

#[derive(Debug)]
#[repr(C)]
struct TableEntry {
    path_index: usize,
    offset: usize,
    len: usize,
}

impl Initrd<Ram> {
    /// Specialised implementation for when initrd is in ram to avoid copying data to a buffer while reading.
    /// While those copies are needed for generic storage devices, there's no need to copy data if it's
    /// already in ram.
    pub unsafe fn new_ram(start: usize, len: usize) -> Option<Self> {
        Self::new_shared(start as *const _, len)
    }
}

impl<S: StorageDevice> Initrd<S> {
    unsafe fn new_shared(location: *const u8, len: usize) -> Option<Self> {
        // u32 - magic number "KTIY"
        // u32 - reserved
        // u64 - header count
        // u64 - string table len
        // headers
        // string table
        // data
        if *(location as *const u32) != u32::from_ne_bytes(*b"KTIY") {
            return None;
        }

        let header_count = *(location.add(8) as *const u64);
        let string_table_len = *(location.add(16) as *const u64);

        let mut header_len = 24;
        let entries = core::slice::from_raw_parts(
            location.add(header_len) as *const _,
            header_count as usize,
        );

        header_len += header_count as usize * core::mem::size_of::<TableEntry>();

        let string_table =
            core::slice::from_raw_parts(location.add(header_len), string_table_len as usize);

        header_len += string_table_len as usize;

        let header = Header {
            entries,
            string_table,
        };

        let data = core::slice::from_raw_parts(location.add(header_len), len as usize - header_len);

        Some(Self {
            header,
            data,
            phantom: PhantomData {},
        })
    }

    pub unsafe fn new(start: usize, len: usize, mut device: S) -> Option<Self> {
        let mut buffer = vec![0; len];
        let len = device.read(start, len, &mut buffer);

        let location = buffer.as_ptr();

        Self::new_shared(location, len)
    }

    fn find_entry(&self, path: impl AsRef<Path>) -> Option<&TableEntry> {
        let path = path.as_ref();
        self.header
            .entries
            .iter()
            .find(|entry| self.header.get_path(entry.path_index) == Some(path))
    }
}

impl<S: StorageDevice> fs::FileSystem for Initrd<S> {
    fn open_file(&self, path: &fs::Path) -> bool {
        let entry = self.find_entry(path);

        entry.is_some()
    }

    fn read_file(&self, file: &fs::File, buffer: &mut [u8]) -> usize {
        let entry = self.find_entry(file.path().path().unwrap());

        if entry.is_none() {
            return 0;
        };

        let entry = entry.unwrap();
        let start_addr = entry.offset + file.offset();

        let end_addr = start_addr + entry.len;
        let to_read = (end_addr - start_addr).min(buffer.len());

        unsafe {
            core::ptr::copy(
                self.data.as_ptr().add(start_addr),
                buffer.as_mut_ptr(),
                to_read,
            );
        }

        to_read
    }
}
