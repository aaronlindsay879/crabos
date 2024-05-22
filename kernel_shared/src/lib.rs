#![no_std]
#![feature(iter_intersperse)]

pub mod logger;
pub mod memory;
pub mod port;
pub mod serial;

pub const HEAP_SIZE: usize = 128 * 1024; // 128 KiB
