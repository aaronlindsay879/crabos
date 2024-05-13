use core::arch::asm;

use crate::fs::{File, Path};

/// Index of `no_function` syscall
pub const NO_FUNCTION: usize = 0;
/// Index of `open` syscall
pub const OPEN: usize = 1;
/// Index of `read` syscall
pub const READ: usize = 2;

/// Helper macro to generate a syscall with the provided opcode and registers,
/// making sure to pass arguments in the correct registers
macro_rules! syscall {
    ($opcode:expr) => {
        asm!("int 0x80", in("rax") $opcode)
    };
    ($opcode:expr; $arg1:expr) => {
        asm!(
            "int 0x80",
            in("rax") $opcode,
            in("rdi") $arg1
        )
    };
    ($opcode:expr; $arg1:expr, $arg2:expr) => {
        asm!(
            "int 0x80",
            in("rax") $opcode,
            in("rdi") $arg1,
            in("rsi") $arg2,
        )
    };
    ($opcode:expr; $arg1:expr, $arg2:expr, $arg3:expr) => {
        asm!(
            "int 0x80",
            in("rax") $opcode,
            in("rdi") $arg1,
            in("rsi") $arg2,
            in("rdx") $arg3,
        )
    };
    ($opcode:expr; $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
        asm!(
            "int 0x80",
            in("rax") $opcode,
            in("rdi") $arg1,
            in("rsi") $arg2,
            in("rdx") $arg3,
            in("rcx") $arg4,
        )
    };
}

/// Performs an `open` syscall, opening a file with the given path.
pub fn open(path: &Path) -> Option<File> {
    let mut is_valid = false;
    unsafe {
        syscall!(OPEN;
            path.as_ptr() as usize,
            path.len(),
            &mut is_valid
        );
    }

    if is_valid {
        Some(unsafe { File::new_unchecked(path) })
    } else {
        None
    }
}

/// Performs a `read` syscall, reading data from the given file to the provided buffer,
/// returning the number of bytes read.
pub fn read(file: &mut File, buffer: &mut [u8]) -> usize {
    let mut bytes_read: usize = 0;

    unsafe {
        syscall!(READ;
            file as *mut _ as usize,
            buffer.as_ptr() as usize,
            buffer.len(),
            &mut bytes_read
        );
    }

    bytes_read
}
