use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use super::{
    entry::{Entry, EntryFlags},
    ENTRY_COUNT,
};
use crate::memory::FrameAllocator;

/// Pointer to active level 4 table
pub const P4: *mut Table<Level4> = 0xFFFFFFFF_FFFFF000 as *mut _;

/// Marker trait for recursive table levels
pub trait TableLevel {}

/// Level 4 page table
pub enum Level4 {}

/// Level 3 page table
pub enum Level3 {}

/// Level 2 page table
pub enum Level2 {}

/// Level 1 page table
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

/// Marker trait to indicate which table levels have sub-levels
pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

/// Stores a page table of a specific level
pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L: TableLevel> Table<L> {
    /// Sets all entries in table to unused
    pub fn zero(&mut self) {
        for entry in &mut self.entries {
            entry.set_unused();
        }
    }

    /// Checks if the page table is empty by iterating over each entry
    pub fn is_empty(&self) -> bool {
        self.entries.iter().all(Entry::is_unused)
    }
}

impl<L: HierarchicalLevel> Table<L> {
    /// Finds address of the next level table at the specified index
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();

        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE)
        {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    /// Finds the next level table with the specified index
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    /// Finds the next level table with the specified index
    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    /// Finds the next level table with the specified index, creating a blank table if it doesnt exist
    pub fn next_table_create<A: FrameAllocator>(
        &mut self,
        index: usize,
        allocator: &mut A,
    ) -> &mut Table<L::NextLevel> {
        if self.next_table(index).is_none() {
            assert!(
                !self.entries[index].flags().contains(EntryFlags::HUGE_PAGE),
                "mapping code does not support huge pages"
            );

            // create, set, and zero a new frame
            let frame = allocator.allocate_frame().expect("no available frames");

            log::trace!(
                "allocating new {} table at frame {frame:?}",
                core::any::type_name::<L::NextLevel>()
            );
            self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }

        self.next_table_mut(index).unwrap()
    }
}

impl<L: TableLevel> Index<usize> for Table<L> {
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L: TableLevel> IndexMut<usize> for Table<L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}
