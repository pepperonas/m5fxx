//! Display buffer and drawing primitives for the 240x135 IPS LCD display.

use crate::color::Color565;
use crate::font::get_glyph_5x7;

pub const DISPLAY_WIDTH: usize = 240;
pub const DISPLAY_HEIGHT: usize = 135;

pub struct DisplayBuffer {
    pub pixels: [Color565; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    dirty: bool,
}

impl Default for DisplayBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayBuffer {
    pub fn new() -> Self {
        Self {
            pixels: [Color565::BLACK; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            dirty: true,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    #[inline]
    pub fn clear(&mut self, color: Color565) {
        self.pixels.fill(color);
        self.dirty = true;
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color565) {
        if x >= 0 && (x as usize) < DISPLAY_WIDTH && y >= 0 && (y as usize) < DISPLAY_HEIGHT {
            let idx = (y as usize) * DISPLAY_WIDTH + (x as usize);
            self.pixels[idx] = color;
            self.dirty = true;
        }
    }

    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Color565 {
        if x >= 0 && (x as usize) < DISPLAY_WIDTH && y >= 0 && (y as usize) < DISPLAY_HEIGHT {
            let idx = (y as usize) * DISPLAY_WIDTH + (x as usize);
            self.pixels[idx]
        } else {
            Color565::BLACK
        }
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color565) {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_pixel(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color565) {
        if w <= 0 || h <= 0 {
            return;
        }
        self.draw_line(x, y, x + w - 1, y, color);
        self.draw_line(x, y + h - 1, x + w - 1, y + h - 1, color);
        self.draw_line(x, y, x, y + h - 1, color);
        self.draw_line(x + w - 1, y, x + w - 1, y + h - 1, color);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color565) {
        if w <= 0 || h <= 0 {
            return;
        }
        let x_start = x.max(0) as usize;
        let y_start = y.max(0) as usize;
        let x_end = ((x + w).min(DISPLAY_WIDTH as i32)).max(0) as usize;
        let y_end = ((y + h).min(DISPLAY_HEIGHT as i32)).max(0) as usize;

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        for cy in y_start..y_end {
            let row_start = cy * DISPLAY_WIDTH + x_start;
            let row_end = cy * DISPLAY_WIDTH + x_end;
            self.pixels[row_start..row_end].fill(color);
        }
        self.dirty = true;
    }

    /// Draws a character using the 5x7 font with pixel scaling.
    pub fn draw_char(
        &mut self,
        x: i32,
        y: i32,
        c: char,
        color: Color565,
        bg: Option<Color565>,
        scale: i32,
    ) -> i32 {
        let glyph = get_glyph_5x7(c);
        let s = scale.max(1);

        for (col, &col_byte) in glyph.iter().enumerate() {
            for row in 0..7 {
                let bit = (col_byte >> row) & 1;
                let px = x + (col as i32) * s;
                let py = y + row * s;
                if bit == 1 {
                    self.fill_rect(px, py, s, s, color);
                } else if let Some(bg_color) = bg {
                    self.fill_rect(px, py, s, s, bg_color);
                }
            }
        }
        // Spacing column (1 scaled pixel)
        if let Some(bg_color) = bg {
            self.fill_rect(x + 5 * s, y, s, 7 * s, bg_color);
        }

        6 * s
    }

    /// Draws a string of text.
    pub fn draw_string(
        &mut self,
        x: i32,
        y: i32,
        text: &str,
        color: Color565,
        bg: Option<Color565>,
        scale: i32,
    ) {
        let mut cur_x = x;
        let mut cur_y = y;
        let s = scale.max(1);
        let line_height = 8 * s;

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = x;
                cur_y += line_height;
                continue;
            }
            cur_x += self.draw_char(cur_x, cur_y, ch, color, bg, scale);
        }
    }

    /// Draws a 16-bit RGB565 bitmap image.
    pub fn draw_bitmap_rgb565(&mut self, x: i32, y: i32, w: usize, h: usize, data: &[Color565]) {
        for row in 0..h {
            let py = y + row as i32;
            if py < 0 || py >= DISPLAY_HEIGHT as i32 {
                continue;
            }
            for col in 0..w {
                let px = x + col as i32;
                if px < 0 || px >= DISPLAY_WIDTH as i32 {
                    continue;
                }
                let src_idx = row * w + col;
                if src_idx < data.len() {
                    let dst_idx = (py as usize) * DISPLAY_WIDTH + (px as usize);
                    self.pixels[dst_idx] = data[src_idx];
                }
            }
        }
        self.dirty = true;
    }

    /// Converts the current RGB565 frame to an RGBA8888 byte buffer for GPU/GUI texture upload.
    pub fn to_rgba8888(&self, out: &mut [u8]) {
        assert_eq!(out.len(), DISPLAY_WIDTH * DISPLAY_HEIGHT * 4);
        for (i, p) in self.pixels.iter().enumerate() {
            let [r, g, b, a] = p.to_rgba8888();
            let base = i * 4;
            out[base] = r;
            out[base + 1] = g;
            out[base + 2] = b;
            out[base + 3] = a;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_buffer_bounds() {
        let mut buf = DisplayBuffer::new();
        buf.set_pixel(-1, -1, Color565::WHITE);
        buf.set_pixel(DISPLAY_WIDTH as i32, DISPLAY_HEIGHT as i32, Color565::WHITE);
        assert_eq!(buf.get_pixel(0, 0), Color565::BLACK);

        buf.set_pixel(10, 20, Color565::RED);
        assert_eq!(buf.get_pixel(10, 20), Color565::RED);

        buf.fill_rect(50, 50, 10, 10, Color565::GREEN);
        assert_eq!(buf.get_pixel(55, 55), Color565::GREEN);
        assert_eq!(buf.get_pixel(49, 50), Color565::BLACK);
    }
}
