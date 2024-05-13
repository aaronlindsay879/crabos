use core::sync::atomic::AtomicU64;

use bit_field::BitField;
use bitflags::bitflags;

use super::TaskStateSegment;
use crate::{segment_selector::SegmentSelector, IntoDescriptorTable, PrivilegeLevel};

#[repr(transparent)]
pub struct GdtEntry(AtomicU64);

impl GdtEntry {
    pub const fn new(val: u64) -> Self {
        Self(AtomicU64::new(val))
    }
}

pub struct GlobalDescriptorTable {
    pub(crate) table: [GdtEntry; 8],
    pub(crate) len: usize,
}

impl Default for GlobalDescriptorTable {
    fn default() -> Self {
        Self {
            table: [const { GdtEntry::new(0) }; 8],
            len: 1,
        }
    }
}

impl GlobalDescriptorTable {
    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => {
                if self.len > 7 {
                    panic!("GDT full");
                }

                self.push(value)
            }
            Descriptor::SystemSegment(value_low, value_high) => {
                if self.len > 6 {
                    panic!("GDT full");
                }

                let index = self.push(value_low);
                self.push(value_high);

                index
            }
        };

        SegmentSelector::new(index as u16, entry.dpl())
    }

    fn push(&mut self, value: u64) -> usize {
        let index = self.len;
        self.table[index] = GdtEntry::new(value);
        self.len += 1;

        index
    }

    pub fn load(&'static self) {
        let dtr = self.as_dtr();

        unsafe {
            dtr.load_gdt();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

bitflags! {
    /// Flags for a GDT descriptor. Not all flags are valid for all descriptor types.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct DescriptorFlags: u64 {
        const ACCESSED          = 1 << 40;
        const WRITABLE          = 1 << 41;
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const USER_SEGMENT      = 1 << 44;
        const DPL_RING_3        = 3 << 45;
        const PRESENT           = 1 << 47;
        const AVAILABLE         = 1 << 52;
        const LONG_MODE         = 1 << 53;
        const DEFAULT_SIZE      = 1 << 54;
        const GRANULARITY       = 1 << 55;

        const LIMIT_0_15        = 0xFFFF;
        const LIMIT_16_19       = 0xF << 48;
        const BASE_0_23         = 0xFF_FFFF << 16;
        const BASE_24_31        = 0xFF << 56;
    }
}

impl DescriptorFlags {
    const COMMON: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::LIMIT_0_15.bits()
            | Self::LIMIT_16_19.bits()
            | Self::GRANULARITY.bits(),
    );

    pub const KERNEL_DATA: Self =
        Self::from_bits_truncate(Self::COMMON.bits() | Self::DEFAULT_SIZE.bits());
    pub const KERNEL_CODE32: Self = Self::from_bits_truncate(
        Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::DEFAULT_SIZE.bits(),
    );
    pub const KERNEL_CODE64: Self = Self::from_bits_truncate(
        Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::LONG_MODE.bits(),
    );

    pub const USER_DATA: Self =
        Self::from_bits_truncate(Self::KERNEL_DATA.bits() | Self::DPL_RING_3.bits());
    pub const USER_CODE32: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE32.bits() | Self::DPL_RING_3.bits());
    pub const USER_CODE64: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE64.bits() | Self::DPL_RING_3.bits());
}

impl Descriptor {
    pub fn dpl(&self) -> PrivilegeLevel {
        let value_low = match self {
            Descriptor::UserSegment(v) => v,
            Descriptor::SystemSegment(v, _) => v,
        };

        let dpl = (value_low & DescriptorFlags::DPL_RING_3.bits()) >> 45;
        PrivilegeLevel::from_u16(dpl as u16)
    }

    pub fn kernel_code_segment() -> Self {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_CODE64.bits())
    }

    pub const fn kernel_data_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_DATA.bits())
    }

    pub fn tss_segment(tss: &'static TaskStateSegment) -> Self {
        let tss = tss as *const TaskStateSegment;
        let ptr = tss as u64;

        let mut low = DescriptorFlags::PRESENT.bits();

        low.set_bits(0..16, (core::mem::size_of::<TaskStateSegment>() - 1) as u64);
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(40..44, 0b1001);
        low.set_bits(56..64, ptr.get_bits(24..32));

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Self::SystemSegment(low, high)
    }
}
