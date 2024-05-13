use core::arch::asm;

use crate::registers::CpuFlags;

pub fn are_interrupts_enabled() -> bool {
    CpuFlags::read().contains(CpuFlags::INTERRUPT_FLAG)
}

pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nostack, preserves_flags));
    }
}

pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nostack, preserves_flags));
    }
}

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let saved_input_flag = are_interrupts_enabled();

    if saved_input_flag {
        disable_interrupts();
    }

    let ret = f();

    if saved_input_flag {
        enable_interrupts();
    }

    ret
}
