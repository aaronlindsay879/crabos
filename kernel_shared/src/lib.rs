#![no_std]
#![feature(iter_intersperse)]

pub mod logger;
pub mod memory;
pub mod serial;
pub mod serial_port;

pub const HEAP_SIZE: usize = 128 * 1024; // 128 KiB
pub const STACK_SIZE: usize = 128 * 1024; // 128 KiB
