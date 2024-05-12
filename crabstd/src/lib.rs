#![no_std]
#![feature(
    const_mut_refs,
    const_trait_impl,
    const_intrinsic_copy,
    const_ptr_write
)]

extern crate alloc;

pub mod cursor;
pub mod fs;
pub mod mutex;
pub mod volatile;

pub mod syscall;
