use core::{arch::asm, ops::Deref};

use crabstd::{
    fs::{File, FileSystem, Path},
    syscall as syscalls,
};

macro_rules! syscall {
    ($arg1:expr) => {
        asm!(
            "",
            out("rdi") $arg1,
        )
    };
    ($arg1:expr, $arg2:expr) => {
        asm!(
            "",
            out("rdi") $arg1,
            out("rsi") $arg2,
        )
    };
    ($arg1:expr, $arg2:expr, $arg3:expr) => {
        asm!(
            "",
            out("rdi") $arg1,
            out("rsi") $arg2,
            out("rdx") $arg3,
        )
    };
    ($arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
        asm!(
            "",
            out("rdi") $arg1,
            out("rsi") $arg2,
            out("rdx") $arg3,
            out("rcx") $arg4,
        )
    };
}

macro_rules! syscall_table {
    ($($number:expr => $function:ident$(,)?)*) => {{
        let mut table = [unsafe { &*(no_function as *const ()) }; 256];

        $(
            table[$number] = unsafe { &*($function as *const ()) };
        )*

        table
    }};
}

#[used]
#[link_section = ".syscall_table"]
pub(super) static SYSCALL_TABLE: [&(); 256] = syscall_table!(
    syscalls::NO_FUNCTION => no_function,
    syscalls::OPEN => open,
    syscalls::READ => read,
);

#[no_mangle]
extern "x86-interrupt" fn no_function() {
    let rax: usize;
    unsafe {
        asm!("mov {}, rax", out(reg) rax);
    }

    log::info!("syscall called with incorrect operand `{rax}`");
}

#[no_mangle]
extern "x86-interrupt" fn read() {
    let file: *const File;
    let buffer: *mut u8;
    let buffer_len: usize;
    let bytes_read: *mut usize;

    unsafe {
        syscall!(file, buffer, buffer_len, bytes_read);
    }

    let file = unsafe { &*file };
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, buffer_len) };

    log::info!("read syscall called");
    log::trace!("\t* file: {file:?}");
    log::trace!(
        "\t* buffer addr: {:#X}, buffer len: {:#X}",
        buffer.as_ptr() as usize,
        buffer.len()
    );

    let device = file.path().device().unwrap();

    let written = match device {
        "ramfs" => crate::RAMFS
            .lock()
            .as_ref()
            .unwrap()
            .read_file(file, buffer),
        _ => {
            log::warn!(
                "attempted to read from invalid device `{}`, path `{}`",
                device,
                file.path().deref()
            );
            0
        }
    };

    unsafe {
        *bytes_read = written;
    }
}

#[no_mangle]
extern "x86-interrupt" fn open() {
    let path: *const u8;
    let path_len: usize;
    let is_valid: *mut bool;

    unsafe {
        syscall!(path, path_len, is_valid);
    }

    let path = Path::new(unsafe { core::str::from_raw_parts(path, path_len) });
    log::info!("open syscall called");
    log::trace!("\t* path: {path:?}");

    let (device, path) = path.device_path().unwrap();

    let driver_response = match device {
        "ramfs" => crate::RAMFS
            .lock()
            .as_ref()
            .unwrap()
            .open_file(Path::new(path)),
        _ => {
            log::warn!(
                "attempted to open path `{}` on invalid device `{}`",
                path,
                device,
            );
            false
        }
    };

    unsafe {
        *is_valid = driver_response;
    }
}
