#![no_std]
#![feature(iter_intersperse, abi_x86_interrupt)]

use core::{arch::asm, marker::PhantomData};

use structures::{GlobalDescriptorTable, InterruptDescriptorTable};

pub mod interrupts;
pub mod io;
pub mod registers;
pub mod segment_selector;
pub mod structures;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1,
    Ring2,
    Ring3,
}

impl PrivilegeLevel {
    pub const fn from_u16(pl: u16) -> Self {
        match pl {
            0 => Self::Ring0,
            1 => Self::Ring1,
            2 => Self::Ring2,
            3 => Self::Ring3,
            _ => panic!("invalid privilege level"),
        }
    }
}

trait IntoDescriptorTable
where
    Self: Sized,
{
    fn as_dtr(&'static self) -> DescriptorTablePointer<Self>;
}

impl IntoDescriptorTable for InterruptDescriptorTable {
    fn as_dtr(&'static self) -> DescriptorTablePointer<Self> {
        DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (core::mem::size_of::<Self>()) as u16,
            phantom: PhantomData {},
        }
    }
}

impl IntoDescriptorTable for GlobalDescriptorTable {
    fn as_dtr(&'static self) -> DescriptorTablePointer<GlobalDescriptorTable> {
        DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.len * core::mem::size_of::<u64>() - 1) as u16,
            phantom: PhantomData {},
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer<T> {
    /// Size of the DT.
    limit: u16,
    /// Pointer to the memory region containing the DT.
    base: u64,
    phantom: PhantomData<T>,
}

impl DescriptorTablePointer<InterruptDescriptorTable> {
    /// Loads the given descriptor table as an interrupt descriptor table
    ///
    /// # Safety
    /// Descriptor table must point to a valid interrupt descriptor table
    pub unsafe fn load_idt(self) {
        unsafe {
            asm!("lidt [{}]", in(reg) &self, options(readonly, nostack, preserves_flags));
        }
    }
}

impl DescriptorTablePointer<GlobalDescriptorTable> {
    /// Loads the given descriptor table as an global descriptor table
    ///
    /// # Safety
    /// Descriptor table must point to a valid global descriptor table
    pub unsafe fn load_gdt(self) {
        unsafe {
            asm!("lgdt [{}]", in(reg) &self, options(readonly, nostack, preserves_flags));
        }
    }
}

/// Invalidates a given address in the TLB
pub fn invalidate_address(addr: VirtualAddress) {
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) addr as u64,
            options(nostack, preserves_flags)
        )
    }
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}
