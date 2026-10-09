//! Authentic vector rendering and layout metrics for the M5Stack Cardputer (Model K132).
//!
//! Physical Cardputer Dimensions & Reference:
//! - 84.0 mm x 54.0 mm x 19.7 mm (56 tactile buttons, 1.14" IPS LCD, M5Stamp-S3 MCU module)
//! - Display: 1.14" ST7789v2 IPS LCD (240x135 pixels, active area 24.9mm x 14.9mm)
//! - Keyboard: 56 keys arranged in 4 rows x 14 columns
//!
//! Visual Details from Official Hardware:
//! 1. Warm industrial light-gray moulded plastic enclosure (#C8CBD0 / #D2D5DC) with subtle bevels and screw recesses.
//! 2. Left Upper Section:
//!    - White screen-printed silkscreen badge with "[CARD COMPUTER]" and orange corner brackets
//!    - Microphone label "Mic  DataG46  ClkG43" with 2 oval acoustic slots
//! 3. Center Upper Section:
//!    - Recessed glossy black acrylic bezel enclosing the 240x135 LCD screen
//!    - Indicator dots on the left edge ("Aa", "fn", "ctrl", "opt", "alt")
//!    - Embossed/printed "M5" logo on the right edge of the bezel
//! 4. Right Upper Section:
//!    - Authentic M5Stamp-S3 system-on-module with visible PCB surface (#1E232A)
//!    - 2.4GHz PCB antenna trace in gold/copper
//!    - Laser-marked metal shield "STAMP S3 / ESP32-S3FN8"
//!    - Side pinouts with color-coded dot matrix (G1, G2, G41, G42, 5V, GND, 3V3, etc.)
//!    - Tactile G0 user button with gold ring accent
//!    - Side USB-C metallic receptacle visible on edge
//! 5. Hardware Screws:
//!    - Two dark metallic M2 hex/Torx screw recesses directly below display and stamp
//! 6. Keyboard Deck:
//!    - Clean recessed tray with 56 black rounded-pill keycaps
//!    - Authentic white legends with orange arrow glyphs (▲ ▼ ◄ ►) and orange "ok ↵" accent

use egui::{
    pos2, vec2, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Stroke, StrokeKind, Ui,
};
use m5fxx_core::input::{get_key_labels, KeyCoord, COLS, ROWS};
use m5fxx_core::CardputerHal;

pub const DESIGN_WIDTH: f32 = 840.0;
pub const DESIGN_HEIGHT: f32 = 540.0;

// Display bezel in design space
pub const DISPLAY_BEZEL_X: f32 = 188.0;
pub const DISPLAY_BEZEL_Y: f32 = 36.0;
pub const DISPLAY_BEZEL_W: f32 = 340.0;
pub const DISPLAY_BEZEL_H: f32 = 196.0;

// LCD active screen area (240x135 aspect ratio 16:9)
pub const SCREEN_INNER_X: f32 = 226.0;
pub const SCREEN_INNER_Y: f32 = 55.0;
pub const SCREEN_INNER_W: f32 = 264.0;
pub const SCREEN_INNER_H: f32 = 148.5;

// Keyboard dimensions in design space
pub const KB_START_X: f32 = 36.0;
pub const KB_START_Y: f32 = 262.0;
pub const KB_TOTAL_W: f32 = 768.0;
pub const KB_TOTAL_H: f32 = 244.0;

pub const KEY_GAP_X: f32 = 9.0;
pub const KEY_GAP_Y: f32 = 11.0;
pub const KEY_W: f32 = (KB_TOTAL_W - (COLS as f32 - 1.0) * KEY_GAP_X) / COLS as f32; // ~46.5 pt
pub const KEY_H: f32 = (KB_TOTAL_H - (ROWS as f32 - 1.0) * KEY_GAP_Y) / ROWS as f32; // ~52.7 pt

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
        let scale = scale_x.min(scale_y).max(0.2); // Maintain aspect ratio

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

/// Renders the complete, authentic vector body of the M5Stack Cardputer
pub fn render_cardputer_device(
    ui: &mut Ui,
    tf: &DeviceViewTransform,
    hal: &mut CardputerHal,
    display_texture_id: egui::TextureId,
    clicked_key: &mut Option<KeyCoord>,
) {
    let painter = ui.painter();
    let radius = |r: f32| CornerRadius::same(((r * tf.scale).max(1.0)) as u8);

    // =========================================================================
    // 1. DROP SHADOW (realistic soft depth)
    // =========================================================================
    let body_rect = Rect::from_min_size(Pos2::ZERO, vec2(DESIGN_WIDTH, DESIGN_HEIGHT));
    for (spread, offset_y, alpha) in [(24.0, 18.0, 18), (14.0, 10.0, 28), (6.0, 5.0, 40)] {
        painter.rect_filled(
            tf.to_screen_rect(body_rect.translate(vec2(0.0, offset_y)).expand(spread)),
            radius(34.0),
            Color32::from_black_alpha(alpha),
        );
    }

    // =========================================================================
    // 2. MAIN ENCLOSURE (Authentic light industrial warm-gray ABS plastic)
    // =========================================================================
    // Darker chassis bottom rim
    painter.rect_filled(
        tf.to_screen_rect(body_rect),
        radius(30.0),
        Color32::from_rgb(0x9E, 0xA2, 0xAA),
    );
    // Beveled mid-case ridge
    painter.rect_filled(
        tf.to_screen_rect(body_rect.shrink2(vec2(2.5, 3.0))),
        radius(28.0),
        Color32::from_rgb(0xB8, 0xBC, 0xC4),
    );
    // Main top faceplate
    let face_rect = body_rect.shrink2(vec2(5.0, 6.0));
    painter.rect_filled(
        tf.to_screen_rect(face_rect),
        radius(26.0),
        Color32::from_rgb(0xD1, 0xD4, 0xDC), // Authentic Cardputer light-gray
    );
    // Subtle top specular highlight
    painter.rect_stroke(
        tf.to_screen_rect(face_rect.shrink(1.5)),
        radius(25.0),
        Stroke::new(1.5 * tf.scale, Color32::from_rgb(0xEE, 0xF0, 0xF5)),
        StrokeKind::Inside,
    );

    // =========================================================================
    // 3. CASE PERIPHERAL DETAILS (USB-C cutout on right edge, MicroSD slot on left)
    // =========================================================================
    // USB-C metal port visible at top right edge
    let usb_c_slot = Rect::from_min_size(pos2(DESIGN_WIDTH - 6.0, 80.0), vec2(6.0, 48.0));
    painter.rect_filled(
        tf.to_screen_rect(usb_c_slot),
        radius(3.0),
        Color32::from_rgb(0x60, 0x64, 0x6C),
    );
    let usb_c_inner = Rect::from_min_size(pos2(DESIGN_WIDTH - 5.0, 86.0), vec2(4.0, 36.0));
    painter.rect_filled(
        tf.to_screen_rect(usb_c_inner),
        radius(2.0),
        Color32::from_rgb(0x28, 0x2A, 0x2F),
    );

    // MicroSD slot on bottom edge left
    let sd_slot = Rect::from_min_size(pos2(56.0, DESIGN_HEIGHT - 6.0), vec2(44.0, 6.0));
    painter.rect_filled(
        tf.to_screen_rect(sd_slot),
        radius(2.0),
        Color32::from_rgb(0x40, 0x43, 0x48),
    );

    // =========================================================================
    // 4. UPPER LEFT SECTION: [CARD COMPUTER] Badge & Mic Apertures
    // =========================================================================
    let badge_rect = Rect::from_min_size(pos2(32.0, 36.0), vec2(138.0, 196.0));
    // White textured silk badge inlay
    painter.rect_filled(
        tf.to_screen_rect(badge_rect),
        radius(12.0),
        Color32::from_rgb(0xFA, 0xFA, 0xFC),
    );
    painter.rect_stroke(
        tf.to_screen_rect(badge_rect),
        radius(12.0),
        Stroke::new(1.0 * tf.scale, Color32::from_rgb(0xCD, 0xD0, 0xD8)),
        StrokeKind::Inside,
    );

    // Orange corner brackets decoration
    let draw_corner_bracket = |painter: &egui::Painter, p: Pos2, vx: f32, vy: f32| {
        let sp = tf.to_screen_pos(p);
        let s_vx = vx * tf.scale;
        let s_vy = vy * tf.scale;
        let stroke = Stroke::new(2.2 * tf.scale, Color32::from_rgb(0xFA, 0x6A, 0x00));
        painter.line_segment([sp, sp + vec2(s_vx, 0.0)], stroke);
        painter.line_segment([sp, sp + vec2(0.0, s_vy)], stroke);
    };
    draw_corner_bracket(painter, pos2(40.0, 44.0), 12.0, 12.0);
    draw_corner_bracket(painter, pos2(162.0, 44.0), -12.0, 12.0);
    draw_corner_bracket(painter, pos2(40.0, 224.0), 12.0, -12.0);
    draw_corner_bracket(painter, pos2(162.0, 224.0), -12.0, -12.0);

    // Badge Title "[ CARD COMPUTER ]"
    painter.text(
        tf.to_screen_pos(pos2(101.0, 56.0)),
        egui::Align2::CENTER_CENTER,
        "[ CARD COMPUTER ]",
        FontId::new((12.5 * tf.scale).max(7.0), FontFamily::Monospace),
        Color32::from_rgb(0x1A, 0x1E, 0x24),
    );

    // Decorative circuit trace / tech graphic in badge center
    let trace_y = 86.0;
    painter.line_segment(
        [
            tf.to_screen_pos(pos2(48.0, trace_y)),
            tf.to_screen_pos(pos2(154.0, trace_y)),
        ],
        Stroke::new(1.2 * tf.scale, Color32::from_rgb(0xD0, 0xD4, 0xDE)),
    );

    // Microphone details (SPM1423 PDM microphone)
    painter.text(
        tf.to_screen_pos(pos2(101.0, 108.0)),
        egui::Align2::CENTER_CENTER,
        "PDM MIC",
        FontId::new((11.0 * tf.scale).max(6.5), FontFamily::Monospace),
        Color32::from_rgb(0x3B, 0x42, 0x4E),
    );
    painter.text(
        tf.to_screen_pos(pos2(101.0, 126.0)),
        egui::Align2::CENTER_CENTER,
        "Data: G46",
        FontId::new((9.5 * tf.scale).max(6.0), FontFamily::Monospace),
        Color32::from_rgb(0x70, 0x76, 0x84),
    );
    painter.text(
        tf.to_screen_pos(pos2(101.0, 140.0)),
        egui::Align2::CENTER_CENTER,
        "Clk:  G43",
        FontId::new((9.5 * tf.scale).max(6.0), FontFamily::Monospace),
        Color32::from_rgb(0x70, 0x76, 0x84),
    );

    // Two authentic horizontal oval mic acoustic holes
    for mic_y in [168.0, 184.0] {
        let mic_rect = Rect::from_center_size(pos2(101.0, mic_y), vec2(28.0, 6.5));
        painter.rect_filled(
            tf.to_screen_rect(mic_rect),
            radius(3.2),
            Color32::from_rgb(0x2B, 0x2E, 0x34),
        );
        painter.rect_stroke(
            tf.to_screen_rect(mic_rect),
            radius(3.2),
            Stroke::new(1.0 * tf.scale, Color32::from_rgb(0x50, 0x55, 0x60)),
            StrokeKind::Inside,
        );
    }

    // Small speaker grille dots on lower left
    for i in 0..4 {
        painter.circle_filled(
            tf.to_screen_pos(pos2(74.0 + i as f32 * 18.0, 208.0)),
            2.2 * tf.scale,
            Color32::from_rgb(0x8A, 0x90, 0x9C),
        );
    }

    // =========================================================================
    // 5. CENTER UPPER SECTION: Black Display Bezel & 240x135 IPS Screen
    // =========================================================================
    let bezel_rect_design = Rect::from_min_size(
        pos2(DISPLAY_BEZEL_X, DISPLAY_BEZEL_Y),
        vec2(DISPLAY_BEZEL_W, DISPLAY_BEZEL_H),
    );
    let bezel_rect_screen = tf.to_screen_rect(bezel_rect_design);
    let bezel_rad = radius(12.0);

    // Recessed dark bezel shadow
    painter.rect_filled(
        tf.to_screen_rect(bezel_rect_design.expand(2.0)),
        radius(14.0),
        Color32::from_rgb(0xA8, 0xAC, 0xB5),
    );
    // Glossy black acrylic bezel
    painter.rect_filled(
        bezel_rect_screen,
        bezel_rad,
        Color32::from_rgb(0x14, 0x15, 0x18),
    );
    painter.rect_stroke(
        bezel_rect_screen,
        bezel_rad,
        Stroke::new(1.5 * tf.scale, Color32::from_rgb(0x28, 0x2A, 0x30)),
        StrokeKind::Inside,
    );

    // Left bezel status indicators (Aa, fn, ctrl, opt, alt)
    let indicators = [
        ("Aa", hal.input.shift_active),
        ("fn", hal.input.fn_active),
        ("ctrl", hal.input.ctrl_active),
        ("opt", hal.input.opt_active),
        ("alt", hal.input.alt_active),
    ];
    for (i, (label, active)) in indicators.iter().enumerate() {
        let ind_y = 66.0 + i as f32 * 27.0;
        let color = if *active {
            Color32::from_rgb(0x00, 0xE6, 0x76) // Bright green when active
        } else {
            Color32::from_rgb(0x4A, 0x50, 0x5C) // Dim grey
        };
        painter.circle_filled(tf.to_screen_pos(pos2(202.0, ind_y)), 2.8 * tf.scale, color);
        painter.text(
            tf.to_screen_pos(pos2(212.0, ind_y)),
            egui::Align2::LEFT_CENTER,
            *label,
            FontId::new((8.5 * tf.scale).max(5.0), FontFamily::Monospace),
            if *active {
                Color32::WHITE
            } else {
                Color32::from_rgb(0x72, 0x78, 0x86)
            },
        );
    }

    // Embossed "M5" logo on right bezel margin
    let m5_logo_rect = Rect::from_center_size(pos2(508.0, 134.0), vec2(20.0, 36.0));
    painter.rect_filled(
        tf.to_screen_rect(m5_logo_rect),
        radius(4.0),
        Color32::from_rgb(0x22, 0x24, 0x2A),
    );
    painter.text(
        tf.to_screen_pos(pos2(508.0, 134.0)),
        egui::Align2::CENTER_CENTER,
        "M5",
        FontId::new((12.0 * tf.scale).max(7.0), FontFamily::Monospace),
        Color32::from_rgb(0xFA, 0x6A, 0x00), // Orange M5 accent
    );

    // Active IPS Screen viewport (ST7789 240x135)
    let screen_rect_design = Rect::from_min_size(
        pos2(SCREEN_INNER_X, SCREEN_INNER_Y),
        vec2(SCREEN_INNER_W, SCREEN_INNER_H),
    );
    let screen_rect_screen = tf.to_screen_rect(screen_rect_design);

    // Framebuffer Texture Image
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    painter.image(display_texture_id, screen_rect_screen, uv, Color32::WHITE);

    // LCD inner rim shadow border
    painter.rect_stroke(
        screen_rect_screen,
        CornerRadius::ZERO,
        Stroke::new(1.5 * tf.scale, Color32::from_rgb(0x0A, 0x0C, 0x10)),
        StrokeKind::Inside,
    );

    // =========================================================================
    // 6. RIGHT UPPER SECTION: Authentic M5Stamp-S3 Module
    // =========================================================================
    let stamp_rect = Rect::from_min_size(pos2(544.0, 36.0), vec2(260.0, 196.0));
    let stamp_rad = radius(10.0);

    // Dark PCB base plate
    painter.rect_filled(
        tf.to_screen_rect(stamp_rect),
        stamp_rad,
        Color32::from_rgb(0x1B, 0x20, 0x26), // Matte black PCB
    );
    painter.rect_stroke(
        tf.to_screen_rect(stamp_rect),
        stamp_rad,
        Stroke::new(1.2 * tf.scale, Color32::from_rgb(0x32, 0x3A, 0x44)),
        StrokeKind::Inside,
    );

    // Gold PCB 2.4GHz Antenna meander trace at top of Stamp
    let ant_rect = Rect::from_min_size(pos2(560.0, 44.0), vec2(140.0, 22.0));
    painter.rect_filled(
        tf.to_screen_rect(ant_rect),
        radius(3.0),
        Color32::from_rgb(0x28, 0x30, 0x3A),
    );
    // Meander gold line segments
    let gold_stroke = Stroke::new(1.8 * tf.scale, Color32::from_rgb(0xD4, 0xAF, 0x37));
    for i in 0..7 {
        let x0 = 568.0 + i as f32 * 18.0;
        painter.line_segment(
            [
                tf.to_screen_pos(pos2(x0, 48.0)),
                tf.to_screen_pos(pos2(x0 + 8.0, 48.0)),
            ],
            gold_stroke,
        );
        painter.line_segment(
            [
                tf.to_screen_pos(pos2(x0 + 8.0, 48.0)),
                tf.to_screen_pos(pos2(x0 + 8.0, 60.0)),
            ],
            gold_stroke,
        );
        painter.line_segment(
            [
                tf.to_screen_pos(pos2(x0 + 8.0, 60.0)),
                tf.to_screen_pos(pos2(x0 + 16.0, 60.0)),
            ],
            gold_stroke,
        );
    }

    // Metal RF Shield Can "STAMP S3 / ESP32-S3FN8"
    let shield_rect = Rect::from_min_size(pos2(560.0, 74.0), vec2(150.0, 102.0));
    painter.rect_filled(
        tf.to_screen_rect(shield_rect),
        radius(5.0),
        Color32::from_rgb(0x7D, 0x84, 0x8E), // Nickel-silver RF shield
    );
    painter.rect_stroke(
        tf.to_screen_rect(shield_rect),
        radius(5.0),
        Stroke::new(1.2 * tf.scale, Color32::from_rgb(0x9E, 0xA6, 0xB2)),
        StrokeKind::Inside,
    );

    // Laser engravings on RF Shield
    painter.text(
        tf.to_screen_pos(pos2(635.0, 94.0)),
        egui::Align2::CENTER_CENTER,
        "M5 STAMP S3",
        FontId::new((12.0 * tf.scale).max(6.5), FontFamily::Monospace),
        Color32::from_rgb(0x35, 0x3B, 0x44),
    );
    painter.text(
        tf.to_screen_pos(pos2(635.0, 114.0)),
        egui::Align2::CENTER_CENTER,
        "ESP32-S3FN8",
        FontId::new((10.5 * tf.scale).max(6.0), FontFamily::Monospace),
        Color32::from_rgb(0x40, 0x47, 0x52),
    );
    painter.text(
        tf.to_screen_pos(pos2(635.0, 132.0)),
        egui::Align2::CENTER_CENTER,
        "2.4G Wi-Fi & BLE 5",
        FontId::new((8.5 * tf.scale).max(5.0), FontFamily::Monospace),
        Color32::from_rgb(0x50, 0x58, 0x64),
    );

    // Color-coded GPIO pin header dots on right of Stamp
    let pin_colors = [
        Color32::from_rgb(0xEF, 0x44, 0x44), // 5V Red
        Color32::from_rgb(0xF5, 0x9E, 0x0B), // 3V3 Orange
        Color32::from_rgb(0x10, 0xB9, 0x81), // GPIO Green
        Color32::from_rgb(0x3B, 0x82, 0xF6), // GPIO Blue
        Color32::from_rgb(0x8B, 0x5C, 0xF6), // GPIO Purple
        Color32::from_rgb(0x4B, 0x55, 0x63), // GND Dark
    ];
    for col_idx in 0..2 {
        for row_idx in 0..6 {
            let px = 730.0 + col_idx as f32 * 18.0;
            let py = 52.0 + row_idx as f32 * 19.0;
            let color = pin_colors[(col_idx * 6 + row_idx) % pin_colors.len()];
            painter.circle_filled(tf.to_screen_pos(pos2(px, py)), 4.0 * tf.scale, color);
            painter.circle_filled(
                tf.to_screen_pos(pos2(px, py)),
                2.0 * tf.scale,
                Color32::from_rgb(0xD4, 0xAF, 0x37), // Gold pad center
            );
        }
    }

    // Physical G0 User Pushbutton (below RF shield)
    let g0_btn_rect = Rect::from_center_size(pos2(635.0, 202.0), vec2(78.0, 24.0));
    let g0_screen_rect = tf.to_screen_rect(g0_btn_rect);
    let g0_hovered = ui.rect_contains_pointer(g0_screen_rect);
    let pointer_down = ui.input(|i| i.pointer.primary_down());

    if g0_hovered && pointer_down {
        hal.input.press_btn_g0();
    } else {
        hal.input.release_btn_g0();
    }

    let g0_bg = if hal.input.btn_g0_pressed {
        Color32::from_rgb(0xFA, 0x6A, 0x00) // Lit orange when pressed
    } else if g0_hovered {
        Color32::from_rgb(0x52, 0x5A, 0x66)
    } else {
        Color32::from_rgb(0x38, 0x3E, 0x48)
    };
    painter.rect_filled(g0_screen_rect, radius(5.0), g0_bg);
    painter.rect_stroke(
        g0_screen_rect,
        radius(5.0),
        Stroke::new(1.2 * tf.scale, Color32::from_rgb(0xD4, 0xAF, 0x37)), // Gold border
        StrokeKind::Inside,
    );
    painter.text(
        g0_screen_rect.center(),
        egui::Align2::CENTER_CENTER,
        "BTN G0",
        FontId::new((11.0 * tf.scale).max(6.5), FontFamily::Monospace),
        Color32::WHITE,
    );

    // =========================================================================
    // 7. HARDWARE M2 SCREW FASTENERS (Two hex/torx screws below display)
    // =========================================================================
    for screw_pos in [pos2(212.0, 246.0), pos2(626.0, 246.0)] {
        let center = tf.to_screen_pos(screw_pos);
        // Outer recess
        painter.circle_filled(center, 7.5 * tf.scale, Color32::from_rgb(0xA2, 0xA6, 0xAF));
        // Screw head
        painter.circle_filled(center, 5.5 * tf.scale, Color32::from_rgb(0x4A, 0x4E, 0x56));
        // Hex/Torx socket indent
        painter.circle_filled(center, 2.4 * tf.scale, Color32::from_rgb(0x22, 0x25, 0x2B));
    }

    // =========================================================================
    // 8. KEYBOARD TRAY (Recessed white mounting deck)
    // =========================================================================
    let kb_tray_design = Rect::from_min_size(
        pos2(KB_START_X - 8.0, KB_START_Y - 8.0),
        vec2(KB_TOTAL_W + 16.0, KB_TOTAL_H + 16.0),
    );
    let kb_tray_screen = tf.to_screen_rect(kb_tray_design);
    let tray_radius = radius(12.0);

    // Recessed shadow border
    painter.rect_filled(
        kb_tray_screen,
        tray_radius,
        Color32::from_rgb(0xB8, 0xBC, 0xC4),
    );
    // Pure white keyboard base tray
    painter.rect_filled(
        tf.to_screen_rect(kb_tray_design.shrink(2.0)),
        tray_radius,
        Color32::from_rgb(0xF4, 0xF5, 0xF8),
    );
    painter.rect_stroke(
        kb_tray_screen,
        tray_radius,
        Stroke::new(1.2 * tf.scale, Color32::from_rgb(0x9E, 0xA2, 0xAA)),
        StrokeKind::Inside,
    );

    // =========================================================================
    // 9. 56 TACTILE KEYCAPS (4x14 Matrix with authentic Cardputer legends)
    // =========================================================================
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

    for row in 0..ROWS {
        for col in 0..COLS {
            let key_rect_design = get_key_rect_design(row, col);
            let key_rect_screen = tf.to_screen_rect(key_rect_design);

            let is_hovered = pointer_pos.is_some_and(|pos| key_rect_screen.contains(pos));
            let is_pressed_virtual = hal.input.is_key_pressed(row as u8, col as u8);

            // Click handling with mouse
            if is_hovered && pointer_down {
                *clicked_key = Some(KeyCoord::new(row as u8, col as u8));
                hal.input.press_key(row as u8, col as u8);
            }

            // Authentic Cardputer Keycap Colors:
            // Tactile matte black caps (#181A1D)
            // Accent keys:
            //   Enter (2, 13) has orange "ok" legend
            //   Arrows (2,11 Up, 3,10 Left, 3,11 Down, 3,12 Right) have orange arrows
            //   Fn key (2, 0)
            let key_bg = if is_pressed_virtual {
                Color32::from_rgb(0xFA, 0x6A, 0x00) // Vivid M5 orange when pressed
            } else if is_hovered {
                Color32::from_rgb(0x36, 0x3A, 0x42)
            } else {
                Color32::from_rgb(0x18, 0x1A, 0x1D) // Standard deep black tactile cap
            };

            let cap_rad = radius(8.0);

            // Keycap lower drop-shadow / tactile bevel
            painter.rect_filled(
                key_rect_screen.translate(vec2(0.0, 2.5 * tf.scale)),
                cap_rad,
                Color32::from_rgb(0x0C, 0x0E, 0x10),
            );
            // Keycap top body
            painter.rect_filled(key_rect_screen, cap_rad, key_bg);

            // Top edge bevel highlight
            if !is_pressed_virtual {
                painter.line_segment(
                    [
                        key_rect_screen.min + vec2(6.0 * tf.scale, 2.0 * tf.scale),
                        pos2(
                            key_rect_screen.max.x - 6.0 * tf.scale,
                            key_rect_screen.min.y + 2.0 * tf.scale,
                        ),
                    ],
                    Stroke::new(1.0 * tf.scale, Color32::from_white_alpha(35)),
                );
            }

            // Labels for this key
            let (primary, shift_label, fn_label) = get_key_labels(row, col);

            let font_primary = FontId::new((15.5 * tf.scale).max(6.8), FontFamily::Monospace);
            let font_secondary = FontId::new((8.5 * tf.scale).max(4.8), FontFamily::Monospace);

            // Check if this is an arrow key on the Fn layer
            let is_arrow = matches!((row, col), (2, 11) | (3, 10) | (3, 11) | (3, 12));
            let arrow_symbol = match (row, col) {
                (2, 11) => Some("▲"),
                (3, 10) => Some("◄"),
                (3, 11) => Some("▼"),
                (3, 12) => Some("►"),
                _ => None,
            };

            let primary_text_color = if is_pressed_virtual || (row == 2 && col == 13) {
                Color32::WHITE
            } else {
                Color32::from_rgb(0xEB, 0xED, 0xF2) // Crisp white silk legend
            };

            // Primary label in center / center-bottom
            let center_offset = if is_arrow || !shift_label.is_empty() || !fn_label.is_empty() {
                vec2(0.0, 3.5 * tf.scale)
            } else {
                vec2(0.0, 0.0)
            };

            painter.text(
                key_rect_screen.center() + center_offset,
                egui::Align2::CENTER_CENTER,
                primary,
                font_primary,
                primary_text_color,
            );

            // Shift / secondary symbol in upper left (light grey)
            if !shift_label.is_empty() {
                let pos = key_rect_screen.min + vec2(4.0 * tf.scale, 3.0 * tf.scale);
                painter.text(
                    pos,
                    egui::Align2::LEFT_TOP,
                    shift_label,
                    font_secondary.clone(),
                    if is_pressed_virtual {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(0x9E, 0xA4, 0xB0)
                    },
                );
            }

            // Orange arrow glyph or Fn label in upper right
            if let Some(arrow) = arrow_symbol {
                let pos = pos2(
                    key_rect_screen.max.x - 4.0 * tf.scale,
                    key_rect_screen.min.y + 3.0 * tf.scale,
                );
                painter.text(
                    pos,
                    egui::Align2::RIGHT_TOP,
                    arrow,
                    font_secondary.clone(),
                    if is_pressed_virtual {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(0xFA, 0x6A, 0x00) // Cardputer signature orange arrow
                    },
                );
            } else if row == 2 && col == 13 {
                // "ok" legend on Enter key in orange
                let pos = pos2(
                    key_rect_screen.max.x - 4.0 * tf.scale,
                    key_rect_screen.min.y + 3.0 * tf.scale,
                );
                painter.text(
                    pos,
                    egui::Align2::RIGHT_TOP,
                    "ok",
                    font_secondary.clone(),
                    if is_pressed_virtual {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(0xFA, 0x6A, 0x00)
                    },
                );
            } else if !fn_label.is_empty() && fn_label != primary {
                let pos = pos2(
                    key_rect_screen.max.x - 4.0 * tf.scale,
                    key_rect_screen.min.y + 3.0 * tf.scale,
                );
                painter.text(
                    pos,
                    egui::Align2::RIGHT_TOP,
                    fn_label,
                    font_secondary,
                    if is_pressed_virtual {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(0x3B, 0x82, 0xF6) // Cyan/blue Fn legend
                    },
                );
            }
        }
    }
}
