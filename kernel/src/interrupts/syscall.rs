use core::arch::asm;

use crabstd::{
    fs::{File, FileSystem, Path},
    syscall as syscalls,
};

use crate::println;

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
    ($($number:expr => $function:ident$(,)?)*) => {
        #[used]
        #[no_mangle]
        static SYSCALL_TABLE: [&'static (); 256] = {
            let mut table = [unsafe { &*(no_function as *const ()) }; 256];

            $(
                table[$number] = unsafe { &*($function as *const ()) };
            )*

            table
        };
    };
}

syscall_table!(
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

    println!("syscall called with invalid code {rax}");
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

    let device = file.path().device().unwrap();

    let written = match device {
        "ramfs" => crate::RAMFS
            .lock()
            .as_ref()
            .unwrap()
            .read_file(file, buffer),
        _ => {
            println!("unknown device!");
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
    let (device, path) = path.device_path().unwrap();

    let driver_response = match device {
        "ramfs" => crate::RAMFS
            .lock()
            .as_ref()
            .unwrap()
            .open_file(Path::new(path)),
        _ => {
            println!("unknown device!");
            false
        }
    };

    unsafe {
        *is_valid = driver_response;
    }
}
