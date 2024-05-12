use core::fmt::Display;

use crate::{registers::CpuFlags, segment_selector::SegmentSelector};

#[derive(Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: SegmentSelector,
    _reserved1: [u8; 6],
    pub cpu_flags: CpuFlags,
    pub stack_pointer: u64,
    pub stack_segment: SegmentSelector,
    _reserved2: [u8; 6],
}

impl Display for ExceptionStackFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Exception stack frame:")?;
        writeln!(f, "\tInstruction pointer: {:#X}", self.instruction_pointer)?;
        writeln!(f, "\tCode segment: {:?}", self.code_segment)?;
        writeln!(f, "\tCpu flags: {}", self.cpu_flags)?;
        writeln!(f, "\tStack pointer: {:#X}", self.stack_pointer)?;
        writeln!(f, "\tStack segment: {:?}", self.stack_segment)?;

        Ok(())
    }
}
