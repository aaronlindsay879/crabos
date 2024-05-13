use core::{arch::asm, fmt::Display};

use bitflags::bitflags;

use crate::{structures::Frame, VirtualAddress};

pub struct CR3;

impl CR3 {
    /// Reads the frame and flags from CR3 register
    pub fn read() -> (Frame, u16) {
        let val: u64;

        unsafe {
            asm!("mov {}, cr3", out(reg) val, options(nostack, preserves_flags));
        }

        let addr = val & 0x_000F_FFFF_FFFF_F000;
        let frame = Frame::containing_address(addr as usize);

        (frame, (val & 0xFFF) as u16)
    }

    /// Writes the provided frame and flags to CR3 register
    ///
    /// # Safety
    /// `frame` and `flags` must be valid to write to `CR3`.
    pub unsafe fn write(frame: Frame, flags: u16) {
        let addr = frame.start_address();
        let val = addr as u64 | flags as u64;

        unsafe {
            asm!("mov cr3, {}", in(reg) val, options(nostack, preserves_flags));
        }
    }

    /// Invalidate the TLB by reloading the CR3 register
    pub fn flush_tlb() {
        unsafe {
            asm!(
                "mov {temp:r}, cr3",
                "mov cr3, {temp:r}",
                temp = out(reg) _,
                options(nostack, preserves_flags)
            )
        }
    }
}

pub struct CR2;

impl CR2 {
    pub fn read() -> VirtualAddress {
        let value: u64;

        unsafe {
            asm!("mov {}, cr2", out(reg) value, options(nostack, preserves_flags));
        }

        value as VirtualAddress
    }
}
bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
    pub struct CpuFlags: u64 {
        const ID = 1 << 21;
        const VIRTUAL_INTERRUPT_PENDING = 1 << 20;
        const VIRTUAL_INTERRUPT = 1 << 19;
        const ALIGNMENT_CHECK = 1 << 18;
        const VIRTUAL_8086_MODE = 1 << 17;
        const RESUME_FLAG = 1 << 16;
        const NESTED_TASK = 1 << 14;
        const IOPL_HIGH = 1 << 13;
        const IOPL_LOW = 1 << 12;
        const OVERFLOW_FLAG = 1 << 11;
        const DIRECTION_FLAG = 1 << 10;
        const INTERRUPT_FLAG = 1 << 9;
        const TRAP_FLAG = 1 << 8;
        const SIGN_FLAG = 1 << 7;
        const ZERO_FLAG = 1 << 6;
        const AUXILIARY_CARRY_FLAG = 1 << 4;
        const PARITY_FLAG = 1 << 2;
        const CARRY_FLAG = 1;
    }
}

impl CpuFlags {
    pub fn read() -> Self {
        let r: u64;

        unsafe {
            asm!("pushfq; pop {}", out(reg) r, options(nomem, preserves_flags));
        }

        Self::from_bits_truncate(r)
    }
}

impl Display for CpuFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for string in self.iter_names().map(|(a, _)| a).intersperse(" | ") {
            f.write_str(string)?;
        }

        Ok(())
    }
}
