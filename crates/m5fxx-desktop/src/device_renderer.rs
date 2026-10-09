//! ADV-inspired vector rendering and layout metrics for the M5Stack Cardputer.
//!
//! Physical Cardputer Dimensions:
//! - 84.0 mm x 54.0 mm x 19.7 mm
//! - Display: 1.14" IPS LCD (240x135 pixels, active area ~24.9mm x 14.9mm)
//! - Keyboard: 56 keys arranged in 4 rows x 14 columns
//!
//! Decorative module details do not represent emulated GPIO functionality.
//! Both hardware models share this shell; the badge follows the selected model.
//!
//! In virtual normalized design space:
//! Device width: 840.0 pt
//! Device height: 540.0 pt

use egui::{
    pos2, vec2, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Stroke, StrokeKind, Ui,
};
use m5fxx_core::input::{get_key_labels, KeyCoord, COLS, ROWS};
use m5fxx_core::{CardputerHal, CardputerModel};

pub const DESIGN_WIDTH: f32 = 840.0;
pub const DESIGN_HEIGHT: f32 = 540.0;

// Device dimensions in design space
pub const CASE_CORNER_RADIUS: u8 = 28;

// Display bezel in design space
pub const DISPLAY_BEZEL_X: f32 = 166.0;
pub const DISPLAY_BEZEL_Y: f32 = 32.0;
pub const DISPLAY_BEZEL_W: f32 = 300.0;
pub const DISPLAY_BEZEL_H: f32 = 192.0;

pub const SCREEN_INNER_X: f32 = 185.0;
pub const SCREEN_INNER_Y: f32 = 49.0;
pub const SCREEN_INNER_W: f32 = 264.0;
pub const SCREEN_INNER_H: f32 = 148.5; // Exactly 240:135 aspect ratio (16:9)

// Keyboard dimensions in design space
pub const KB_START_X: f32 = 38.0;
pub const KB_START_Y: f32 = 262.0;
pub const KB_TOTAL_W: f32 = 764.0;
pub const KB_TOTAL_H: f32 = 238.0;

pub const KEY_GAP_X: f32 = 10.0;
pub const KEY_GAP_Y: f32 = 13.0;
pub const KEY_W: f32 = (KB_TOTAL_W - (COLS as f32 - 1.0) * KEY_GAP_X) / COLS as f32; // Key width in design space
pub const KEY_H: f32 = (KB_TOTAL_H - (ROWS as f32 - 1.0) * KEY_GAP_Y) / ROWS as f32; // Key height in design space

/// Calculates the bounding box in design space for a key at (row, col)
pub fn get_key_rect_design(row: usize, col: usize) -> Rect {
    let x = KB_START_X + (col as f32) * (KEY_W + KEY_GAP_X);
    let y = KB_START_Y + (row as f32) * (KEY_H + KEY_GAP_Y);
    Rect::from_min_size(pos2(x, y), vec2(KEY_W, KEY_H))
}

pub struct DeviceViewTransform {
    pub offset: Pos2,
    pub scale: f32,
}

impl DeviceViewTransform {
    pub fn new(available_rect: Rect) -> Self {
        let scale_x = available_rect.width() / DESIGN_WIDTH;
        let scale_y = available_rect.height() / DESIGN_HEIGHT;
        let scale = scale_x.min(scale_y).max(0.2); // Keep aspect ratio

        let scaled_w = DESIGN_WIDTH * scale;
        let scaled_h = DESIGN_HEIGHT * scale;

        let offset_x = available_rect.min.x + (available_rect.width() - scaled_w) * 0.5;
        let offset_y = available_rect.min.y + (available_rect.height() - scaled_h) * 0.5;

        Self {
            offset: pos2(offset_x, offset_y),
            scale,
        }
    }

    #[inline]
    pub fn to_screen_pos(&self, p: Pos2) -> Pos2 {
        pos2(
            self.offset.x + p.x * self.scale,
            self.offset.y + p.y * self.scale,
        )
    }

    #[inline]
    pub fn to_screen_rect(&self, r: Rect) -> Rect {
        Rect::from_min_max(self.to_screen_pos(r.min), self.to_screen_pos(r.max))
    }

    #[inline]
    pub fn to_design_pos(&self, p: Pos2) -> Pos2 {
        pos2(
            (p.x - self.offset.x) / self.scale,
            (p.y - self.offset.y) / self.scale,
        )
    }
}

/// Renders the complete vector body of the M5Stack Cardputer
pub fn render_cardputer_device(
    ui: &mut Ui,
    tf: &DeviceViewTransform,
    hal: &mut CardputerHal,
    display_texture_id: egui::TextureId,
    clicked_key: &mut Option<KeyCoord>,
) {
    let painter = ui.painter();

    // Layered moulded shell: dark base, silver edge and warm white faceplate.
    let radius = |r: f32| CornerRadius::same((r * tf.scale).max(1.0) as u8);
    let body = Rect::from_min_size(Pos2::ZERO, vec2(DESIGN_WIDTH, DESIGN_HEIGHT));
    for (spread, alpha) in [(16.0, 10), (10.0, 18), (4.0, 30)] {
        painter.rect_filled(
            tf.to_screen_rect(body.translate(vec2(0.0, 16.0)).expand(spread)),
            radius(28.0),
            Color32::from_black_alpha(alpha),
        );
    }
    painter.rect_filled(
        tf.to_screen_rect(body),
        radius(24.0),
        Color32::from_rgb(45, 48, 49),
    );
    let edge = Rect::from_min_size(pos2(0.0, 0.0), vec2(840.0, 524.0));
    painter.rect_filled(
        tf.to_screen_rect(edge),
        radius(24.0),
        Color32::from_rgb(167, 173, 171),
    );
    let face = Rect::from_min_size(pos2(3.0, 3.0), vec2(834.0, 505.0));
    painter.rect_filled(
        tf.to_screen_rect(face),
        radius(22.0),
        Color32::from_rgb(230, 233, 227),
    );
    painter.rect_stroke(
        tf.to_screen_rect(face.shrink(3.0)),
        radius(20.0),
        Stroke::new(tf.scale, Color32::from_rgb(252, 253, 248)),
        StrokeKind::Inside,
    );

    for p in [
        pos2(20.0, 20.0),
        pos2(820.0, 20.0),
        pos2(20.0, 493.0),
        pos2(820.0, 493.0),
    ] {
        let center = tf.to_screen_pos(p);
        painter.circle_filled(center, 6.0 * tf.scale, Color32::from_rgb(128, 134, 130));
        painter.circle_filled(center, 4.0 * tf.scale, Color32::from_rgb(197, 202, 195));
        painter.line_segment(
            [
                center - vec2(2.5, 2.5) * tf.scale,
                center + vec2(2.5, 2.5) * tf.scale,
            ],
            Stroke::new(tf.scale, Color32::from_rgb(83, 89, 86)),
        );
    }
    painter.text(
        tf.to_screen_pos(pos2(32.0, 38.0)),
        egui::Align2::LEFT_TOP,
        "CARDPUTER",
        FontId::monospace(15.0 * tf.scale),
        Color32::from_rgb(53, 61, 59),
    );
    let badge = match hal.status.model {
        CardputerModel::CardputerOriginal => "M5",
        CardputerModel::CardputerAdv => "ADV",
    };
    painter.text(
        tf.to_screen_pos(pos2(32.0, 63.0)),
        egui::Align2::LEFT_TOP,
        badge,
        FontId::proportional(35.0 * tf.scale),
        Color32::from_rgb(24, 33, 32),
    );
    painter.text(
        tf.to_screen_pos(pos2(32.0, 111.0)),
        egui::Align2::LEFT_TOP,
        "M5STACK",
        FontId::monospace(13.0 * tf.scale),
        Color32::from_rgb(69, 78, 73),
    );

    // Exposed expansion-module label, as on the ADV reference.
    let module = Rect::from_min_size(pos2(510.0, 30.0), vec2(286.0, 160.0));
    painter.rect_filled(
        tf.to_screen_rect(module),
        radius(10.0),
        Color32::from_rgb(187, 195, 189),
    );
    painter.rect_filled(
        tf.to_screen_rect(module.shrink(5.0)),
        radius(7.0),
        Color32::from_rgb(247, 248, 237),
    );
    painter.text(
        tf.to_screen_pos(pos2(528.0, 42.0)),
        egui::Align2::LEFT_TOP,
        "M5Stack / ESP32-S3",
        FontId::monospace(13.0 * tf.scale),
        Color32::from_rgb(54, 65, 59),
    );
    for col in 0..12 {
        for row in 0..2 {
            let x = 530.0 + col as f32 * 21.5;
            let y = 72.0 + row as f32 * 60.0;
            let colors = [
                Color32::from_rgb(231, 151, 76),
                Color32::from_rgb(104, 186, 183),
                Color32::from_rgb(221, 178, 189),
            ];
            painter.rect_filled(
                tf.to_screen_rect(Rect::from_min_size(pos2(x, y), vec2(15.0, 44.0))),
                radius(2.0),
                colors[col % 3],
            );
            for hole in 0..3 {
                painter.circle_filled(
                    tf.to_screen_pos(pos2(x + 7.5, y + 8.0 + hole as f32 * 13.0)),
                    3.0 * tf.scale,
                    Color32::from_rgb(61, 72, 67),
                );
            }
        }
    }
    // Small physical G0 button below the branding.
    let g0 = tf.to_screen_rect(Rect::from_min_size(pos2(78.0, 156.0), vec2(42.0, 33.0)));
    let hover = ui.rect_contains_pointer(g0);
    if hover && ui.input(|i| i.pointer.primary_down()) {
        hal.input.press_btn_g0();
    } else {
        hal.input.release_btn_g0();
    }
    painter.rect_filled(g0, radius(9.0), Color32::from_rgb(64, 72, 67));
    painter.rect_filled(
        g0.shrink(4.0 * tf.scale),
        radius(6.0),
        if hal.input.btn_g0_pressed {
            Color32::from_rgb(235, 145, 62)
        } else {
            Color32::from_rgb(193, 201, 189)
        },
    );
    painter.text(
        g0.center(),
        egui::Align2::CENTER_CENTER,
        "G0",
        FontId::monospace(12.0 * tf.scale),
        Color32::from_rgb(33, 40, 36),
    );
    for col in 0..5 {
        painter.circle_filled(
            tf.to_screen_pos(pos2(37.0 + col as f32 * 8.0, 207.0)),
            2.0 * tf.scale,
            Color32::from_rgb(77, 85, 80),
        );
    }

    // 7. Display Bezel (Glossy black acrylic border around LCD)
    let bezel_rect_design = Rect::from_min_size(
        pos2(DISPLAY_BEZEL_X, DISPLAY_BEZEL_Y),
        vec2(DISPLAY_BEZEL_W, DISPLAY_BEZEL_H),
    );
    let bezel_rect_screen = tf.to_screen_rect(bezel_rect_design);
    let bezel_radius = ((12.0 * tf.scale) as u8).max(1);
    painter.rect_filled(
        bezel_rect_screen,
        CornerRadius::same(bezel_radius),
        Color32::from_rgb(0x12, 0x13, 0x15),
    );
    painter.rect_stroke(
        bezel_rect_screen,
        CornerRadius::same(bezel_radius),
        Stroke::new(2.0 * tf.scale, Color32::from_rgb(0x22, 0x24, 0x28)),
        StrokeKind::Inside,
    );

    // Display Active Area & Texture
    let screen_rect_design = Rect::from_min_size(
        pos2(SCREEN_INNER_X, SCREEN_INNER_Y),
        vec2(SCREEN_INNER_W, SCREEN_INNER_H),
    );
    let screen_rect_screen = tf.to_screen_rect(screen_rect_design);

    // Draw the emulated ST7789 Framebuffer Texture
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    painter.image(display_texture_id, screen_rect_screen, uv, Color32::WHITE);

    // Subtle LCD inner rim shadow
    painter.rect_stroke(
        screen_rect_screen,
        CornerRadius::ZERO,
        Stroke::new(1.5 * tf.scale, Color32::from_rgb(0x05, 0x05, 0x08)),
        StrokeKind::Inside,
    );

    // 8. Keyboard Plate (Dark recessed tray)
    let kb_tray_design = Rect::from_min_size(
        pos2(KB_START_X - 6.0, KB_START_Y - 6.0),
        vec2(KB_TOTAL_W + 12.0, KB_TOTAL_H + 12.0),
    );
    let kb_tray_screen = tf.to_screen_rect(kb_tray_design);
    let tray_radius = ((8.0 * tf.scale) as u8).max(1);
    painter.rect_filled(
        kb_tray_screen,
        CornerRadius::same(tray_radius),
        Color32::from_rgb(215, 220, 212),
    );
    painter.rect_stroke(
        kb_tray_screen,
        CornerRadius::same(tray_radius),
        Stroke::new(1.5 * tf.scale, Color32::from_rgb(198, 205, 196)),
        StrokeKind::Inside,
    );

    // 9. 56 Key Caps Rendering and Click Handling
    let pointer_down = ui.input(|i| i.pointer.primary_down());
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

    for row in 0..ROWS {
        for col in 0..COLS {
            let key_rect_design = get_key_rect_design(row, col);
            let key_rect_screen = tf.to_screen_rect(key_rect_design);

            let is_hovered = pointer_pos.is_some_and(|pos| key_rect_screen.contains(pos));
            let is_pressed_virtual = hal.input.is_key_pressed(row as u8, col as u8);

            // Click interaction
            if is_hovered && pointer_down {
                *clicked_key = Some(KeyCoord::new(row as u8, col as u8));
                hal.input.press_key(row as u8, col as u8);
            }

            // Tiny raised black keycaps with orange and green modifier accents.
            let accent = if row == 2 && (col == 0 || col == 13) {
                Some(Color32::from_rgb(239, 126, 47))
            } else if row == 3 && col == 0 {
                Some(Color32::from_rgb(74, 175, 125))
            } else {
                None
            };
            let key_bg = if is_pressed_virtual {
                Color32::from_rgb(55, 146, 162)
            } else if is_hovered {
                Color32::from_rgb(80, 87, 81)
            } else {
                accent.unwrap_or(Color32::from_rgb(37, 43, 39))
            };
            let text_color = if accent.is_some() && !is_pressed_virtual {
                Color32::from_rgb(28, 35, 29)
            } else {
                Color32::from_rgb(243, 245, 234)
            };
            let key_radius = radius(10.0);
            painter.rect_filled(
                key_rect_screen.translate(vec2(0.0, 3.0 * tf.scale)),
                key_radius,
                Color32::from_rgb(131, 140, 130),
            );
            painter.rect_filled(key_rect_screen, key_radius, key_bg);
            painter.line_segment(
                [
                    key_rect_screen.min + vec2(7.0, 2.0) * tf.scale,
                    pos2(
                        key_rect_screen.max.x - 7.0 * tf.scale,
                        key_rect_screen.min.y + 2.0 * tf.scale,
                    ),
                ],
                Stroke::new(tf.scale, Color32::from_white_alpha(45)),
            );

            // Labels
            let (primary, shift_label, fn_label) = get_key_labels(row, col);

            let font_primary = FontId::new((16.0 * tf.scale).max(7.0), FontFamily::Monospace);
            let font_secondary = FontId::new((8.5 * tf.scale).max(5.5), FontFamily::Monospace);

            // Primary label in center
            let center_offset = if !shift_label.is_empty() || !fn_label.is_empty() {
                vec2(0.0, 3.0 * tf.scale)
            } else {
                vec2(0.0, 0.0)
            };
            painter.text(
                key_rect_screen.center() + center_offset,
                egui::Align2::CENTER_CENTER,
                primary,
                font_primary,
                text_color,
            );

            // Secondary label (top-left for shift/symbol)
            if !shift_label.is_empty() {
                let pos = key_rect_screen.min + vec2(3.0 * tf.scale, 2.0 * tf.scale);
                painter.text(
                    pos,
                    egui::Align2::LEFT_TOP,
                    shift_label,
                    font_secondary.clone(),
                    Color32::from_rgb(0x9C, 0xA3, 0xAF),
                );
            }

            // Fn label (top-right for Fn modifier, e.g. arrows, F-keys, Esc)
            if !fn_label.is_empty() && fn_label != primary {
                let pos = pos2(
                    key_rect_screen.max.x - 3.0 * tf.scale,
                    key_rect_screen.min.y + 2.0 * tf.scale,
                );
                painter.text(
                    pos,
                    egui::Align2::RIGHT_TOP,
                    fn_label,
                    font_secondary,
                    Color32::from_rgb(95, 185, 150), // Orange-amber for Fn actions
                );
            }
        }
    }
}
