use x86_64::structures::Frame;

use super::FrameAllocator;

/// Minimal frame allocator that just hands out one of a few saved frames
pub struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
    /// Construct a new tiny allocator, allocating three frames in the process
    pub fn new<A: FrameAllocator>(allocator: &mut A) -> Self {
        let mut f = || allocator.allocate_frame();
        let frames = [f(), f(), f()];

        Self(frames)
    }
}

impl FrameAllocator for TinyAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        // find and return one of the saved frames
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        // find a frame that's been handed out and save the unused frame there
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("Tiny allocator can hold only 3 frames.");
    }
}
