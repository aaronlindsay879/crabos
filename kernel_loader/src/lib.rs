#![no_std]
#![feature(const_mut_refs)]

use core::panic::PanicInfo;

use multiboot::prelude::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86_64::hlt_loop()
}

#[no_mangle]
extern "C" fn loader_main(_addr: *const u32) {
    x86_64::hlt_loop();
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
