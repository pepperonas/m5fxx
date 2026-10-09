//! Realistic vector rendering and layout metrics for the M5Stack Cardputer.
//!
//! Physical Cardputer Dimensions:
//! - 84.0 mm x 54.0 mm x 19.7 mm
//! - Display: 1.14" IPS LCD (240x135 pixels, active area ~24.9mm x 14.9mm)
//! - Keyboard: 56 keys arranged in 4 rows x 14 columns
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
pub const DISPLAY_BEZEL_X: f32 = 220.0;
pub const DISPLAY_BEZEL_Y: f32 = 36.0;
pub const DISPLAY_BEZEL_W: f32 = 400.0;
pub const DISPLAY_BEZEL_H: f32 = 236.0;

pub const SCREEN_INNER_X: f32 = 240.0;
pub const SCREEN_INNER_Y: f32 = 52.0;
pub const SCREEN_INNER_W: f32 = 360.0;
pub const SCREEN_INNER_H: f32 = 202.5; // Exactly 240:135 aspect ratio (16:9)

// Keyboard dimensions in design space
pub const KB_START_X: f32 = 48.0;
pub const KB_START_Y: f32 = 296.0;
pub const KB_TOTAL_W: f32 = 744.0;
pub const KB_TOTAL_H: f32 = 212.0;

pub const KEY_GAP_X: f32 = 5.0;
pub const KEY_GAP_Y: f32 = 6.0;
pub const KEY_W: f32 = (KB_TOTAL_W - (COLS as f32 - 1.0) * KEY_GAP_X) / COLS as f32; // ~48.5 pt
pub const KEY_H: f32 = (KB_TOTAL_H - (ROWS as f32 - 1.0) * KEY_GAP_Y) / ROWS as f32; // ~48.5 pt

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

    // 1. Device Outer Shadow
    let case_rect_design = Rect::from_min_size(pos2(0.0, 0.0), vec2(DESIGN_WIDTH, DESIGN_HEIGHT));
    let shadow_rect_screen = tf.to_screen_rect(case_rect_design.translate(vec2(8.0, 12.0)));
    let case_radius = ((CASE_CORNER_RADIUS as f32 * tf.scale) as u8).max(1);
    painter.rect_filled(
        shadow_rect_screen,
        CornerRadius::same(case_radius),
        Color32::from_black_alpha(70),
    );

    // 2. Main Case Body (Cardputer dark graphite industrial finish: #2C2C30)
    let case_rect_screen = tf.to_screen_rect(case_rect_design);
    let case_color = Color32::from_rgb(0x28, 0x2A, 0x2E);
    let case_bevel = Color32::from_rgb(0x3E, 0x41, 0x47);
    painter.rect_filled(
        case_rect_screen,
        CornerRadius::same(case_radius),
        case_color,
    );
    painter.rect_stroke(
        case_rect_screen,
        CornerRadius::same(case_radius),
        Stroke::new(2.5 * tf.scale, case_bevel),
        StrokeKind::Outside,
    );

    // 3. Side details: Screws / Accents
    let screw_color = Color32::from_rgb(0x75, 0x78, 0x82);
    let screw_positions = [
        pos2(24.0, 24.0),
        pos2(DESIGN_WIDTH - 24.0, 24.0),
        pos2(24.0, DESIGN_HEIGHT - 24.0),
        pos2(DESIGN_WIDTH - 24.0, DESIGN_HEIGHT - 24.0),
    ];
    for p in screw_positions {
        let sp = tf.to_screen_pos(p);
        painter.circle_filled(sp, 6.0 * tf.scale, screw_color);
        painter.circle_stroke(
            sp,
            6.0 * tf.scale,
            Stroke::new(1.0 * tf.scale, Color32::BLACK),
        );
    }

    // 4. M5Stack Brand Label & Model Badge
    let brand_pos = tf.to_screen_pos(pos2(60.0, 42.0));
    painter.text(
        brand_pos,
        egui::Align2::LEFT_TOP,
        "M5STACK",
        FontId::new(22.0 * tf.scale, FontFamily::Proportional),
        Color32::from_rgb(0xF0, 0x50, 0x22), // Official M5Stack Orange
    );
    let model_badge = match hal.status.model {
        CardputerModel::CardputerOriginal => "CARDPUTER",
        CardputerModel::CardputerAdv => "CARDPUTER ADV",
    };
    let model_pos = tf.to_screen_pos(pos2(60.0, 68.0));
    painter.text(
        model_pos,
        egui::Align2::LEFT_TOP,
        model_badge,
        FontId::new(15.0 * tf.scale, FontFamily::Monospace),
        Color32::from_rgb(0x9E, 0xA2, 0xAE),
    );

    // 5. Speaker Grille (Left of screen, perforated dots)
    let speaker_start_x = 60.0;
    let speaker_start_y = 104.0;
    for row in 0..5 {
        for col in 0..6 {
            let p = pos2(
                speaker_start_x + col as f32 * 14.0,
                speaker_start_y + row as f32 * 14.0,
            );
            painter.circle_filled(
                tf.to_screen_pos(p),
                2.5 * tf.scale,
                Color32::from_rgb(0x18, 0x19, 0x1B),
            );
        }
    }

    // 6. Top/Right Hardware Controls: BtnG0 (Download/Action button) and LED
    let btn_g0_rect_design =
        Rect::from_min_size(pos2(DESIGN_WIDTH - 150.0, 40.0), vec2(90.0, 36.0));
    let btn_g0_rect_screen = tf.to_screen_rect(btn_g0_rect_design);
    let g0_hover = ui.rect_contains_pointer(btn_g0_rect_screen);
    let g0_pressed = g0_hover && ui.input(|i| i.pointer.primary_down());

    if g0_pressed {
        hal.input.press_btn_g0();
    } else {
        hal.input.release_btn_g0();
    }

    let g0_color = if hal.input.btn_g0_pressed {
        Color32::from_rgb(0x40, 0x80, 0x50)
    } else if g0_hover {
        Color32::from_rgb(0x55, 0x58, 0x62)
    } else {
        Color32::from_rgb(0x3B, 0x3D, 0x44)
    };
    let g0_radius = ((6.0 * tf.scale) as u8).max(1);
    painter.rect_filled(btn_g0_rect_screen, CornerRadius::same(g0_radius), g0_color);
    painter.rect_stroke(
        btn_g0_rect_screen,
        CornerRadius::same(g0_radius),
        Stroke::new(1.5 * tf.scale, Color32::BLACK),
        StrokeKind::Inside,
    );
    painter.text(
        btn_g0_rect_screen.center(),
        egui::Align2::CENTER_CENTER,
        "G0 / BTN",
        FontId::new(13.0 * tf.scale, FontFamily::Monospace),
        Color32::WHITE,
    );

    // Power status LED (green glowing indicator)
    let led_pos = tf.to_screen_pos(pos2(DESIGN_WIDTH - 105.0, 95.0));
    painter.circle_filled(led_pos, 5.0 * tf.scale, Color32::from_rgb(0x22, 0xC5, 0x5E));
    painter.circle_stroke(
        led_pos,
        7.0 * tf.scale,
        Stroke::new(
            1.0 * tf.scale,
            Color32::from_rgba_premultiplied(34, 197, 94, 100),
        ),
    );
    painter.text(
        tf.to_screen_pos(pos2(DESIGN_WIDTH - 105.0, 110.0)),
        egui::Align2::CENTER_TOP,
        "PWR",
        FontId::new(10.0 * tf.scale, FontFamily::Monospace),
        Color32::from_rgb(0x75, 0x78, 0x82),
    );

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
        Color32::from_rgb(0x1C, 0x1D, 0x20),
    );
    painter.rect_stroke(
        kb_tray_screen,
        CornerRadius::same(tray_radius),
        Stroke::new(1.5 * tf.scale, Color32::from_rgb(0x10, 0x11, 0x13)),
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

            // Visual Key Cap styling
            let (key_bg, text_color) = if is_pressed_virtual {
                (Color32::from_rgb(0x3B, 0x82, 0xF6), Color32::WHITE) // Highlighted Bright Blue
            } else if is_hovered {
                (
                    Color32::from_rgb(0x45, 0x48, 0x52),
                    Color32::from_rgb(0xF3, 0xF4, 0xF6),
                )
            } else {
                // Color coding for special keys
                if (row == 2 && col == 0) || (row == 2 && col == 1) {
                    // Fn, Shift
                    (
                        Color32::from_rgb(0x32, 0x35, 0x3D),
                        Color32::from_rgb(0xFB, 0xBF, 0x24),
                    ) // Amber
                } else if row == 2 && col == 13 {
                    // Enter
                    (
                        Color32::from_rgb(0x2E, 0x3A, 0x4E),
                        Color32::from_rgb(0x60, 0xA5, 0xFA),
                    ) // Blue
                } else if row == 0 && col == 13 {
                    // Del
                    (
                        Color32::from_rgb(0x45, 0x2A, 0x2E),
                        Color32::from_rgb(0xF8, 0x71, 0x71),
                    ) // Red
                } else {
                    (
                        Color32::from_rgb(0x38, 0x3B, 0x43),
                        Color32::from_rgb(0xDD, 0xDF, 0xE5),
                    )
                }
            };

            // Keycap shape with tactile bevel
            let key_radius = ((4.0 * tf.scale) as u8).max(1);
            painter.rect_filled(key_rect_screen, CornerRadius::same(key_radius), key_bg);
            painter.rect_stroke(
                key_rect_screen,
                CornerRadius::same(key_radius),
                Stroke::new(1.0 * tf.scale, Color32::from_rgb(0x22, 0x24, 0x29)),
                StrokeKind::Inside,
            );

            // Labels
            let (primary, shift_label, fn_label) = get_key_labels(row, col);

            let font_primary = FontId::new((12.5 * tf.scale).max(7.0), FontFamily::Monospace);
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
                    Color32::from_rgb(0xF5, 0x9E, 0x0B), // Orange-amber for Fn actions
                );
            }
        }
    }
}
