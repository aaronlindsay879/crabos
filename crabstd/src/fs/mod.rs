use alloc::{borrow::ToOwned, string::String};
use core::{borrow::Borrow, ops::Deref};

use super::syscall;

#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Path {
    path: str,
}

impl Path {
    pub fn new(path: &str) -> &Self {
        unsafe { &*(path as *const str as *const Path) }
    }

    pub fn open(&self) -> Option<File> {
        syscall::open(self)
    }

    pub fn device_path(&self) -> Option<(&str, &str)> {
        self.split_once("//")
    }

    pub fn device(&self) -> Option<&str> {
        self.device_path().unzip().0
    }

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

    pub fn read(&mut self, buffer: &mut [u8]) -> usize {
        syscall::read(self, buffer)
    }

    pub fn path(&self) -> &Path {
        self.path.borrow()
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub trait FileSystem {
    /// Performs any operations needed to open the file,
    /// and then returns a bool indicated if the file exists.
    fn open_file(&self, path: &Path) -> bool;

    /// Reads the given file into the given buffer, returning number of bytes read.
    fn read_file(&self, file: &File, buffer: &mut [u8]) -> usize;
}

pub trait StorageDevice {
    fn read(&mut self, start: usize, count: usize, buf: &mut [u8]) -> usize;
}
