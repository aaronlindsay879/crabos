use core::ptr;

/// Simple volatile type, where all reads and writes are guaranteed to not be optimised away by the compiler.
#[derive(Clone, Copy)]
pub struct Volatile<T: Copy>(T);

impl<T: Copy> Volatile<T> {
    /// Constructs a new `Volatile` type with the given value
    pub const fn new(value: T) -> Self {
        Volatile(value)
    }

    /// Reads the value of the volatile type
    pub fn read(&self) -> T {
        // SAFETY: we know self.0 is properly initialised and therefore will fulfill read_volatile requirements
        unsafe { ptr::read_volatile(&self.0) }
    }

    /// Writes a value to the volatile type
    pub fn write(&mut self, value: T) {
        // SAFETY: we know self.0 is properly initialised and therefore will fulfill write_volatile requirements
        unsafe { ptr::write_volatile(&mut self.0, value) }
    }
}
