pub mod active_table;
pub mod entry;
pub mod table;

/// Number of entries per page (4KiB / 8 bytes)
const ENTRY_COUNT: usize = 512;

/// Offset for physical memory mapping
const PHYS_MEM_OFFSET: usize = 0xFFFF800000000000;
