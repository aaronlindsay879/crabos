use core::{arch::asm, marker::PhantomData};

use paste::paste;

pub trait PortRead {
    /// Reads a `Self` value from the given port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn read_from_port(port: u16) -> Self;
}

pub trait PortWrite {
    /// Writes a `Self` value to the given port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn write_to_port(port: u16, value: Self);
}

macro_rules! port_definition {
    ($reg:expr, $type:ty) => {
        paste! {
            impl crate::port::PortRead for $type {
                unsafe fn read_from_port(port: u16) -> $type {
                    let value: $type;
                    asm!(
                        concat!("in ", $reg, ", dx"),
                        out($reg) value,
                        in("dx") port,
                        options(nomem, nostack, preserves_flags)
                    );

                    value
                }
            }

            impl crate::port::PortWrite for $type {
                unsafe fn write_to_port(port: u16, value: $type) {
                    asm!(
                        concat!("out dx, ", $reg),
                        in("dx") port,
                        in($reg) value,
                        options(nomem, nostack, preserves_flags)
                    )
                }
            }
        }
    };
}

port_definition!("al", u8);
port_definition!("ax", u16);
port_definition!("eax", u32);

pub struct Port<T> {
    port: u16,
    _phantom: PhantomData<T>,
}

impl<T> Port<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }
}

impl<T: PortRead> Port<T> {
    /// Reads a `T` value from the given port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    pub unsafe fn read(&mut self) -> T {
        T::read_from_port(self.port)
    }
}

impl<T: PortWrite> Port<T> {
    /// Writes a `T` value to the given port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    pub unsafe fn write(&mut self, value: T) {
        T::write_to_port(self.port, value)
    }
}
