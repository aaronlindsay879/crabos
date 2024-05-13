#![no_std]

use alloc::vec;
use core::{ffi::CStr, marker::PhantomData, ops::Deref};

use crabstd::fs::{self, Path, StorageDevice};
use ram::Ram;

extern crate alloc;

/// File system for initial ramdisk.
///
/// ## Format
/// | Type          | Name              | Description                           |
/// |---------------|-------------------|---------------------------------------|
/// | u32           | Magic             | Magic number b"KTIY"                  |
/// | u32           | reserved          |                                       |
/// | u64           | header count      | number of header entries              |
/// | u64           | string table len  | length of string table                |
/// | \[TableEntry] | entries           | one header entry per file             |
/// | \[u8]         | string table      | one null-terminated string per file   |
/// | \[u8]         | data              | raw file data                         |
#[derive(Debug)]
pub struct Initrd<S: StorageDevice> {
    /// Header containing file information
    header: Header,
    /// Raw file data
    data: &'static [u8],
    _phantom: PhantomData<S>,
}

/// Header of initrd file system, contains file table entries and string table for file names.
#[derive(Debug)]
struct Header {
    /// Table of file information
    entries: &'static [TableEntry],
    /// Raw table of file names
    string_table: &'static [u8],
}

impl Header {
    /// Gets the path starting at a specific index in the string table
    pub fn get_path(&self, path_index: usize) -> Option<&Path> {
        CStr::from_bytes_until_nul(&self.string_table[path_index..])
            .ok()
            .and_then(|string| string.to_str().ok())
            .map(Path::new)
    }
}

/// Stores information about a single file
#[derive(Debug)]
#[repr(C)]
struct TableEntry {
    /// Index of start of path in the string table
    path_index: usize,
    /// Offset into data
    offset: usize,
    /// Length of file
    len: usize,
}

impl Initrd<Ram> {
    /// Specialised implementation for when initrd is in ram to avoid copying data to a buffer while reading.
    /// While those copies are needed for generic storage devices, there's no need to copy data if it's
    /// already in ram.
    ///
    /// # Safety
    /// Memory from `start` to `start + len` must be valid to read.
    pub unsafe fn new_ram(start: usize, len: usize) -> Option<Self> {
        log::trace!("constructing initrd with backing ram storage");
        Self::new_shared(start as *const _, len)
    }
}

impl<S: StorageDevice> Initrd<S> {
    /// Shared code for creating an initrd struct from various storage devices
    unsafe fn new_shared(location: *const u8, len: usize) -> Option<Self> {
        // u32 - magic number "KTIY"
        // u32 - reserved
        // u64 - header count
        // u64 - string table len
        // headers
        // string table
        // data

        // make sure actually reading initrd file
        let magic = *(location as *const [u8; 4]);
        if magic != *b"KTIY" {
            log::warn!("tried to load initrd module with incorrect magic value `{magic:?}`");
            return None;
        }

        // then read header count and string table length
        let header_count = *(location.add(8) as *const u64);
        let string_table_len = *(location.add(16) as *const u64);

        // current length of header
        let mut header_len = 24;

        // read entries based on header_count, and advance header length
        let entries = core::slice::from_raw_parts(
            location.add(header_len) as *const _,
            header_count as usize,
        );
        header_len += header_count as usize * core::mem::size_of::<TableEntry>();

        // then read string_table based on string table len, and advance header length
        let string_table =
            core::slice::from_raw_parts(location.add(header_len), string_table_len as usize);
        header_len += string_table_len as usize;

        // then construct header and read remaining data as raw file data
        let header = Header {
            entries,
            string_table,
        };

        let data = core::slice::from_raw_parts(location.add(header_len), len - header_len);

        Some(Self {
            header,
            data,
            _phantom: PhantomData {},
        })
    }

    /// Creates an initrd struct with generic storage device.
    ///
    /// This will perform copies that can be avoided with [Self::new_ram] if using ram as a storage device.
    pub fn new(start: usize, len: usize, mut device: S) -> Option<Self> {
        log::trace!(
            "constructing initrd with backing storage device `{}`",
            core::any::type_name::<S>()
        );

        let mut buffer = vec![0; len];
        let len = device.read(start, len, &mut buffer);

        let location = buffer.as_ptr();

        // location points to a buffer with correct length - definitely safe
        unsafe { Self::new_shared(location, len) }
    }

    /// Finds the table entry storing information about a specific file.
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
        log::trace!("attempting to open file `{}`", path.deref());

        // return if file exists in header table
        let found = self.find_entry(path).is_some();

        if found {
            log::trace!("\t* file at `{}` found", path.deref());
        } else {
            log::trace!("\t* file at `{}` not found", path.deref());
        }

        found
    }

    fn read_file(&self, file: &fs::File, buffer: &mut [u8]) -> usize {
        log::trace!("attempting to read file `{}`", file.path().deref());
        // firstly make sure entry exists, returning 0 if it doesn't
        let entry = {
            if let Some(entry) = self.find_entry(file.path().path().unwrap()) {
                entry
            } else {
                log::warn!("\t* trying to read file `{file:?}` which doesn't exist on file system");
                return 0;
            }
        };

        // find start and end address of file
        let start_addr = entry.offset + file.offset();

        log::trace!("\t* copying from {start_addr:#X} in initrd");

        // then find how much to read - length, capped by the buffer size
        let to_read = (entry.len).min(buffer.len());

        log::trace!(
            "\t* copying {to_read:#X} bytes to buffer at addr {:#X}",
            buffer.as_ptr() as usize
        );

        // finally copy to buffer and return
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
