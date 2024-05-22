mod heap_allocator;

use core::sync::atomic::{AtomicBool, Ordering};

use multiboot::Module;

use crate::BootInfo;

/// Initialises memory
pub fn init(_bootinfo: &BootInfo, _initrd: &Module) {
    static INIT_CALLED: AtomicBool = AtomicBool::new(false);

    if INIT_CALLED.swap(true, Ordering::Relaxed) {
        panic!("memory::init must only be called once")
    }

    log::info!("initialising memory");

    log::info!("memory initialised");
}
