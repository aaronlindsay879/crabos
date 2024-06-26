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
    naked_functions,
    array_windows
)]

extern crate alloc;

use alloc::string::{String, ToString};
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

use crabstd::{fs::File, mutex::Mutex};
use initrd::Initrd;
use kernel_shared::{
    logger::Logger,
    memory::{frame_alloc::bitmap::BitmapFrameAllocator, paging::active_table::ActivePageTable},
    serial_println,
};
use ram::Ram;

use crate::io::{Writer, WRITER};

mod gdt;
mod interrupts;
mod io;
mod memory;

static LOGGER: Logger = Logger::new(log::LevelFilter::Trace);

pub const MODULE_COUNT: usize = 4;
pub type BootInfo = multiboot::BootInfo<MODULE_COUNT>;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    log::error!("{}", info);
    println!("err: {}", info);

    x86_64::hlt_loop()
}

static RAMFS: Mutex<Option<Initrd<Ram>>> = Mutex::new(None);

// needed for false positive on `BootInfo::new`
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn kernel_main(addr: *const u32, loader_start: usize, loader_end: usize) {
    // bootinfo is only valid for this scope
    let (
        InitInfo {
            mut frame_alloc,
            mut active_table,
            initrd_range: (initrd_start, initrd_end),
        },
        bootinfo_start,
        bootinfo_end,
    ) = {
        let bootinfo = unsafe { BootInfo::new(addr) };

        (
            init(&bootinfo, loader_start, loader_end),
            addr as usize,
            addr as usize + bootinfo.total_size,
        )
    };

    unsafe {
        memory::free_region(
            &mut active_table,
            &mut frame_alloc,
            bootinfo_start,
            bootinfo_end,
        )
    }

    fn read_file(path: &str) -> Option<String> {
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
        "reading file `notramfs//nope`:\n{:?}\n",
        read_file("notramfs//nope")
    );

    println!(
        "reading file `ramfs//big`:\n{:?}\n",
        read_file("ramfs//big")
    );

    // finally free initrd info to remove all mappings in user-space
    unsafe {
        memory::free_region(
            &mut active_table,
            &mut frame_alloc,
            initrd_start,
            initrd_end,
        )
    }

    x86_64::hlt_loop();
}

/// Struct representing information returned by [init]
struct InitInfo {
    frame_alloc: BitmapFrameAllocator,
    active_table: ActivePageTable,
    initrd_range: (usize, usize),
}

/// Initialises everything required for kernel
fn init(bootinfo: &BootInfo, loader_start: usize, loader_end: usize) -> InitInfo {
    static INIT_CALLED: AtomicBool = AtomicBool::new(false);

    if INIT_CALLED.swap(true, Ordering::Relaxed) {
        panic!("init must only be called once")
    }

    LOGGER.init().expect("failed to init logger");
    log::trace!("logger initialised");

    let initrd = bootinfo
        .modules
        .iter()
        .flatten()
        .find(|module| module.string == c"initrd")
        .expect("no initrd module");
    log::trace!(
        "initrd module found in range {:#X}-{:#X}",
        initrd.start,
        initrd.end
    );

    let (frame_alloc, active_table) = memory::init(bootinfo, loader_start, loader_end);

    log::trace!("initialising stdio");
    *WRITER.lock().get_mut() =
        Some(Writer::from_bootinfo(bootinfo).expect("invalid framebuffer type"));
    log::trace!("stdio initialised");

    *RAMFS.lock() =
        unsafe { Initrd::new_ram(initrd.start as usize, (initrd.end - initrd.start) as usize) };

    if RAMFS.lock().is_none() {
        panic!("no ramfs driver loaded");
    }
    log::trace!("ramfs initialised");

    gdt::init();
    interrupts::init();

    log::trace!("kernel initialised");

    InitInfo {
        frame_alloc,
        active_table,
        initrd_range: (initrd.start as usize, initrd.end as usize),
    }
}
