use core::arch::asm;

use paste::paste;

macro_rules! port_definition {
    ($char:ident => $reg:expr, $type:ty) => {
        paste! {
                pub unsafe fn [<in $char>](port: u16) -> $type {
                    let value: $type;
                    asm!(concat!("in ", $reg, ", dx"), out($reg) value, in("dx") port, options(nomem, nostack, preserves_flags));

                    value
                }

                pub unsafe fn [<out $char>](port: u16, value: $type) {
                    asm!(concat!("out dx, ", $reg), in("dx") port, in($reg) value, options(nomem, nostack, preserves_flags))
                }
            }
    };
}

port_definition!(b => "al", u8);
port_definition!(w => "ax", u16);
port_definition!(l => "eax", u32);
