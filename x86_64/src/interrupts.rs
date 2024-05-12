mod interrupts {
    use core::arch::asm;

    use crate::registers::CpuFlags;

    pub fn is_enabled() -> bool {
        CpuFlags::read().contains(CpuFlags::INTERRUPT_FLAG)
    }

    pub fn enable() {
        unsafe {
            asm!("sti", options(nostack, preserves_flags));
        }
    }

    pub fn disable() {
        unsafe {
            asm!("cli", options(nostack, preserves_flags));
        }
    }
}

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let saved_input_flag = interrupts::is_enabled();

    if saved_input_flag {
        interrupts::disable();
    }

    let ret = f();

    if saved_input_flag {
        interrupts::enable();
    }

    ret
}
