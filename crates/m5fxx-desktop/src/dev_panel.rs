//! Interactive developer panel showing simulation metrics, hardware state, and controls.

use egui::{CollapsingHeader, Color32, RichText, ScrollArea, Ui};
use m5fxx_app_demo::DemoApp;
use m5fxx_core::hal::CardputerModel;
use m5fxx_core::storage::SdCardStorage;
use m5fxx_core::CardputerHal;
use std::path::PathBuf;

pub struct DevPanelState {
    pub is_paused: bool,
    pub sd_folder_path: String,
    pub fps: f32,
    pub frame_counter: u32,
    pub last_fps_time: std::time::Instant,
}

impl Default for DevPanelState {
    fn default() -> Self {
        Self {
            is_paused: false,
            sd_folder_path: String::from("./virtual_sd"),
            fps: 60.0,
            frame_counter: 0,
            last_fps_time: std::time::Instant::now(),
        }
    }
}

impl DevPanelState {
    pub fn update_fps(&mut self) {
        self.frame_counter += 1;
        let elapsed = self.last_fps_time.elapsed().as_secs_f32();
        if elapsed >= 0.5 {
            self.fps = (self.frame_counter as f32) / elapsed;
            self.frame_counter = 0;
            self.last_fps_time = std::time::Instant::now();
        }
    }
}

pub fn render_dev_panel(
    ui: &mut Ui,
    dev_state: &mut DevPanelState,
    hal: &mut CardputerHal,
    app: &mut DemoApp,
    display_only_mode: &mut bool,
) {
    ui.heading("Cardputer Simulator Controls");
    ui.separator();

    // 1. Simulation controls (Play/Pause, Reset, Display Only)
    ui.horizontal(|ui| {
        let play_text = if dev_state.is_paused {
            "▶ Resume"
        } else {
            "⏸ Pause"
        };
        if ui.button(play_text).clicked() {
            dev_state.is_paused = !dev_state.is_paused;
            hal.log(if dev_state.is_paused {
                "Simulator Paused"
            } else {
                "Simulator Resumed"
            });
        }

        if ui.button("🔄 Reset App").clicked() {
            app.reset();
            hal.reset_uptime();
            hal.input.reset_all();
            hal.log("App and HAL reset");
        }

        ui.checkbox(display_only_mode, "Display-Only Zoom Mode");
    });

    ui.add_space(4.0);
    ui.label(format!(
        "Engine FPS: {:.1} | Uptime: {}s",
        dev_state.fps,
        hal.millis() / 1000
    ));

    ui.separator();

    // 2. Hardware Model Selection & Features
    CollapsingHeader::new("Hardware Profile & Features")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Cardputer Model:");
                if ui
                    .selectable_label(
                        hal.status.model == CardputerModel::CardputerOriginal,
                        "Original (GPIO)",
                    )
                    .clicked()
                {
                    hal.status.model = CardputerModel::CardputerOriginal;
                    hal.log("Switched to Cardputer Original");
                }
                if ui
                    .selectable_label(
                        hal.status.model == CardputerModel::CardputerAdv,
                        "ADV (TCA8418)",
                    )
                    .clicked()
                {
                    hal.status.model = CardputerModel::CardputerAdv;
                    hal.log("Switched to Cardputer ADV");
                }
            });

            ui.label(
                RichText::new(format!(
                    "Keyboard scanning: {}",
                    hal.status.model.keyboard_controller_desc()
                ))
                .small()
                .color(Color32::GRAY),
            );

            ui.add_space(4.0);
            ui.label("Hardware Implementation Status:");
            ui.label(
                RichText::new("✔ ST7789V2 240x135 IPS Display (Full Emulation)")
                    .color(Color32::GREEN),
            );
            ui.label(
                RichText::new("✔ 56-Key Keyboard + G0 Button (Full Emulation)")
                    .color(Color32::GREEN),
            );
            ui.label(
                RichText::new("✔ Sandboxed MicroSD Filesystem (Active)").color(Color32::GREEN),
            );
            ui.label(
                RichText::new("✔ Battery & Power Circuitry (Simulated)").color(Color32::GREEN),
            );
            ui.label(
                RichText::new("⚠ NS4168 1W Speaker (Audio stub ready)").color(Color32::YELLOW),
            );
            ui.label(RichText::new("⚠ SPM1423 PDM Microphone (Stub ready)").color(Color32::YELLOW));
            ui.label(
                RichText::new("✖ Wi-Fi / ESP-NOW / Bluetooth (Not implemented)")
                    .color(Color32::LIGHT_RED),
            );
            ui.label(
                RichText::new("✖ Grove / External GPIO Header (Not implemented)")
                    .color(Color32::LIGHT_RED),
            );
        });

    ui.separator();

    // 3. Virtual SD Card Folder Config
    CollapsingHeader::new("MicroSD Card Sandbox")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Local Path:");
                ui.text_edit_singleline(&mut dev_state.sd_folder_path);
            });

            ui.horizontal(|ui| {
                if ui.button("Browse Folder...").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        dev_state.sd_folder_path = folder.to_string_lossy().to_string();
                        match SdCardStorage::new(&dev_state.sd_folder_path) {
                            Ok(sd) => {
                                hal.set_storage(sd);
                                hal.log(format!("Mounted SD at {}", dev_state.sd_folder_path));
                            }
                            Err(e) => {
                                hal.log(format!("SD mount failed: {}", e));
                            }
                        }
                    }
                }

                if ui.button("Mount / Apply").clicked() {
                    let path = PathBuf::from(&dev_state.sd_folder_path);
                    match SdCardStorage::new(path) {
                        Ok(sd) => {
                            hal.set_storage(sd);
                            hal.log(format!("Mounted SD at {}", dev_state.sd_folder_path));
                        }
                        Err(e) => {
                            hal.log(format!("SD mount failed: {}", e));
                        }
                    }
                }
            });

            if let Some(sd) = &hal.storage {
                ui.label(
                    RichText::new(format!("Mounted at: {:?}", sd.root()))
                        .small()
                        .color(Color32::GREEN),
                );
                if let Ok(files) = sd.list_dir(".") {
                    ui.label(format!("Files on root: {:?}", files));
                }
            } else {
                ui.label(RichText::new("SD card not mounted").color(Color32::RED));
            }
        });

    ui.separator();

    // 4. Real-time Pressed Keys & Modifiers
    CollapsingHeader::new("Keyboard Live Status")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Active Modifiers:");
                if hal.input.fn_active {
                    ui.colored_label(Color32::YELLOW, "[Fn]");
                }
                if hal.input.shift_active {
                    ui.colored_label(Color32::YELLOW, "[Shift]");
                }
                if hal.input.ctrl_active {
                    ui.colored_label(Color32::YELLOW, "[Ctrl]");
                }
                if hal.input.opt_active {
                    ui.colored_label(Color32::YELLOW, "[Opt]");
                }
                if hal.input.alt_active {
                    ui.colored_label(Color32::YELLOW, "[Alt]");
                }
                if !hal.input.fn_active
                    && !hal.input.shift_active
                    && !hal.input.ctrl_active
                    && !hal.input.opt_active
                    && !hal.input.alt_active
                {
                    ui.label("None");
                }
            });

            let active_keys: Vec<String> = hal
                .input
                .pressed_matrix_keys
                .iter()
                .map(|k| format!("(R{}, C{})", k.row, k.col))
                .collect();
            ui.label(format!(
                "Pressed Matrix Keys: {}",
                if active_keys.is_empty() {
                    "None".to_string()
                } else {
                    active_keys.join(", ")
                }
            ));
            ui.label(format!("BtnG0: {}", hal.input.btn_g0_pressed));
        });

    ui.separator();

    // 5. System Logs
    CollapsingHeader::new("Simulator Logs")
        .default_open(true)
        .show(ui, |ui| {
            ScrollArea::vertical()
                .max_height(140.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for log in &hal.logs {
                        ui.label(RichText::new(log).small().monospace());
                    }
                });
        });
}
