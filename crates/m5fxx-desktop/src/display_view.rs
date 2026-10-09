//! Integer scaling in physical monitor pixels, independent of egui point density.
use egui::{pos2, vec2, Rect};

pub fn pixel_rect(available: Rect, pixels_per_point: f32) -> (Rect, u32) {
    let ppp = pixels_per_point.max(0.1);
    let scale = ((available.width() * ppp / 240.0)
        .min(available.height() * ppp / 135.0)
        .floor() as u32)
        .max(1);
    let size = vec2(240.0, 135.0) * scale as f32 / ppp;
    let min = available.center() - size / 2.0;
    let min = pos2((min.x * ppp).round() / ppp, (min.y * ppp).round() / ppp);
    (Rect::from_min_size(min, size), scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_monitor_pixels_at_all_densities() {
        for ppp in [1.0, 1.25, 1.5, 2.0, 3.0] {
            let available = Rect::from_min_size(pos2(10.3, 20.7), vec2(900.0, 600.0));
            let (rect, scale) = pixel_rect(available, ppp);
            assert!((rect.width() * ppp - 240.0 * scale as f32).abs() < 0.001);
            assert!((rect.height() * ppp - 135.0 * scale as f32).abs() < 0.001);
            assert!((rect.min.x * ppp - (rect.min.x * ppp).round()).abs() < 0.001);
            assert!(available.expand(1.0).contains_rect(rect));
        }
    }
}
