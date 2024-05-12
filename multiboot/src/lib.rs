#![no_std]
#![feature(
    const_mut_refs,
    const_trait_impl,
    const_intrinsic_copy,
    effects,
    iter_intersperse
)]

pub mod bootinfo;
pub mod header;

pub mod prelude {
    pub use crate::{
        bootinfo::*,
        header::{HeaderTagValue::*, *},
        multiboot_header,
    };
}

pub use prelude::*;
