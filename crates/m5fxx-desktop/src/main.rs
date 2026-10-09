//! Main desktop simulator application for M5Stack Cardputer.

pub mod dev_panel;
pub mod device_renderer;
pub mod display_view;
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
    factory: m5fxx_factory::FactoryFirmware,
    factory_mode: bool,
    binary_mode: bool,
    esp32: Option<m5fxx_esp32::Esp32Firmware>,
    pending_image: Option<Vec<u8>>,
    firmware_images: std::collections::HashMap<String, Vec<u8>>,
    factory_inputs: m5fxx_factory::SimulationInputs,
    panel: m5fxx_core::st7789::St7789,
    factory_frame: m5fxx_core::DisplayBuffer,
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
        hal.status.model = m5fxx_core::CardputerModel::CardputerAdv;

        let app = DemoApp::new();

        let mut panel = m5fxx_core::st7789::St7789::default();
        panel.command(0x11);
        panel.command(0x21);
        panel.command(0x29);
        let factory_inputs = m5fxx_factory::SimulationInputs {
            sd: u8::from(hal.storage.is_some()),
            ..Default::default()
        };
        Self {
            factory: m5fxx_factory::FactoryFirmware::default(),
            factory_mode: true,
            binary_mode: false,
            esp32: None,
            pending_image: None,
            firmware_images: Default::default(),
            factory_inputs,
            panel,
            factory_frame: Default::default(),
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

    fn factory_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("ADV factory firmware");
        ui.label("Original M5Stack V0.3 · simulated peripherals");
        if ui.button("Restart factory firmware").clicked() {
            self.factory.reset();
            self.hal.input.reset_all();
        }
        ui.collapsing("Display controller", |ui| {
            ui.add(egui::Slider::new(&mut self.panel.brightness, 0..=255).text("Backlight"));
            let mut on = self.panel.display_on;
            if ui.checkbox(&mut on, "Display enabled").changed() {
                self.panel.command(if on { 0x29 } else { 0x28 });
            }
            let mut sleeping = self.panel.sleeping;
            if ui.checkbox(&mut sleeping, "Sleep").changed() {
                self.panel.command(if sleeping { 0x10 } else { 0x11 });
            }
            let mut inverted = self.panel.inverted;
            if ui
                .checkbox(&mut inverted, "Hardware inversion (normal: on)")
                .changed()
            {
                self.panel.command(if inverted { 0x21 } else { 0x20 });
            }
        });
        ui.collapsing("Simulated hardware", |ui| {
            ui.add(egui::Slider::new(&mut self.factory_inputs.battery, 0..=100).text("Battery %"));
            for (value, label) in [
                (&mut self.factory_inputs.wifi, "Wi-Fi connected"),
                (&mut self.factory_inputs.ble, "BLE connected"),
                (&mut self.factory_inputs.usb, "USB connected"),
                (&mut self.factory_inputs.sd, "SD card inserted"),
                (&mut self.factory_inputs.cap, "LoRa/GPS cap attached"),
            ] {
                let mut enabled = *value != 0;
                if ui.checkbox(&mut enabled, label).changed() {
                    *value = u8::from(enabled);
                }
            }
            for (axis, value) in self.factory_inputs.accel.iter_mut().enumerate() {
                ui.add(
                    egui::Slider::new(value, -2.0..=2.0)
                        .text(format!("Acceleration {}", ["X", "Y", "Z"][axis])),
                );
            }
            for (axis, value) in self.factory_inputs.gyro.iter_mut().enumerate() {
                ui.add(
                    egui::Slider::new(value, -180.0..=180.0)
                        .text(format!("Gyroscope {}", ["X", "Y", "Z"][axis])),
                );
            }
            ui.add(
                egui::Slider::new(&mut self.factory_inputs.audio_amplitude, 0.0..=1.0)
                    .text("Microphone test signal"),
            );
        });
        ui.separator();
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
                                if (self.factory_mode || self.binary_mode)
                                    && self.hal.input.is_key_pressed(coord.row, coord.col)
                                {
                                    continue;
                                }
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
    }

    /// Processes Drag and Drop dropped files (e.g. .bin, .hex firmware images).
    /// This is called unconditionally, independent of keyboard focus, so dropping files
    /// from Finder/Explorer into the window works reliably on all platforms.
    fn handle_drag_and_drop(&mut self, ctx: &Context) {
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

            let bytes = if let Some(bytes) = &file.bytes {
                Ok(bytes.to_vec())
            } else if let Some(path) = &file.path {
                std::fs::read(path)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Drop has no file contents",
                ))
            };
            match bytes {
                Ok(bytes) if bytes.len() <= 16 * 1024 * 1024 => {
                    self.dev_state.firmware_import_requested = Some((file_name, bytes));
                }
                Ok(_) => self.hal.log("Firmware rejected: image larger than 16 MiB"),
                Err(error) => self
                    .hal
                    .log(format!("Cannot read dropped firmware: {error}")),
            }
        }
    }

    fn import_firmware(&mut self, name: String, bytes: Vec<u8>) {
        if let Err(error) = m5fxx_esp32::prepare_flash(&bytes) {
            self.hal.log(format!("Firmware rejected: {error}"));
            return;
        }
        self.esp32 = None;
        self.factory_mode = false;
        self.binary_mode = true;
        self.dev_state.is_paused = false;
        self.hal.input.reset_all();
        self.pending_image = Some(bytes.clone());
        if !self.dev_state.installed_firmwares.contains(&name) {
            self.dev_state.installed_firmwares.push(name.clone());
        }
        self.firmware_images.insert(name.clone(), bytes.clone());
        self.app.trigger_firmware_flash(name, bytes.len());
        self.hal
            .log("Merged image accepted; ESP32-S3 execution will start after import");
    }

    fn update_binary(&mut self, dt: f32) {
        self.hal.status.model = m5fxx_core::CardputerModel::CardputerOriginal;
        if let Some(cpu) = &mut self.esp32 {
            for (coord, pressed) in self.hal.input.take_matrix_events() {
                if let Err(error) = cpu.key(coord.row, coord.col, pressed) {
                    self.hal.log(error.to_string());
                }
            }
            let _ = cpu.home(self.hal.input.btn_g0_pressed);
            self.hal.input.take_chars();
            self.hal.input.take_keys();
            if let Err(error) = cpu.poll(&mut self.hal.display) {
                self.hal.log(error.to_string());
            }
        } else {
            self.hal.input.take_matrix_events();
            self.app.update(&mut self.hal, dt);
            if self.app.screen == m5fxx_app_demo::AppScreen::LoadedFirmware {
                if let Some(bytes) = self.pending_image.take() {
                    match m5fxx_esp32::Esp32Firmware::start(&bytes) {
                        Ok(cpu) => {
                            self.hal.display.clear(m5fxx_core::Color565::BLACK);
                            self.hal.display.draw_string(
                                5,
                                10,
                                "ESP32-S3 CPU booting...",
                                m5fxx_core::Color565::WHITE,
                                None,
                                1,
                            );
                            self.esp32 = Some(cpu);
                        }
                        Err(error) => {
                            self.app.fw_log_lines =
                                vec!["Emulator could not start".into(), error.to_string()];
                            self.app.render(&mut self.hal);
                            self.hal.log(format!("Emulator startup failed: {error}"));
                        }
                    }
                }
            }
        }
    }

    /// Uploads the current DisplayBuffer to egui GPU texture using Nearest-Neighbor filtering
    fn update_texture(&mut self, ctx: &Context) {
        if self.hal.display.is_dirty() || self.display_texture.is_none() {
            self.hal.display.to_rgba8888(&mut self.rgba_buffer);
            let brightness = if self.factory_mode {
                self.factory.brightness as u16 * self.panel.brightness as u16 / 255
            } else if self.binary_mode {
                self.esp32
                    .as_ref()
                    .map(|cpu| cpu.panel.brightness as u16)
                    .unwrap_or(255)
            } else {
                255
            };
            for (index, channel) in self.rgba_buffer.iter_mut().enumerate() {
                if index % 4 != 3 {
                    *channel = (*channel as u16 * brightness / 255) as u8;
                }
            }
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
        self.handle_drag_and_drop(ctx);
        self.handle_host_input(ctx);
        if let Some(name) = self.dev_state.boot_firmware_requested.take() {
            if let Some(bytes) = self.firmware_images.get(&name) {
                self.dev_state.firmware_import_requested = Some((name, bytes.clone()));
            }
        }
        if let Some((name, bytes)) = self.dev_state.firmware_import_requested.take() {
            self.import_firmware(name, bytes);
        }

        // Advance simulation tick if not paused
        let dt = self.hal.update();
        if let Some(cpu) = &mut self.esp32 {
            if let Err(error) = cpu.set_paused(!self.binary_mode || self.dev_state.is_paused) {
                self.hal.log(error.to_string());
            }
        }

        if !self.dev_state.is_paused {
            if self.binary_mode {
                self.update_binary(dt);
            } else if self.factory_mode {
                self.hal.status.model = m5fxx_core::CardputerModel::CardputerAdv;
                for (coord, pressed) in self.hal.input.take_matrix_events() {
                    self.factory.key(coord.row, coord.col, pressed);
                }
                self.hal.input.take_chars();
                self.hal.input.take_keys();
                self.factory.home(self.hal.input.btn_g0_pressed);
                if let Some(storage) = &self.hal.storage {
                    self.factory.sd_root(storage.root());
                    self.factory_inputs.sd_bytes = std::fs::read_dir(storage.root())
                        .map(|entries| {
                            entries
                                .flatten()
                                .filter_map(|e| e.metadata().ok())
                                .map(|m| m.len())
                                .sum()
                        })
                        .unwrap_or(0);
                }
                self.factory.inputs(&self.factory_inputs);
                self.factory.step(
                    (dt * 1000.0).round().clamp(1.0, 100.0) as u32,
                    &mut self.factory_frame,
                );
                self.panel.write_landscape(&self.factory_frame.pixels);
                self.panel.present(&mut self.hal.display);
            } else {
                self.hal.input.take_matrix_events();
                self.app.update(&mut self.hal, dt);
            }
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
                if ui
                    .selectable_label(self.factory_mode && !self.binary_mode, "ADV Factory")
                    .clicked()
                {
                    self.factory_mode = true;
                    self.binary_mode = false;
                }
                if ui
                    .selectable_label(!self.factory_mode && !self.binary_mode, "Rust Demo")
                    .clicked()
                {
                    self.factory_mode = false;
                    self.binary_mode = false;
                }
                if ui
                    .add_enabled(
                        self.esp32.is_some(),
                        egui::Button::new("ESP32 Binary").selected(self.binary_mode),
                    )
                    .clicked()
                {
                    self.factory_mode = false;
                    self.binary_mode = true;
                }
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
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.binary_mode {
                            ui.heading("ESP32-S3 binary execution");
                            if let Some(cpu) = &mut self.esp32 {
                                ui.label(&cpu.status);
                                if ui.button("Restart binary firmware").clicked() {
                                    if let Err(error) = cpu.reset() {
                                        self.hal.log(error.to_string());
                                    }
                                }
                                egui::CollapsingHeader::new("Actual UART / emulator log").show(
                                    ui,
                                    |ui| {
                                        for line in &cpu.logs {
                                            ui.monospace(line);
                                        }
                                    },
                                );
                            } else {
                                ui.label("Preparing merged flash image");
                            }
                        } else if self.factory_mode {
                            self.factory_controls(ui);
                        }
                        render_dev_panel(
                            ui,
                            &mut self.dev_state,
                            &mut self.hal,
                            &mut self.app,
                            &mut self.display_only_mode,
                            self.factory_mode,
                            self.binary_mode,
                        );
                        if std::mem::take(&mut self.dev_state.factory_reset_requested) {
                            if self.binary_mode {
                                if let Some(cpu) = &mut self.esp32 {
                                    let _ = cpu.reset();
                                }
                            } else {
                                self.factory.reset();
                            }
                        }
                    });
                });
        }

        // Central viewport: Cardputer Vector Device or Zoomed Display
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();

            if self.display_only_mode {
                let (screen_rect, int_scale) = display_view::pixel_rect(
                    available_rect.shrink2(vec2(0.0, 20.0)),
                    ctx.pixels_per_point(),
                );
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
                    DISPLAY_WIDTH as u32 * int_scale,
                    DISPLAY_HEIGHT as u32 * int_scale
                );
                painter.text(
                    egui::pos2(screen_rect.min.x, screen_rect.max.y + 8.0),
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

                let text = format!("Drop '{}' to boot ESP32-S3 firmware", first_file);
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
