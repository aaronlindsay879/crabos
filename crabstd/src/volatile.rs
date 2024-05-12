use core::ptr;

#[derive(Clone, Copy)]
pub struct Volatile<T: Copy>(T);

impl<T: Copy> Volatile<T> {
    pub const fn new(value: T) -> Self {
        Volatile(value)
    }

    pub fn read(&self) -> T {
        // SAFETY: we know self.0 is properly initialised and therefore will fulfill read_volatile requirements
        unsafe { ptr::read_volatile(&self.0) }
    }

    pub fn write(&mut self, value: T) {
        // SAFETY: we know self.0 is properly initialised and therefore will fulfill write_volatile requirements
        unsafe { ptr::write_volatile(&mut self.0, value) }
    }
}
