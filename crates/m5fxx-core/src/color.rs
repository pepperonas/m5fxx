//! RGB565 color representation matching ST7789 / M5GFX hardware formats.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color565(pub u16);

impl Color565 {
    pub const BLACK: Self = Self(0x0000);
    pub const NAVY: Self = Self(0x000F);
    pub const DARKGREEN: Self = Self(0x03E0);
    pub const DARKCYAN: Self = Self(0x03EF);
    pub const MAROON: Self = Self(0x7800);
    pub const PURPLE: Self = Self(0x780F);
    pub const OLIVE: Self = Self(0x7BE0);
    pub const LIGHTGREY: Self = Self(0xD69A);
    pub const DARKGREY: Self = Self(0x7BEF);
    pub const BLUE: Self = Self(0x001F);
    pub const GREEN: Self = Self(0x07E0);
    pub const CYAN: Self = Self(0x07FF);
    pub const RED: Self = Self(0xF800);
    pub const MAGENTA: Self = Self(0xF81F);
    pub const YELLOW: Self = Self(0xFFE0);
    pub const WHITE: Self = Self(0xFFFF);
    pub const ORANGE: Self = Self(0xFD20);
    pub const GREENYELLOW: Self = Self(0xAFE5);
    pub const PINK: Self = Self(0xFC18);

    #[inline]
    pub const fn from_rgb888(r: u8, g: u8, b: u8) -> Self {
        let r5 = (r as u16 >> 3) & 0x1F;
        let g6 = (g as u16 >> 2) & 0x3F;
        let b5 = (b as u16 >> 3) & 0x1F;
        Self((r5 << 11) | (g6 << 5) | b5)
    }

    #[inline]
    pub const fn to_rgb888(self) -> (u8, u8, u8) {
        let r5 = (self.0 >> 11) & 0x1F;
        let g6 = (self.0 >> 5) & 0x3F;
        let b5 = self.0 & 0x1F;

        // Scale 5-bit to 8-bit (x * 255 / 31) -> (x << 3) | (x >> 2)
        let r = ((r5 << 3) | (r5 >> 2)) as u8;
        // Scale 6-bit to 8-bit (x * 255 / 63) -> (x << 2) | (x >> 4)
        let g = ((g6 << 2) | (g6 >> 4)) as u8;
        // Scale 5-bit to 8-bit
        let b = ((b5 << 3) | (b5 >> 2)) as u8;

        (r, g, b)
    }

    #[inline]
    pub const fn to_rgba8888(self) -> [u8; 4] {
        let (r, g, b) = self.to_rgb888();
        [r, g, b, 255]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb565_conversions() {
        assert_eq!(Color565::BLACK.to_rgb888(), (0, 0, 0));
        assert_eq!(Color565::WHITE.to_rgb888(), (255, 255, 255));

        let red = Color565::from_rgb888(255, 0, 0);
        assert_eq!(red, Color565::RED);
        let (r, g, b) = red.to_rgb888();
        assert_eq!((r, g, b), (255, 0, 0));

        let green = Color565::from_rgb888(0, 255, 0);
        assert_eq!(green, Color565::GREEN);
        let (r, g, b) = green.to_rgb888();
        assert_eq!((r, g, b), (0, 255, 0));

        let blue = Color565::from_rgb888(0, 0, 255);
        assert_eq!(blue, Color565::BLUE);
        let (r, g, b) = blue.to_rgb888();
        assert_eq!((r, g, b), (0, 0, 255));
    }
}
