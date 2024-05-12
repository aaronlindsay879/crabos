#![allow(unused)]

use core::{
    cell::Cell,
    fmt::{self, Write},
};

use crabstd::mutex::Mutex;
use x86_64::interrupts;

use self::{framebuffer::FrameBufferWriter, textbuffer::TextBufferWriter};
use crate::BootInfo;

pub mod framebuffer;
pub mod serial;
pub mod textbuffer;

/// Stores some form of text writer
pub enum Writer {
    Framebuffer(FrameBufferWriter),
    Textbuffer(TextBufferWriter),
}

impl Writer {
    /// Selects the correct writer type based in information returned by multiboot
    pub fn from_bootinfo(bootinfo: &BootInfo) -> Option<Self> {
        let framebuffer_info = bootinfo.framebuffer_info.unwrap();

        match framebuffer_info.buffer_type {
            0 | 1 => Some(Writer::Framebuffer(FrameBufferWriter::from_framebuffer(
                framebuffer_info,
            ))),
            2 => Some(Writer::Textbuffer(TextBufferWriter::new())),
            _ => None,
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self {
            Writer::Framebuffer(framebuffer) => framebuffer.write_str(s),
            Writer::Textbuffer(textbuffer) => textbuffer.write_str(s),
        }
    }
}

pub static WRITER: Mutex<Cell<Option<Writer>>> = Mutex::new(Cell::new(None));

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        // safe because writer _must_ be initialised as part of booting process
        // if is actually None by the time println is being used, we have bigger problems
        unsafe {
            writer
                .get_mut()
                .as_mut()
                .unwrap_unchecked()
                .write_fmt(args)
                .unwrap();
        }
    })
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
