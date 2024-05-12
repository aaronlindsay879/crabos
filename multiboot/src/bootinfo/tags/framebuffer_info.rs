use super::{ParseTag, Tag};

/// Stores information about the framebuffer
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Framebuffer-info
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FramebufferInfo {
    pub buffer_addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub buffer_type: u8,
    _reserved: u8,
    pub colour: FramebufferColour,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum FramebufferColour {
    None,
    Indexed(&'static [[u8; 3]]),
    Direct {
        red_field_position: u8,
        red_mask_size: u8,

        green_field_position: u8,
        green_mask_size: u8,

        blue_field_position: u8,
        blue_mask_size: u8,
    },
}

impl ParseTag for FramebufferInfo {
    unsafe fn parse(addr: *const u32, _size: usize) -> Option<Tag> {
        /*      +--------------------+
        u32     | type = 8           |
        u32     | size               |
        u64     | framebuffer_addr   |
        u32     | framebuffer_pitch  |
        u32     | framebuffer_width  |
        u32     | framebuffer_height |
        u8      | framebuffer_bpp    |
        u8      | framebuffer_type   |
        u8      | reserved           |
        varies  | color_info         |
                +--------------------+ */

        let buffer_addr = *(addr as *const u64);
        let pitch = *addr.add(2);
        let width = *addr.add(3);
        let height = *addr.add(4);

        let addr = addr.add(5) as *const u8;
        let bpp = *addr;
        let buffer_type = *addr.add(1);
        let reserved = *addr.add(2);

        let colour = match buffer_type {
            0 => {
                /*       +----------------------------------+
                u32     | framebuffer_palette_num_colors   |
                varies  | framebuffer_palette              |
                        +----------------------------------+  */

                /*       +-------------+
                u8      | red_value   |
                u8      | green_value |
                u8      | blue_value  |
                        +-------------+ */

                let num = *(addr.add(3) as *const u32) as usize;
                FramebufferColour::Indexed(core::slice::from_raw_parts(
                    addr.add(7) as *const [u8; 3],
                    num,
                ))
            }
            1 => {
                /*     +----------------------------------+
                u8     | framebuffer_red_field_position   |
                u8     | framebuffer_red_mask_size        |
                u8     | framebuffer_green_field_position |
                u8     | framebuffer_green_mask_size      |
                u8     | framebuffer_blue_field_position  |
                u8     | framebuffer_blue_mask_size       |
                      +----------------------------------+ */

                let addr = addr.add(3);

                FramebufferColour::Direct {
                    red_field_position: *addr,
                    red_mask_size: *addr.add(1),
                    green_field_position: *addr.add(2),
                    green_mask_size: *addr.add(3),
                    blue_field_position: *addr.add(4),
                    blue_mask_size: *addr.add(5),
                }
            }
            _ => FramebufferColour::None,
        };

        Some(Tag::FramebufferInfo(FramebufferInfo {
            buffer_addr,
            pitch,
            width,
            height,
            bpp,
            buffer_type,
            _reserved: reserved,
            colour,
        }))
    }
}
