#![no_std]
#![feature(iter_intersperse, abi_x86_interrupt)]

use core::{arch::asm, marker::PhantomData};

use structures::{GlobalDescriptorTable, InterruptDescriptorTable, PAGE_SIZE};

pub mod interrupts;
pub mod port;
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

/// Align downwards - returns the greatest _x_ with alignment `align`
/// such that _x_ <= addr. `align` must be power of 2
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be power of two")
    }
}

/// Align downwards - returns the greatest _x_ with alignment of page size
/// such that _x_ <= addr. `align` must be power of 2
pub fn align_down_to_page(addr: usize) -> usize {
    align_down(addr, PAGE_SIZE)
}

/// Align upwards - returns the smallest _x_ with alignment `align`
/// such that _x_ >= addr. `align` must be power of 2
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

/// Align upwards - returns the smallest _x_ with alignment of page size
/// such that _x_ >= addr. `align` must be power of 2
pub fn align_up_to_page(addr: usize) -> usize {
    align_up(addr, PAGE_SIZE)
}
