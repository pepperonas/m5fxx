//! ST7789 visible-panel model for the Cardputer ADV.
//! Native orientation is 135x240 in 240x320 controller RAM, offset (52,40).
use crate::{Color565, DisplayBuffer};

pub struct St7789 {
    ram: Vec<Color565>,
    command: u8,
    parameters: Vec<u8>,
    window: [u16; 4],
    cursor: (u16, u16),
    high_byte: Option<u8>,
    pub madctl: u8,
    pub colmod: u8,
    pub sleeping: bool,
    pub display_on: bool,
    pub inverted: bool,
    pub brightness: u8,
}
impl Default for St7789 {
    fn default() -> Self {
        Self {
            ram: vec![Color565::BLACK; 240 * 320],
            command: 0,
            parameters: Vec::new(),
            window: [0, 239, 0, 319],
            cursor: (0, 0),
            high_byte: None,
            madctl: 0,
            colmod: 0x55,
            sleeping: true,
            display_on: false,
            inverted: false,
            brightness: 255,
        }
    }
}
impl St7789 {
    pub fn command(&mut self, command: u8) {
        self.command = command;
        self.parameters.clear();
        self.high_byte = None;
        match command {
            0x01 => *self = Self::default(),
            0x10 => self.sleeping = true,
            0x11 => self.sleeping = false,
            0x20 => self.inverted = false,
            0x21 => self.inverted = true,
            0x28 => self.display_on = false,
            0x29 => self.display_on = true,
            0x2c => self.cursor = (self.window[0], self.window[2]),
            _ => {}
        }
    }
    pub fn data(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            if self.command == 0x2c || self.command == 0x3c {
                if self.colmod & 7 != 5 {
                    continue;
                }
                if let Some(high) = self.high_byte.take() {
                    let mut color = Color565(u16::from_be_bytes([high, byte]));
                    if self.madctl & 8 != 0 {
                        color = Color565(
                            (color.0 & 0x07e0) | ((color.0 & 31) << 11) | ((color.0 >> 11) & 31),
                        );
                    }
                    let (mut x, mut y) = (self.cursor.0 as usize, self.cursor.1 as usize);
                    if self.madctl & 0x20 != 0 {
                        std::mem::swap(&mut x, &mut y);
                    }
                    if self.madctl & 0x40 != 0 {
                        x = 239usize.saturating_sub(x);
                    }
                    if self.madctl & 0x80 != 0 {
                        y = 319usize.saturating_sub(y);
                    }
                    if x < 240 && y < 320 {
                        self.ram[y * 240 + x] = color;
                    }
                    if self.cursor.0 < self.window[1] {
                        self.cursor.0 += 1;
                    } else {
                        self.cursor.0 = self.window[0];
                        if self.cursor.1 < self.window[3] {
                            self.cursor.1 += 1;
                        } else {
                            self.cursor.1 = self.window[2];
                        }
                    }
                } else {
                    self.high_byte = Some(byte);
                }
            } else {
                self.parameters.push(byte);
                match (self.command, self.parameters.len()) {
                    (0x2a, 4) => {
                        self.window[0] =
                            u16::from_be_bytes([self.parameters[0], self.parameters[1]]);
                        self.window[1] =
                            u16::from_be_bytes([self.parameters[2], self.parameters[3]]);
                    }
                    (0x2b, 4) => {
                        self.window[2] =
                            u16::from_be_bytes([self.parameters[0], self.parameters[1]]);
                        self.window[3] =
                            u16::from_be_bytes([self.parameters[2], self.parameters[3]]);
                    }
                    (0x36, 1) => self.madctl = byte,
                    (0x3a, 1) => self.colmod = byte,
                    _ => {}
                }
            }
        }
    }
    /// Native desktop graphics writes the same logical landscape frame through RAMWR.
    pub fn write_landscape(&mut self, pixels: &[Color565]) {
        assert_eq!(pixels.len(), 240 * 135);
        self.command(0x36);
        self.data(&[0x60]);
        self.command(0x2a);
        self.data(&[0, 40, 1, 23]);
        self.command(0x2b);
        self.data(&[0, 53, 0, 187]);
        self.command(0x2c);
        for p in pixels {
            self.data(&p.0.to_be_bytes());
        }
    }
    pub fn present(&self, out: &mut DisplayBuffer) {
        for y in 0..135 {
            for x in 0..240 {
                // Landscape rotation 1: controller MV|MX.
                let mut color = self.ram[(40 + x) * 240 + (186 - y)];
                if !self.inverted {
                    color = Color565(!color.0);
                }
                if self.sleeping || !self.display_on {
                    color = Color565::BLACK;
                }
                out.pixels[y * 240 + x] = color;
            }
        }
        out.mark_dirty();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn landscape_roundtrip_and_power() {
        let mut panel = St7789::default();
        panel.command(0x11);
        panel.command(0x29);
        panel.command(0x21);
        let frame: Vec<_> = (0..240 * 135).map(|i| Color565(i as u16)).collect();
        panel.write_landscape(&frame);
        let mut out = DisplayBuffer::new();
        panel.present(&mut out);
        assert_eq!(out.pixels.as_slice(), frame);
        panel.command(0x20);
        panel.present(&mut out);
        assert_eq!(out.pixels[123], Color565(!frame[123].0));
        panel.command(0x10);
        panel.present(&mut out);
        assert!(out.pixels.iter().all(|p| *p == Color565::BLACK));
        panel.command(0x01);
        assert!(panel.sleeping);
        assert!(!panel.display_on);
    }
    #[test]
    fn split_rgb565_and_bgr() {
        let mut panel = St7789::default();
        panel.command(0x36);
        panel.data(&[8]);
        panel.command(0x2c);
        panel.data(&[0xf8]);
        panel.data(&[0x00]);
        assert_eq!(panel.ram[0], Color565::BLUE);
    }
}
