use alloc::{borrow::ToOwned, string::String};
use core::{borrow::Borrow, ops::Deref};

use super::syscall;

/// Wrapper type for [str] which represents a path of the form
/// {device}//{path}, where {device} can be omitted to mean default device
#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Path {
    path: str,
}

impl Path {
    /// Constructs path from a given str
    pub fn new(path: &str) -> &Self {
        unsafe { &*(path as *const str as *const Path) }
    }

    /// Opens the file at the given path
    pub fn open(&self) -> Option<File> {
        syscall::open(self)
    }

    /// Returns a tuple of device and path
    pub fn device_path(&self) -> Option<(&str, &str)> {
        self.split_once("//")
    }

    /// Returns the device section of path
    pub fn device(&self) -> Option<&str> {
        self.device_path().unzip().0
    }

    /// Returns the path section of path
    pub fn path(&self) -> Option<&str> {
        self.device_path().unzip().1
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl AsRef<Path> for &str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl ToOwned for Path {
    type Owned = PathBuf;

    fn to_owned(&self) -> Self::Owned {
        PathBuf {
            path: self.path.to_owned(),
        }
    }
}

/// Owned version of [Path], uses [String] instead of [str]
#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub struct PathBuf {
    path: String,
}

impl Borrow<Path> for PathBuf {
    fn borrow(&self) -> &Path {
        Path::new(&self.path)
    }
}

#[derive(Debug)]
pub struct File {
    /// Current offset into the file
    offset: usize,
    /// File path
    path: PathBuf,
}

impl File {
    /// Constructs a new file from the given path, returning None if the file does not exist.
    pub fn new(path: impl AsRef<Path>) -> Option<Self> {
        path.as_ref().open()
    }

    /// Creates a file struct without checking that it exists on the system.
    ///
    /// # Safety
    /// File must exist before this function is called
    pub unsafe fn new_unchecked(path: impl AsRef<Path>) -> Self {
        Self {
            offset: 0,
            path: path.as_ref().to_owned(),
        }
    }

    /// Reads the file into the given buffer, returning the number of bytes read
    pub fn read(&mut self, buffer: &mut [u8]) -> usize {
        syscall::read(self, buffer)
    }

    /// Gets path of the file.
    pub fn path(&self) -> &Path {
        self.path.borrow()
    }

    /// Gets current offset into the file.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

/// Trait representing an arbitrary file system, such as initrd or ext4.
pub trait FileSystem {
    /// Performs any operations needed to open the file,
    /// and then returns a bool indicated if the file exists.
    fn open_file(&self, path: &Path) -> bool;

    /// Reads the given file into the given buffer, returning number of bytes read.
    fn read_file(&self, file: &File, buffer: &mut [u8]) -> usize;
}

/// Trait representing an arbitrary storage device, such as ram or AHCI.
pub trait StorageDevice {
    /// Reads `count` bytes from the given `start` address into the provided buffer,
    /// returning number of bytes read.
    fn read(&mut self, start: usize, count: usize, buf: &mut [u8]) -> usize;
}
