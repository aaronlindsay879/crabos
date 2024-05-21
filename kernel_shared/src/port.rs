use core::fmt::Write;

use bitflags::bitflags;
use x86_64::io::{inb, outb};

pub mod ports {
    pub const COM1: u16 = 0x3F8;
    pub const COM2: u16 = 0x2F8;
    pub const COM3: u16 = 0x3E8;
    pub const COM4: u16 = 0x2E8;
    pub const COM5: u16 = 0x5F8;
    pub const COM6: u16 = 0x4F8;
    pub const COM7: u16 = 0x5E8;
    pub const COM8: u16 = 0x4E8;
}

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop()
        }
    };
    ($self:expr => LINE_STATUS_EMPTY) => {
        while !$self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY) {
            core::hint::spin_loop()
        }
    };
}

pub struct SerialPort(u16);

impl SerialPort {
    /// Creates a new serial port at the given port
    ///
    /// # Safety
    /// `port` must be a valid port to read and/or write from
    pub const unsafe fn new(port: u16) -> Self {
        Self(port)
    }

    pub fn init(&mut self) {
        unsafe {
            // disable interrupts
            outb(self.port_int_en(), 0x00);

            // enable DLAB
            outb(self.port_line_ctrl(), 0x80);

            // Set divisor to 3 (38400 baud)
            outb(self.port_data(), 0x03);
            outb(self.port_int_en(), 0x00);

            // disable DLAB and set data word length to 8 bits
            outb(self.port_line_ctrl(), 0x03);

            // enable FIFO, clear queues, and set interrupt watermark at 14 bytes
            outb(self.port_fifo_control(), 0xC7);

            // mark data terminal ready, signal request to send
            // and enabkle output #2 (interrupt line)
            outb(self.port_modem_ctrl(), 0x0B);

            // enable interrupts
            outb(self.port_int_en(), 0x01);
        }
    }

    /// Sends a byte
    pub fn send(&mut self, data: u8) {
        unsafe {
            match data {
                8 | 0x7F => {
                    wait_for!(self => LINE_STATUS_EMPTY);
                    outb(self.port_data(), 8);
                    wait_for!(self => LINE_STATUS_EMPTY);
                    outb(self.port_data(), b' ');
                    wait_for!(self => LINE_STATUS_EMPTY);
                    outb(self.port_data(), 8);
                }
                _ => {
                    wait_for!(self => LINE_STATUS_EMPTY);
                    outb(self.port_data(), data);
                }
            }
        }
    }

    /// Data port, R+W
    const fn port_data(&self) -> u16 {
        self.0
    }

    /// Interrupt enable port, W
    const fn port_int_en(&self) -> u16 {
        self.0 + 1
    }

    /// Fifo control port, W
    const fn port_fifo_control(&self) -> u16 {
        self.0 + 2
    }

    /// Line control port, W
    const fn port_line_ctrl(&self) -> u16 {
        self.0 + 3
    }

    /// Modem control port, W
    const fn port_modem_ctrl(&self) -> u16 {
        self.0 + 4
    }

    /// Line status port, R
    const fn port_line_status(&self) -> u16 {
        self.0 + 5
    }

    fn line_status(&self) -> LineStatusFlags {
        unsafe { LineStatusFlags::from_bits_truncate(inb(self.port_line_status())) }
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }

        Ok(())
    }
}

bitflags! {
    /// Line status flags
    struct LineStatusFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}
