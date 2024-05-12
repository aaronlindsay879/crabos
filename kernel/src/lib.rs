#![no_std]
#![feature(
    inline_const_pat,
    const_mut_refs,
    const_trait_impl,
    effects,
    allocator_api,
    abi_x86_interrupt,
    iter_intersperse,
    str_from_raw_parts,
    naked_functions
)]

extern crate alloc;

use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
};
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

use crabstd::{
    fs::{File, Path},
    mutex::Mutex,
};
use initrd::Initrd;
use multiboot::prelude::*;
use ram::Ram;

use crate::io::{Writer, WRITER};

mod gdt;
mod interrupts;
mod io;
mod memory;
mod serial;

pub const MODULE_COUNT: usize = 4;
pub type BootInfo = multiboot::BootInfo<MODULE_COUNT>;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("err: {}", info);
    println!("err: {}", info);

    x86_64::hlt_loop()
}

static RAMFS: Mutex<Option<Initrd<Ram>>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn kernel_main(addr: *const u32) {
    let bootinfo = unsafe { BootInfo::new(addr) };
    init(&bootinfo);

    fn read_file<'a>(path: &str) -> Option<String> {
        let mut buf = [0; 16384];
        let mut file = File::new(path)?;
        let bytes_read = file.read(&mut buf);

        Some(unsafe { core::str::from_utf8_unchecked(&buf[..bytes_read]).to_string() })
    }

    println!(
        "reading file `ramfs//test`:\n{:?}\n",
        read_file("ramfs//test")
    );

    println!(
        "reading file `ramfs//silly`:\n{:?}\n",
        read_file("ramfs//silly")
    );

    println!(
        "reading file `ramfs//nope`:\n{:?}\n",
        read_file("ramfs//nope")
    );

    println!(
        "reading file `ramfs//big`:\n{:?}\n",
        read_file("ramfs//big")
    );
}

/// Initialises everything required for kernel
fn init(bootinfo: &BootInfo) {
    static INIT_CALLED: AtomicBool = AtomicBool::new(false);

    if INIT_CALLED.swap(true, Ordering::Relaxed) {
        panic!("init must only be called once")
    }

    let initrd = bootinfo
        .modules
        .iter()
        .flatten()
        .find(|module| module.string == c"initrd")
        .expect("no initrd module");

    memory::init(&bootinfo, initrd);

    *WRITER.lock().get_mut() =
        Some(Writer::from_bootinfo(&bootinfo).expect("invalid framebuffer type"));

    *RAMFS.lock() = unsafe {
        Initrd::new_ram(
            initrd.start as usize | 0x00FF_FFFF_0000,
            (initrd.end - initrd.start) as usize,
        )
    };

    if RAMFS.lock().is_none() {
        let location = (initrd.start as usize | 0x00FF_FFFF_0000) as *const [u8; 4];
        println!("{:?}", unsafe { *location });
        println!("{:?}", *b"KTIY");
        panic!("no ramfs driver loaded");
    }

    gdt::init();
    interrupts::init();
}

multiboot_header! {
    arch: 0,
    tags: [
        InformationRequest {
            requests: &[ELF_SYMBOLS, MEMORY_MAP]
        },
        ConsoleFlags::all(),
        Framebuffer {
            width: Value(1920),
            height: Value(1080),
            depth: NoPreference
        },
    ]
}
