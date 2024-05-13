use core::{fmt, ops::Range, ptr};

use crabstd::volatile::Volatile;
use font_constants::{BACKUP_CHAR, CHAR_RASTER_HEIGHT, CHAR_RASTER_WIDTH};
use multiboot::FramebufferInfo;
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;

/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

const TAB_ALIGN: usize = 8 * CHAR_RASTER_WIDTH;

/// Constants for the usage of the [`noto_sans_mono_bitmap`] crate.
mod font_constants {
    use super::*;

    /// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
    /// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size20;

    /// The width of each single symbol of the mono space font.
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FONT_WEIGHT, CHAR_RASTER_HEIGHT);

    /// Backup character if a desired symbol is not available by the font.
    /// The '�' character requires the feature "unicode-specials".
    pub const BACKUP_CHAR: char = ' ';

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;

    pub const SHIFT_Y: usize = CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
}

/// Gets the raster of the given character
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }

    get(c)
        .or_else(|| get(BACKUP_CHAR))
        .expect("backup char failed")
}

/// Struct representing a pixel-based framebuffer
pub struct FrameBufferWriter {
    framebuffer: &'static mut [Volatile<u8>],
    info: FramebufferInfo,
    x_pos: usize,
    y_pos: usize,
    cursor_drawn: bool,
}

impl FrameBufferWriter {
    pub fn from_framebuffer(info: FramebufferInfo) -> Self {
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(
                info.buffer_addr as *mut _,
                (info.pitch * info.height) as usize,
            )
        };

        FrameBufferWriter::new(buffer, info)
    }

    pub fn new(framebuffer: &'static mut [Volatile<u8>], info: FramebufferInfo) -> Self {
        let mut buffer = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
            cursor_drawn: false,
        };
        buffer.clear();
        buffer
    }

    fn disable_cursor(&mut self) {
        for y_offset in 2..3 {
            for x in self.x_pos..self.x_pos + CHAR_RASTER_WIDTH {
                self.write_pixel(x, self.y_pos + CHAR_RASTER_HEIGHT.val() - y_offset, 0);
            }
        }

        self.cursor_drawn = false;
    }

    fn enable_cursor(&mut self) {
        for y_offset in 2..3 {
            for x in self.x_pos..self.x_pos + CHAR_RASTER_WIDTH {
                self.write_pixel(x, self.y_pos + CHAR_RASTER_HEIGHT.val() - y_offset, 127);
            }
        }

        self.cursor_drawn = true;
    }

    pub fn toggle_cursor(&mut self) {
        if self.cursor_drawn {
            self.disable_cursor()
        } else {
            self.enable_cursor()
        }
    }

    fn newline(&mut self) {
        const LINE_SHIFT: usize = CHAR_RASTER_HEIGHT.val() + LINE_SPACING;

        self.y_pos += LINE_SHIFT;
        if self.y_pos + LINE_SHIFT >= self.info.height as usize {
            self.scroll();
        }
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    fn tab(&mut self) {
        let target = ((self.x_pos + TAB_ALIGN - 1) / TAB_ALIGN) * TAB_ALIGN;

        while self.x_pos < target {
            self.write_char(' ');
        }
    }

    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;

        unsafe { ptr::write_bytes(self.framebuffer.as_mut_ptr(), 0, self.framebuffer.len()) }
    }

    pub fn clear_region(&mut self, range: Range<usize>) {
        unsafe {
            ptr::write_bytes(
                self.framebuffer.as_mut_ptr().add(range.start),
                0,
                range.end - range.start,
            )
        }
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            '\t' => self.tab(),
            const { 0x08 as char } => {
                if self.x_pos >= CHAR_RASTER_WIDTH + BORDER_PADDING {
                    self.x_pos -= CHAR_RASTER_WIDTH;
                    self.write_char(' ');
                    self.x_pos -= CHAR_RASTER_WIDTH;
                }
            }
            c => {
                let new_xpos = self.x_pos + CHAR_RASTER_WIDTH;
                if new_xpos >= self.info.width as usize {
                    self.newline();
                }
                let new_ypos = self.y_pos + CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.info.height as usize {
                    self.scroll();
                }
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    // move all text up 1 char
    fn scroll(&mut self) {
        let shift_bytes = font_constants::SHIFT_Y * self.info.pitch as usize;
        let end = (self.info.height * self.info.pitch) as usize;

        self.framebuffer.copy_within(shift_bytes..end, 0);
        self.clear_region((end - shift_bytes)..end);

        self.y_pos -= font_constants::SHIFT_Y;
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x_pos`.
    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * (self.info.width as usize) + x;

        let colour = [intensity / 2, intensity, intensity, 0];

        let bytes_per_pixel = self.info.bpp as usize / 8;
        let byte_offset = pixel_offset * bytes_per_pixel;

        for (&src, dst) in colour[..bytes_per_pixel]
            .iter()
            .zip(&mut self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)])
        {
            dst.write(src);
        }
    }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let cursor_drawn = self.cursor_drawn;

        if cursor_drawn {
            self.disable_cursor();
        }

        for c in s.chars() {
            self.write_char(c);
        }

        if cursor_drawn {
            self.enable_cursor();
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        let cursor_drawn = self.cursor_drawn;

        if cursor_drawn {
            self.disable_cursor();
        }

        self.write_char(c);

        if cursor_drawn {
            self.enable_cursor();
        }

        Ok(())
    }
}
