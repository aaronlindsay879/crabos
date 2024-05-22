#![no_std]
#![feature(
    const_mut_refs,
    const_trait_impl,
    const_intrinsic_copy,
    const_ptr_write
)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod fs;
#[cfg(feature = "alloc")]
pub mod syscall;

pub mod cursor;
pub mod mutex;
pub mod volatile;
