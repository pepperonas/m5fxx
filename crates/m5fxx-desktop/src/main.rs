//! Main desktop simulator application for M5Stack Cardputer.

pub mod dev_panel;
pub mod device_renderer;
pub mod keyboard_mapping;

use dev_panel::{render_dev_panel, DevPanelState};
use device_renderer::{render_cardputer_device, DeviceViewTransform};
use eframe::egui::{
    self, vec2, Color32, ColorImage, Context, Event, Pos2, Rect, TextureFilter, TextureHandle,
    TextureOptions,
};
use m5fxx_app_demo::DemoApp;
use m5fxx_core::input::KeyCoord;
use m5fxx_core::storage::SdCardStorage;
use m5fxx_core::{CardputerHal, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use std::path::PathBuf;

pub struct CardputerSimulatorApp {
    hal: CardputerHal,
    app: DemoApp,
    dev_state: DevPanelState,
    display_texture: Option<TextureHandle>,
    rgba_buffer: Vec<u8>,
    active_mouse_key: Option<KeyCoord>,
    display_only_mode: bool,
    show_dev_panel: bool,
}

impl CardputerSimulatorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let sd_path = PathBuf::from("./virtual_sd");
        let storage = match SdCardStorage::new(&sd_path) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!(
                    "Failed to initialize virtual SD storage at {:?}: {}",
                    sd_path, e
                );
                None
            }
        };

        let mut hal = CardputerHal::new(storage);
        hal.log("Simulator initialized");

        let app = DemoApp::new();

        Self {
            hal,
            app,
            dev_state: DevPanelState::default(),
            display_texture: None,
            rgba_buffer: vec![0u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4],
            active_mouse_key: None,
            display_only_mode: false,
            show_dev_panel: true,
        }
    }

    /// Processes keyboard events from egui
    fn handle_host_input(&mut self, ctx: &Context) {
        // If window lost focus, immediately release all pressed keys to avoid stuck keys
        let has_focus = ctx.input(|i| i.focused);
        if !has_focus {
            self.hal.input.reset_all();
            self.active_mouse_key = None;
            return;
        }

        // Release mouse-clicked key if primary pointer is no longer down
        let pointer_down = ctx.input(|i| i.pointer.primary_down());
        if !pointer_down {
            if let Some(coord) = self.active_mouse_key.take() {
                self.hal.input.release_key(coord.row, coord.col);
            }
        }

        // Synchronize host modifier keys (Shift, Ctrl, Alt) with Cardputer matrix keys
        // Shift is at matrix (2, 1), Ctrl is at (3, 0), Alt is at (3, 2)
        let host_modifiers = ctx.input(|i| i.modifiers);
        let shift_coord = KeyCoord::new(2, 1);
        let ctrl_coord = KeyCoord::new(3, 0);
        let alt_coord = KeyCoord::new(3, 2);

        if host_modifiers.shift
            && !self
                .hal
                .input
                .is_key_pressed(shift_coord.row, shift_coord.col)
        {
            self.hal.input.press_key(shift_coord.row, shift_coord.col);
        } else if !host_modifiers.shift
            && self
                .hal
                .input
                .is_key_pressed(shift_coord.row, shift_coord.col)
        {
            self.hal.input.release_key(shift_coord.row, shift_coord.col);
        }

        if host_modifiers.ctrl
            && !self
                .hal
                .input
                .is_key_pressed(ctrl_coord.row, ctrl_coord.col)
        {
            self.hal.input.press_key(ctrl_coord.row, ctrl_coord.col);
        } else if !host_modifiers.ctrl
            && self
                .hal
                .input
                .is_key_pressed(ctrl_coord.row, ctrl_coord.col)
        {
            self.hal.input.release_key(ctrl_coord.row, ctrl_coord.col);
        }

        if host_modifiers.alt && !self.hal.input.is_key_pressed(alt_coord.row, alt_coord.col) {
            self.hal.input.press_key(alt_coord.row, alt_coord.col);
        } else if !host_modifiers.alt && self.hal.input.is_key_pressed(alt_coord.row, alt_coord.col)
        {
            self.hal.input.release_key(alt_coord.row, alt_coord.col);
        }

        // Process raw host keyboard events
        let events = ctx.input(|i| i.events.clone());
        for ev in events {
            match ev {
                Event::Key {
                    key,
                    pressed,
                    repeat,
                    ..
                } => {
                    let special_override = keyboard_mapping::host_key_to_special(key);
                    if let Some(coord) = keyboard_mapping::host_key_to_matrix(key) {
                        if pressed {
                            if !repeat {
                                self.hal.input.press_key_override(
                                    coord.row,
                                    coord.col,
                                    special_override,
                                );
                            }
                        } else {
                            self.hal.input.release_key(coord.row, coord.col);
                        }
                    } else if pressed && !repeat {
                        if let Some(spec) = special_override {
                            self.hal.input.press_cardputer_key(spec);
                        }
                    }
                }
                Event::Text(txt) => {
                    // Send unicode text input characters if they are printable and not control characters
                    // like '\n', '\r', '\t', '\x08' which are already handled via Key::Enter, Tab, Backspace.
                    for c in txt.chars() {
                        if c >= ' ' && c != '\x7F' {
                            if let Some(coord) = keyboard_mapping::char_to_matrix(c) {
                                // Direct press & release for typed characters (e.g. from German/QWERTZ layout)
                                self.hal.input.press_key(coord.row, coord.col);
                                self.hal.input.release_key(coord.row, coord.col);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Process Drag and Drop dropped files (e.g. .bin, .hex firmware images)
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped_files {
            let file_name = if !file.name.is_empty() {
                file.name.clone()
            } else if let Some(path) = &file.path {
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "firmware.bin".to_string())
            } else {
                "firmware.bin".to_string()
            };

            let file_size = if let Some(bytes) = &file.bytes {
                bytes.len()
            } else if let Some(path) = &file.path {
                std::fs::metadata(path)
                    .map(|m| m.len() as usize)
                    .unwrap_or(0)
            } else {
                0
            };

            self.hal.log(format!(
                "Drag & Drop: Received '{}' ({} bytes)",
                file_name, file_size
            ));
            self.dev_state.installed_firmwares.push(file_name.clone());
            self.app.trigger_firmware_flash(file_name, file_size);
        }
    }

    /// Uploads the current DisplayBuffer to egui GPU texture using Nearest-Neighbor filtering
    fn update_texture(&mut self, ctx: &Context) {
        if self.hal.display.is_dirty() || self.display_texture.is_none() {
            self.hal.display.to_rgba8888(&mut self.rgba_buffer);
            let color_image = ColorImage::from_rgba_unmultiplied(
                [DISPLAY_WIDTH, DISPLAY_HEIGHT],
                &self.rgba_buffer,
            );

            // Use Nearest Neighbor filtering for sharp, pixel-perfect display rendering
            let options = TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Nearest,
                ..Default::default()
            };

            match &mut self.display_texture {
                Some(handle) => handle.set(color_image, options),
                None => {
                    self.display_texture =
                        Some(ctx.load_texture("cardputer_display", color_image, options));
                }
            }
            self.hal.display.mark_clean();
        }
    }
}

impl eframe::App for CardputerSimulatorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Continuous redraw at target rate
        ctx.request_repaint();

        self.dev_state.update_fps();
        self.handle_host_input(ctx);

        // Advance simulation tick if not paused
        let dt = self.hal.update();
        if !self.dev_state.is_paused {
            self.app.update(&mut self.hal, dt);
        }

        // Refresh GPU texture
        self.update_texture(ctx);

        let texture_id = self
            .display_texture
            .as_ref()
            .expect("Texture should be initialized")
            .id();

        // Top bar for quick toggles
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("m5fxx - M5Stack Cardputer Simulator");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(self.show_dev_panel, "🛠 Developer Panel")
                        .clicked()
                    {
                        self.show_dev_panel = !self.show_dev_panel;
                    }
                    if ui
                        .selectable_label(self.display_only_mode, "🔍 Zoomed Display Only")
                        .clicked()
                    {
                        self.display_only_mode = !self.display_only_mode;
                    }
                });
            });
        });

        // Collapsible Developer Side Panel
        if self.show_dev_panel {
            egui::SidePanel::right("dev_panel")
                .resizable(true)
                .default_width(320.0)
                .min_width(260.0)
                .show(ctx, |ui| {
                    render_dev_panel(
                        ui,
                        &mut self.dev_state,
                        &mut self.hal,
                        &mut self.app,
                        &mut self.display_only_mode,
                    );
                });
        }

        // Central viewport: Cardputer Vector Device or Zoomed Display
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();

            if self.display_only_mode {
                // Render solely the display stretched with integer pixel scaling
                let max_w = available_rect.width();
                let max_h = available_rect.height();

                // Compute maximum integer scale factor that fits
                let int_scale = ((max_w / DISPLAY_WIDTH as f32)
                    .min(max_h / DISPLAY_HEIGHT as f32)
                    .floor() as i32)
                    .max(1);

                let target_w = (DISPLAY_WIDTH as i32 * int_scale) as f32;
                let target_h = (DISPLAY_HEIGHT as i32 * int_scale) as f32;

                let offset_x = available_rect.min.x + (max_w - target_w) * 0.5;
                let offset_y = available_rect.min.y + (max_h - target_h) * 0.5;
                let screen_rect =
                    Rect::from_min_size(egui::pos2(offset_x, offset_y), vec2(target_w, target_h));

                let painter = ui.painter();
                painter.rect_filled(available_rect, 0.0, Color32::from_rgb(0x10, 0x10, 0x14));
                painter.image(
                    texture_id,
                    screen_rect,
                    Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.rect_stroke(
                    screen_rect,
                    0.0,
                    egui::Stroke::new(2.0_f32, Color32::from_rgb(0x3B, 0x82, 0xF6)),
                    egui::StrokeKind::Outside,
                );

                let info = format!(
                    "Pixel Scale: {}x ({}x{} px)",
                    int_scale,
                    DISPLAY_WIDTH as i32 * int_scale,
                    DISPLAY_HEIGHT as i32 * int_scale
                );
                painter.text(
                    egui::pos2(screen_rect.min.x + 8.0, screen_rect.min.y + 8.0),
                    egui::Align2::LEFT_TOP,
                    info,
                    egui::FontId::monospace(14.0),
                    Color32::YELLOW,
                );
            } else {
                // Render full realistic M5Stack Cardputer hardware device
                ui.painter()
                    .rect_filled(available_rect, 0.0, Color32::from_rgb(42, 47, 49));
                let tf = DeviceViewTransform::new(available_rect.shrink(32.0));
                render_cardputer_device(
                    ui,
                    &tf,
                    &mut self.hal,
                    texture_id,
                    &mut self.active_mouse_key,
                );
            }

            // Visual feedback when hovering a dragged file over the simulator window
            let hovered_files = ctx.input(|i| i.raw.hovered_files.clone());
            if !hovered_files.is_empty() {
                let painter = ui.painter();
                painter.rect_filled(
                    available_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(0x10, 0x14, 0x20, 200),
                );
                painter.rect_stroke(
                    available_rect.shrink(16.0),
                    8.0,
                    egui::Stroke::new(3.0_f32, Color32::from_rgb(0xFA, 0x6A, 0x00)),
                    egui::StrokeKind::Inside,
                );

                let first_file = hovered_files
                    .first()
                    .and_then(|f| f.path.as_ref())
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "firmware.bin".to_string());

                let text = format!("⚡ Drop '{}' to Flash Firmware", first_file);
                painter.text(
                    available_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::monospace(22.0),
                    Color32::WHITE,
                );
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("M5Stack Cardputer Simulator (m5fxx)")
            .with_inner_size([1120.0, 680.0])
            .with_min_inner_size([720.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "M5Stack Cardputer Simulator",
        native_options,
        Box::new(|cc| Ok(Box::new(CardputerSimulatorApp::new(cc)))),
    )
}
