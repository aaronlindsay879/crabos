use core::arch::asm;

use crate::PrivilegeLevel;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub fn read_cs() -> SegmentSelector {
        let val: u16;

        unsafe {
            asm!("mov {:x}, cs", out(reg) val, options(nostack, preserves_flags));
        }

        SegmentSelector(val)
    }

    /// # Safety
    /// `self` must be a valid segment selector to write to `CS`
    pub unsafe fn write_cs(&self) {
        unsafe {
            asm!(
                "push {sel}",
                "lea {tmp}, [1f + rip]",
                "push {tmp}",
                "retfq",
                "1:",
                sel = in(reg) u64::from(self.0),
                tmp = lateout(reg) _,
                options(preserves_flags),
            );
        }
    }

    /// # Safety
    /// `self` must be a valid segment selector to write to `SS`
    pub unsafe fn write_ss(&self) {
        unsafe {
            asm!("mov ss, {:x}", in(reg) self.0);
        }
    }

    /// # Safety
    /// `self` must be a valid segment selector to load as tss.
    pub unsafe fn load_tss(&self) {
        unsafe {
            asm!("ltr {0:x}", in(reg) self.0, options(nostack, preserves_flags));
        }
    }

    pub const fn new(index: u16, rpl: PrivilegeLevel) -> Self {
        Self(index << 3 | (rpl as u16))
    }

    /// Returns the GDT index
    pub const fn index(&self) -> u16 {
        self.0 >> 3
    }

    /// Returns the requested privilege level
    pub const fn rpl(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0 & 0b11)
    }
}
