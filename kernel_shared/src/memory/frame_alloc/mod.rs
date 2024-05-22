use x86_64::structures::Frame;

pub mod bitmap;
pub mod bump;
pub mod tiny;

pub trait FrameAllocator {
    /// Finds a free frame to allocate and return
    fn allocate_frame(&mut self) -> Option<Frame>;

    /// Frees a given frame
    fn deallocate_frame(&mut self, frame: Frame);
}
