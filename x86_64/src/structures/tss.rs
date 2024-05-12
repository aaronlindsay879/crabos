use core::fmt::Display;

use crate::VirtualAddress;

#[derive(Debug)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    _reserved1: u32,
    pub privilege_stack_table: [VirtualAddress; 3],
    _reserved2: u64,
    pub interrupt_stack_table: [VirtualAddress; 7],
    _reserved3: u64,
    _reserved4: u16,
    pub base_addr: u16,
}

impl TaskStateSegment {
    pub fn new() -> Self {
        Self {
            privilege_stack_table: [0; 3],
            interrupt_stack_table: [0; 7],
            base_addr: core::mem::size_of::<Self>() as u16,
            _reserved1: 0,
            _reserved2: 0,
            _reserved3: 0,
            _reserved4: 0,
        }
    }
}

impl Display for TaskStateSegment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let privilege_table = self.privilege_stack_table;
        let interrupt_table = self.interrupt_stack_table;

        writeln!(f, "Task state segment:")?;
        writeln!(f, "\tPrivilege stack table: {:?}", privilege_table)?;
        writeln!(f, "\tInterrupt stack table: {:?}", interrupt_table)?;
        writeln!(f, "\tIomap base addr: {:#X}", self.base_addr)?;

        Ok(())
    }
}
