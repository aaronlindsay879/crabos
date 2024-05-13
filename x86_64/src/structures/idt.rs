use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use bit_field::BitField;

use super::ExceptionStackFrame;
use crate::{segment_selector::SegmentSelector, IntoDescriptorTable, VirtualAddress};

pub type HandlerFunc = extern "x86-interrupt" fn(_: ExceptionStackFrame);
pub type HandlerFuncError = extern "x86-interrupt" fn(_: ExceptionStackFrame, _: u64);
pub type DivergingHandlerFunc = extern "x86-interrupt" fn(_: ExceptionStackFrame) -> !;
pub type DivergingHandlerFuncError = extern "x86-interrupt" fn(_: ExceptionStackFrame, _: u64) -> !;

#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_error: IdtEntry<HandlerFunc>,
    pub debug: IdtEntry<HandlerFunc>,
    pub non_maskable_interrupt: IdtEntry<HandlerFunc>,
    pub breakpoint: IdtEntry<HandlerFunc>,
    pub overflow: IdtEntry<HandlerFunc>,
    pub bound_range_exceeded: IdtEntry<HandlerFunc>,
    pub invalid_opcode: IdtEntry<HandlerFunc>,
    pub device_not_available: IdtEntry<HandlerFunc>,
    pub double_fault: IdtEntry<DivergingHandlerFuncError>,
    coprocessor_segment_overrun: IdtEntry<HandlerFunc>,
    pub invalid_tss: IdtEntry<HandlerFuncError>,
    pub segment_not_present: IdtEntry<HandlerFuncError>,
    pub stack_segment_fault: IdtEntry<HandlerFuncError>,
    pub general_protection_fault: IdtEntry<HandlerFuncError>,
    pub page_fault: IdtEntry<HandlerFuncError>,
    reserved_1: IdtEntry<HandlerFunc>,
    pub x87_floating_point: IdtEntry<HandlerFunc>,
    pub alignment_check: IdtEntry<HandlerFuncError>,
    pub machine_check: IdtEntry<DivergingHandlerFunc>,
    pub simd_floating_point: IdtEntry<HandlerFunc>,
    pub virtualization: IdtEntry<HandlerFunc>,
    pub cp_protection_exception: IdtEntry<HandlerFuncError>,
    reserved_2: [IdtEntry<HandlerFunc>; 6],
    pub hv_injection_exception: IdtEntry<HandlerFunc>,
    pub vmm_communication_exception: IdtEntry<HandlerFuncError>,
    pub security_exception: IdtEntry<HandlerFuncError>,
    reserved_3: IdtEntry<HandlerFunc>,

    interrupts: [IdtEntry<HandlerFunc>; 256 - 32],
}

impl Default for InterruptDescriptorTable {
    fn default() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            divide_error: IdtEntry::missing(),
            debug: IdtEntry::missing(),
            non_maskable_interrupt: IdtEntry::missing(),
            breakpoint: IdtEntry::missing(),
            overflow: IdtEntry::missing(),
            bound_range_exceeded: IdtEntry::missing(),
            invalid_opcode: IdtEntry::missing(),
            device_not_available: IdtEntry::missing(),
            double_fault: IdtEntry::missing(),
            coprocessor_segment_overrun: IdtEntry::missing(),
            invalid_tss: IdtEntry::missing(),
            segment_not_present: IdtEntry::missing(),
            stack_segment_fault: IdtEntry::missing(),
            general_protection_fault: IdtEntry::missing(),
            page_fault: IdtEntry::missing(),
            reserved_1: IdtEntry::missing(),
            x87_floating_point: IdtEntry::missing(),
            alignment_check: IdtEntry::missing(),
            machine_check: IdtEntry::missing(),
            simd_floating_point: IdtEntry::missing(),
            virtualization: IdtEntry::missing(),
            cp_protection_exception: IdtEntry::missing(),
            reserved_2: [IdtEntry::missing(); 6],
            hv_injection_exception: IdtEntry::missing(),
            vmm_communication_exception: IdtEntry::missing(),
            security_exception: IdtEntry::missing(),
            reserved_3: IdtEntry::missing(),
            interrupts: [IdtEntry::missing(); 256 - 32],
        }
    }
}

impl InterruptDescriptorTable {
    pub fn load(&'static self) {
        let ptr = self.as_dtr();

        unsafe {
            ptr.load_idt();
        }
    }
}

impl Index<u8> for InterruptDescriptorTable {
    type Output = IdtEntry<HandlerFunc>;

    #[inline]
    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.divide_error,
            1 => &self.debug,
            2 => &self.non_maskable_interrupt,
            3 => &self.breakpoint,
            4 => &self.overflow,
            5 => &self.bound_range_exceeded,
            6 => &self.invalid_opcode,
            7 => &self.device_not_available,
            9 => &self.coprocessor_segment_overrun,
            16 => &self.x87_floating_point,
            19 => &self.simd_floating_point,
            20 => &self.virtualization,
            28 => &self.hv_injection_exception,
            i @ 32..=255 => &self.interrupts[usize::from(i) - 32],
            i @ 15 | i @ 31 | i @ 22..=27 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 21 | i @ 29 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i @ 18 => panic!("entry {} is an diverging exception (must not return)", i),
        }
    }
}

impl IndexMut<u8> for InterruptDescriptorTable {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.divide_error,
            1 => &mut self.debug,
            2 => &mut self.non_maskable_interrupt,
            3 => &mut self.breakpoint,
            4 => &mut self.overflow,
            5 => &mut self.bound_range_exceeded,
            6 => &mut self.invalid_opcode,
            7 => &mut self.device_not_available,
            9 => &mut self.coprocessor_segment_overrun,
            16 => &mut self.x87_floating_point,
            19 => &mut self.simd_floating_point,
            20 => &mut self.virtualization,
            28 => &mut self.hv_injection_exception,
            i @ 32..=255 => &mut self.interrupts[usize::from(i) - 32],
            i @ 15 | i @ 31 | i @ 22..=27 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 21 | i @ 29 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i @ 18 => panic!("entry {} is an diverging exception (must not return)", i),
        }
    }
}

/// # Safety
/// Implementors have to ensure that to_virt_addr returns a valid address.
pub unsafe trait HandlerFuncType {
    /// Get the virtual address of the handler function.
    fn to_virt_addr(self) -> VirtualAddress;
}

unsafe impl HandlerFuncType for HandlerFunc {
    fn to_virt_addr(self) -> VirtualAddress {
        self as VirtualAddress
    }
}

unsafe impl HandlerFuncType for HandlerFuncError {
    fn to_virt_addr(self) -> VirtualAddress {
        self as VirtualAddress
    }
}
unsafe impl HandlerFuncType for DivergingHandlerFunc {
    fn to_virt_addr(self) -> VirtualAddress {
        self as VirtualAddress
    }
}
unsafe impl HandlerFuncType for DivergingHandlerFuncError {
    fn to_virt_addr(self) -> VirtualAddress {
        self as VirtualAddress
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IdtEntry<F> {
    low_fn_pointer: u16,
    options: EntryOptions,
    middle_fn_pointer: u16,
    high_fn_pointer: u32,
    _reserved: u32,
    phantom: PhantomData<F>,
}

impl<F: HandlerFuncType> IdtEntry<F> {
    fn missing() -> Self {
        Self {
            low_fn_pointer: 0,
            middle_fn_pointer: 0,
            high_fn_pointer: 0,
            options: EntryOptions::minimal(),
            _reserved: 0,
            phantom: PhantomData {},
        }
    }

    pub fn set(&mut self, handler: F) -> &mut EntryOptions {
        let pointer = handler.to_virt_addr();

        self.low_fn_pointer = pointer as u16;
        self.middle_fn_pointer = (pointer >> 16) as u16;
        self.high_fn_pointer = (pointer >> 32) as u32;

        unsafe {
            self.options.set_gdt_selector(SegmentSelector::read_cs());
        }
        self.options.set_present(true);

        &mut self.options
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EntryOptions {
    cs: SegmentSelector,
    bits: u16,
}

impl Default for EntryOptions {
    /// Constructs options with reasonable defaults (present = true, gate = true)
    fn default() -> Self {
        *Self::minimal().set_present(true)
    }
}

#[allow(unused)]
impl EntryOptions {
    /// Constructs options with minimal options set (only 'must be one' bits)
    pub fn minimal() -> Self {
        Self {
            cs: SegmentSelector(0),
            bits: 0b1110_0000_0000,
        }
    }

    /// # Safety
    /// The passed segment selector must point to a valid, long-mode code segment.
    pub unsafe fn set_gdt_selector(&mut self, selector: SegmentSelector) -> &mut Self {
        self.cs = selector;
        self
    }

    pub fn ist_index(&self) -> u16 {
        self.bits & 0x7
    }

    /// # Safety
    /// The passed stack index must be valid and not used by any other interrupts.
    pub unsafe fn set_ist_index(&mut self, index: u16) -> &mut Self {
        self.bits.set_bits(0..3, index + 1);

        self
    }

    pub fn gate(&self) -> bool {
        (self.bits >> 7) & 1 == 1
    }

    pub fn set_gate(&mut self, gate: bool) -> &mut Self {
        let gate_bit = if gate { 1 } else { 0 };
        self.bits = (self.bits & 0xFF7F) | (gate_bit << 8);

        self
    }

    pub fn privilege_level(&self) -> u16 {
        self.bits & 0x6000
    }

    pub fn set_privilege_level(&mut self, privilege_level: u16) -> &mut Self {
        self.bits = (self.bits & 0x9FFF) | ((privilege_level & 0b11) << 13);

        self
    }

    pub fn present(&self) -> bool {
        (self.bits >> 15) & 1 == 1
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.bits.set_bit(15, present);

        self
    }
}
