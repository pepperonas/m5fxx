//! Hardware-agnostic demo firmware application for the M5Stack Cardputer.
//!
//! Demonstrates:
//! 1. SplashScreen with system information
//! 2. Menu navigation (using Cardputer keys or arrows)
//! 3. Text Editor (full keyboard input, backspace, enter, live cursor)
//! 4. Pixel Graphics & Animation demo (retro bouncing balls & starfield)
//! 5. Virtual SD Card File Viewer & Writer (safe sandbox IO)
//! 6. System Info & Hardware Peripherals Status

use m5fxx_core::{CardputerHal, CardputerKey, Color565, DISPLAY_HEIGHT, DISPLAY_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Splash,
    Menu,
    TextEditor,
    GraphicsDemo,
    SdStorage,
    SysInfo,
    FirmwareFlashing,
    LoadedFirmware,
}

pub struct DemoApp {
    pub screen: AppScreen,
    pub menu_selected: usize,
    pub editor_text: String,
    pub cursor_pos: usize,
    pub anim_tick: f32,
    pub sd_status_msg: String,
    pub sd_file_content: String,
    pub balls: Vec<BouncingBall>,
    pub stars: Vec<Star>,
    // Flashing simulation state
    pub flash_file_name: String,
    pub flash_file_size: usize,
    pub flash_progress: f32,
    pub flash_status: String,
    pub flash_timer: f32,
    // Loaded firmware runtime state
    pub fw_uptime: f32,
    pub fw_log_lines: Vec<String>,
    pub fw_last_key: Option<String>,
}

pub struct BouncingBall {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: i32,
    pub color: Color565,
}

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub color: Color565,
}

const MENU_ITEMS: &[(&str, AppScreen)] = &[
    ("1. Text Editor", AppScreen::TextEditor),
    ("2. Graphics Animation", AppScreen::GraphicsDemo),
    ("3. SD Card Storage", AppScreen::SdStorage),
    ("4. System & HW Info", AppScreen::SysInfo),
];

impl Default for DemoApp {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoApp {
    pub fn new() -> Self {
        let balls = vec![
            BouncingBall {
                x: 30.0,
                y: 30.0,
                vx: 55.0,
                vy: 40.0,
                radius: 4,
                color: Color565::RED,
            },
            BouncingBall {
                x: 80.0,
                y: 50.0,
                vx: -45.0,
                vy: 60.0,
                radius: 5,
                color: Color565::CYAN,
            },
            BouncingBall {
                x: 140.0,
                y: 70.0,
                vx: 50.0,
                vy: -50.0,
                radius: 3,
                color: Color565::YELLOW,
            },
            BouncingBall {
                x: 200.0,
                y: 40.0,
                vx: -60.0,
                vy: -35.0,
                radius: 4,
                color: Color565::GREENYELLOW,
            },
        ];

        let mut stars = Vec::new();
        for i in 0..40 {
            let x = ((i * 37) % DISPLAY_WIDTH as i32) as f32;
            let y = (((i * 47) % (DISPLAY_HEIGHT as i32 - 20)) + 12) as f32;
            let speed = (i % 3 + 1) as f32 * 25.0;
            stars.push(Star {
                x,
                y,
                speed,
                color: if i % 2 == 0 {
                    Color565::WHITE
                } else {
                    Color565::DARKCYAN
                },
            });
        }

        Self {
            screen: AppScreen::Splash,
            menu_selected: 0,
            editor_text: String::from(
                "Cardputer OS v1.0\nType any text here...\n[Del]=Erase [Esc]=Menu",
            ),
            cursor_pos: 0,
            anim_tick: 0.0,
            sd_status_msg: String::from("Press [W]rite or [R]ead file"),
            sd_file_content: String::new(),
            balls,
            stars,
            flash_file_name: String::new(),
            flash_file_size: 0,
            flash_progress: 0.0,
            flash_status: String::new(),
            flash_timer: 0.0,
            fw_uptime: 0.0,
            fw_log_lines: Vec::new(),
            fw_last_key: None,
        }
    }

    /// Triggers firmware installation from a dropped file
    pub fn trigger_firmware_flash(&mut self, file_name: String, file_size: usize) {
        self.screen = AppScreen::FirmwareFlashing;
        self.flash_file_name = file_name;
        self.flash_file_size = file_size;
        self.flash_progress = 0.0;
        self.flash_timer = 0.0;
        self.flash_status = "Preparing merged image...".to_string();
        self.fw_uptime = 0.0;
        self.fw_log_lines.clear();
        self.fw_last_key = None;
    }

    /// Reset app state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Update logic called every frame (e.g. 60Hz)
    pub fn update(&mut self, hal: &mut CardputerHal, dt: f32) {
        self.anim_tick += dt;

        // Process inputs
        let keys = hal.take_keys();
        let chars = hal.take_chars();

        match self.screen {
            AppScreen::Splash => {
                if !keys.is_empty() || !chars.is_empty() || self.anim_tick > 2.5 {
                    self.screen = AppScreen::Menu;
                    hal.log("Transitioned to Menu");
                }
            }
            AppScreen::Menu => {
                for key in &keys {
                    match key {
                        CardputerKey::Up => {
                            if self.menu_selected > 0 {
                                self.menu_selected -= 1;
                            } else {
                                self.menu_selected = MENU_ITEMS.len() - 1;
                            }
                        }
                        CardputerKey::Down => {
                            if self.menu_selected + 1 < MENU_ITEMS.len() {
                                self.menu_selected += 1;
                            } else {
                                self.menu_selected = 0;
                            }
                        }
                        CardputerKey::Enter => {
                            self.screen = MENU_ITEMS[self.menu_selected].1;
                            hal.log(format!("Selected: {}", MENU_ITEMS[self.menu_selected].0));
                        }
                        CardputerKey::Char('1') => self.screen = AppScreen::TextEditor,
                        CardputerKey::Char('2') => self.screen = AppScreen::GraphicsDemo,
                        CardputerKey::Char('3') => self.screen = AppScreen::SdStorage,
                        CardputerKey::Char('4') => self.screen = AppScreen::SysInfo,
                        _ => {}
                    }
                }
            }
            AppScreen::TextEditor => {
                for key in &keys {
                    match key {
                        CardputerKey::Esc => {
                            self.screen = AppScreen::Menu;
                            hal.log("Exited Editor to Menu");
                            return;
                        }
                        CardputerKey::Backspace | CardputerKey::Delete => {
                            self.editor_text.pop();
                        }
                        CardputerKey::Enter => {
                            self.editor_text.push('\n');
                        }
                        CardputerKey::Tab => {
                            self.editor_text.push_str("  ");
                        }
                        _ => {}
                    }
                }
                for ch in chars {
                    if ch >= ' ' && ch != '\x7F' {
                        self.editor_text.push(ch);
                    }
                }
            }
            AppScreen::GraphicsDemo => {
                for key in &keys {
                    if *key == CardputerKey::Esc || *key == CardputerKey::Backspace {
                        self.screen = AppScreen::Menu;
                        hal.log("Exited Graphics Demo to Menu");
                        return;
                    }
                }

                // Update animation physics
                for b in &mut self.balls {
                    b.x += b.vx * dt;
                    b.y += b.vy * dt;

                    let min_x = b.radius as f32;
                    let max_x = (DISPLAY_WIDTH as i32 - b.radius) as f32;
                    let min_y = 12.0 + b.radius as f32;
                    let max_y = (DISPLAY_HEIGHT as i32 - b.radius) as f32;

                    if b.x <= min_x {
                        b.x = min_x;
                        b.vx = b.vx.abs();
                    } else if b.x >= max_x {
                        b.x = max_x;
                        b.vx = -b.vx.abs();
                    }

                    if b.y <= min_y {
                        b.y = min_y;
                        b.vy = b.vy.abs();
                    } else if b.y >= max_y {
                        b.y = max_y;
                        b.vy = -b.vy.abs();
                    }
                }

                for s in &mut self.stars {
                    s.x -= s.speed * dt;
                    if s.x < 0.0 {
                        s.x = DISPLAY_WIDTH as f32;
                    }
                }
            }
            AppScreen::SdStorage => {
                for key in &keys {
                    match key {
                        CardputerKey::Esc | CardputerKey::Backspace => {
                            self.screen = AppScreen::Menu;
                            hal.log("Exited SD Storage to Menu");
                            return;
                        }
                        CardputerKey::Char('w') | CardputerKey::Char('W') => {
                            if let Some(storage) = &hal.storage {
                                let time_sec = hal.millis() / 1000;
                                let sample = format!(
                                    "m5fxx Cardputer File\nUptime: {}s\nStatus: OK!\n",
                                    time_sec
                                );
                                match storage.write_file("sample.txt", sample.as_bytes()) {
                                    Ok(_) => {
                                        self.sd_status_msg =
                                            "Wrote 'sample.txt' successfully!".into();
                                        hal.log("SD: Wrote sample.txt");
                                    }
                                    Err(e) => {
                                        self.sd_status_msg = format!("Write error: {}", e);
                                        hal.log(format!("SD Write error: {}", e));
                                    }
                                }
                            } else {
                                self.sd_status_msg = "No SD card folder mounted!".into();
                            }
                        }
                        CardputerKey::Char('r') | CardputerKey::Char('R') => {
                            if let Some(storage) = &hal.storage {
                                match storage.read_to_string("sample.txt") {
                                    Ok(content) => {
                                        self.sd_file_content = content;
                                        self.sd_status_msg = "Read 'sample.txt' OK".into();
                                        hal.log("SD: Read sample.txt");
                                    }
                                    Err(e) => {
                                        self.sd_status_msg = format!("Read error: {}", e);
                                        hal.log(format!("SD Read error: {}", e));
                                    }
                                }
                            } else {
                                self.sd_status_msg = "No SD card folder mounted!".into();
                            }
                        }
                        _ => {}
                    }
                }
            }
            AppScreen::SysInfo => {
                for key in &keys {
                    if *key == CardputerKey::Esc || *key == CardputerKey::Backspace {
                        self.screen = AppScreen::Menu;
                        hal.log("Exited SysInfo to Menu");
                        return;
                    }
                }
            }
            AppScreen::FirmwareFlashing => {
                self.flash_timer += dt;
                if self.flash_timer < 0.8 {
                    self.flash_progress = (self.flash_timer / 0.8) * 0.15;
                    self.flash_status = "Preparing ESP32-S3 image...".to_string();
                } else if self.flash_timer < 2.5 {
                    let progress = 0.15 + ((self.flash_timer - 0.8) / 1.7) * 0.70;
                    self.flash_progress = progress;
                    self.flash_status = format!("Import: {:.0}%", progress * 100.0);
                } else if self.flash_timer < 3.2 {
                    self.flash_progress = 0.95;
                    self.flash_status = "Preparing emulator...".to_string();
                } else if self.flash_timer < 4.2 {
                    self.flash_progress = 1.0;
                    self.flash_status = "Starting ESP32-S3 emulator...".to_string();
                } else {
                    // Reboot directly into the flashed firmware runtime screen!
                    self.screen = AppScreen::LoadedFirmware;
                    self.fw_uptime = 0.0;
                    self.fw_log_lines.clear();
                    self.fw_log_lines
                        .push("ESP32 binary not executed.".to_string());
                    self.fw_log_lines
                        .push("CPU emulation unavailable.".to_string());
                    self.fw_log_lines
                        .push("Use ADV Factory for native UI.".to_string());
                    hal.log(format!(
                        "Cannot execute ESP32 image '{}': CPU emulation unavailable",
                        self.flash_file_name
                    ));
                    return;
                }

                // Allow cancel via Esc
                for key in &keys {
                    if *key == CardputerKey::Esc {
                        self.screen = AppScreen::Menu;
                        hal.log("Flashing cancelled by user");
                        return;
                    }
                }
            }
            AppScreen::LoadedFirmware => {
                self.fw_uptime += dt;

                // Handle keys in running firmware
                for key in &keys {
                    if *key == CardputerKey::Esc {
                        self.screen = AppScreen::Menu;
                        hal.log("Exited loaded firmware to Menu");
                        return;
                    }
                    self.fw_last_key = Some(format!("{:?}", key));
                    let log_entry = format!("[KEY] Pressed: {:?}", key);
                    self.fw_log_lines.push(log_entry);
                    if self.fw_log_lines.len() > 6 {
                        self.fw_log_lines.remove(0);
                    }
                }

                for ch in &chars {
                    self.fw_last_key = Some(format!("'{}'", ch));
                    let log_entry = format!("[INP] Char: '{}'", ch);
                    self.fw_log_lines.push(log_entry);
                    if self.fw_log_lines.len() > 6 {
                        self.fw_log_lines.remove(0);
                    }
                }
            }
        }

        // Draw current screen to HAL display buffer
        self.render(hal);
    }

    /// Render UI to the display buffer
    pub fn render(&mut self, hal: &mut CardputerHal) {
        match self.screen {
            AppScreen::Splash => self.render_splash(hal),
            AppScreen::Menu => self.render_menu(hal),
            AppScreen::TextEditor => self.render_editor(hal),
            AppScreen::GraphicsDemo => self.render_graphics(hal),
            AppScreen::SdStorage => self.render_sd(hal),
            AppScreen::SysInfo => self.render_sysinfo(hal),
            AppScreen::FirmwareFlashing => self.render_flashing(hal),
            AppScreen::LoadedFirmware => self.render_loaded_firmware(hal),
        }
    }

    fn draw_header(hal: &mut CardputerHal, title: &str) {
        hal.display
            .fill_rect(0, 0, DISPLAY_WIDTH as i32, 11, Color565::NAVY);
        hal.display
            .draw_string(2, 2, title, Color565::WHITE, None, 1);

        let uptime = format!("{}s", hal.millis() / 1000);
        let x = DISPLAY_WIDTH as i32 - (uptime.len() as i32 * 6) - 4;
        hal.display
            .draw_string(x, 2, &uptime, Color565::YELLOW, None, 1);
        hal.display
            .draw_line(0, 11, DISPLAY_WIDTH as i32, 11, Color565::DARKGREY);
    }

    fn render_splash(&self, hal: &mut CardputerHal) {
        hal.display.clear(Color565::BLACK);

        // Logo frame
        hal.display.draw_rect(
            10,
            10,
            DISPLAY_WIDTH as i32 - 20,
            DISPLAY_HEIGHT as i32 - 20,
            Color565::BLUE,
        );
        hal.display.draw_rect(
            12,
            12,
            DISPLAY_WIDTH as i32 - 24,
            DISPLAY_HEIGHT as i32 - 24,
            Color565::NAVY,
        );

        hal.display
            .draw_string(50, 25, "M5STACK CARDPUTER", Color565::YELLOW, None, 1);
        hal.display
            .draw_string(75, 42, "m5fxx SIMULATOR", Color565::WHITE, None, 1);
        hal.display
            .draw_string(60, 60, "Firmware Core v0.1", Color565::LIGHTGREY, None, 1);

        // Loading bar
        let progress = ((self.anim_tick / 2.0).min(1.0) * 160.0) as i32;
        hal.display.draw_rect(40, 85, 160, 10, Color565::WHITE);
        hal.display
            .fill_rect(42, 87, progress.max(0), 6, Color565::GREEN);

        let pulse = (self.anim_tick * 4.0) as i32 % 2 == 0;
        let prompt_color = if pulse {
            Color565::GREENYELLOW
        } else {
            Color565::DARKGREEN
        };
        hal.display
            .draw_string(45, 105, "Press any key to start...", prompt_color, None, 1);
    }

    fn render_menu(&self, hal: &mut CardputerHal) {
        hal.display.clear(Color565::BLACK);
        Self::draw_header(hal, "M5Cardputer Main Menu");

        hal.display.draw_string(
            8,
            16,
            "Select Application (Up/Dn/Enter):",
            Color565::LIGHTGREY,
            None,
            1,
        );

        for (i, (name, _)) in MENU_ITEMS.iter().enumerate() {
            let y = 32 + (i as i32 * 18);
            let is_selected = i == self.menu_selected;

            if is_selected {
                hal.display.fill_rect(8, y - 2, 224, 15, Color565::NAVY);
                hal.display.draw_rect(8, y - 2, 224, 15, Color565::CYAN);
                hal.display
                    .draw_string(14, y + 2, ">", Color565::YELLOW, None, 1);
                hal.display
                    .draw_string(24, y + 2, name, Color565::WHITE, None, 1);
            } else {
                hal.display.draw_rect(8, y - 2, 224, 15, Color565::DARKGREY);
                hal.display
                    .draw_string(24, y + 2, name, Color565::LIGHTGREY, None, 1);
            }
        }

        // Footer helper
        hal.display
            .draw_line(0, 122, DISPLAY_WIDTH as i32, 122, Color565::DARKGREY);
        hal.display.draw_string(
            4,
            125,
            "[Fn+;]=Up [Fn+.]=Dn [Enter]=Select",
            Color565::DARKGREEN,
            None,
            1,
        );
    }

    fn render_editor(&self, hal: &mut CardputerHal) {
        hal.display.clear(Color565::BLACK);
        Self::draw_header(hal, "Text Editor [Esc]=Back");

        let mut line_y = 15;
        let mut cur_x = 4;
        let line_height = 9;

        for ch in self.editor_text.chars() {
            if ch == '\n' {
                line_y += line_height;
                cur_x = 4;
                if line_y > DISPLAY_HEIGHT as i32 - 12 {
                    break;
                }
                continue;
            }
            if cur_x > DISPLAY_WIDTH as i32 - 10 {
                line_y += line_height;
                cur_x = 4;
            }
            if line_y > DISPLAY_HEIGHT as i32 - 12 {
                break;
            }
            cur_x += hal
                .display
                .draw_char(cur_x, line_y, ch, Color565::WHITE, None, 1);
        }

        // Blinking cursor
        let cursor_on = ((self.anim_tick * 3.0) as i32 % 2) == 0;
        if cursor_on && line_y <= DISPLAY_HEIGHT as i32 - 12 {
            hal.display.fill_rect(cur_x, line_y, 5, 7, Color565::GREEN);
        }

        // Bottom status line
        hal.display
            .fill_rect(0, 124, DISPLAY_WIDTH as i32, 11, Color565::NAVY);
        let chars_count = format!("Chars: {}", self.editor_text.len());
        hal.display
            .draw_string(4, 126, &chars_count, Color565::WHITE, None, 1);
        hal.display
            .draw_string(150, 126, "[Esc] Exit", Color565::YELLOW, None, 1);
    }

    fn render_graphics(&self, hal: &mut CardputerHal) {
        hal.display.clear(Color565::BLACK);
        Self::draw_header(hal, "Graphics Demo [Esc]=Back");

        // Starfield
        for star in &self.stars {
            hal.display
                .set_pixel(star.x as i32, star.y as i32, star.color);
        }

        // Bouncing balls
        for b in &self.balls {
            let bx = b.x as i32;
            let by = b.y as i32;
            let r = b.radius;
            // Draw filled circle / diamond
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r {
                        hal.display.set_pixel(bx + dx, by + dy, b.color);
                    }
                }
            }
        }

        // Coordinate overlay
        let info = format!("Balls: {} Stars: {}", self.balls.len(), self.stars.len());
        hal.display.draw_string(
            4,
            DISPLAY_HEIGHT as i32 - 10,
            &info,
            Color565::LIGHTGREY,
            None,
            1,
        );
    }

    fn render_sd(&self, hal: &mut CardputerHal) {
        hal.display.clear(Color565::BLACK);
        Self::draw_header(hal, "SD Storage [Esc]=Back");

        hal.display
            .draw_string(4, 16, "Commands:", Color565::YELLOW, None, 1);
        hal.display
            .draw_string(10, 27, "[W] - Write sample.txt", Color565::WHITE, None, 1);
        hal.display
            .draw_string(10, 38, "[R] - Read sample.txt", Color565::WHITE, None, 1);

        hal.display
            .draw_line(0, 50, DISPLAY_WIDTH as i32, 50, Color565::DARKGREY);

        hal.display
            .draw_string(4, 54, "Status:", Color565::LIGHTGREY, None, 1);
        hal.display
            .draw_string(4, 65, &self.sd_status_msg, Color565::GREENYELLOW, None, 1);

        hal.display
            .draw_string(4, 80, "File Content:", Color565::LIGHTGREY, None, 1);
        let preview = if self.sd_file_content.is_empty() {
            "<empty>"
        } else {
            &self.sd_file_content
        };
        hal.display
            .draw_string(4, 92, preview, Color565::CYAN, None, 1);
    }

    fn render_sysinfo(&self, hal: &mut CardputerHal) {
        hal.display.clear(Color565::BLACK);
        Self::draw_header(hal, "Cardputer Info [Esc]=Back");

        let m = hal.status.model.name();
        hal.display
            .draw_string(4, 16, "Model:", Color565::YELLOW, None, 1);
        hal.display.draw_string(45, 16, m, Color565::WHITE, None, 1);

        hal.display
            .draw_string(4, 28, "MCU:", Color565::YELLOW, None, 1);
        hal.display
            .draw_string(45, 28, "ESP32-S3FN8 240MHz", Color565::WHITE, None, 1);

        hal.display
            .draw_string(4, 40, "Screen:", Color565::YELLOW, None, 1);
        hal.display
            .draw_string(45, 40, "ST7789V2 240x135 IPS", Color565::WHITE, None, 1);

        hal.display
            .draw_string(4, 52, "Keyboard:", Color565::YELLOW, None, 1);
        hal.display.draw_string(
            4,
            62,
            hal.status.model.keyboard_controller_desc(),
            Color565::LIGHTGREY,
            None,
            1,
        );

        let bat = format!(
            "{}% ({} mV)",
            hal.status.battery_percent, hal.status.battery_mv
        );
        hal.display
            .draw_string(4, 76, "Battery:", Color565::YELLOW, None, 1);
        hal.display
            .draw_string(55, 76, &bat, Color565::GREEN, None, 1);

        let charging = if hal.status.is_charging {
            "Charging (USB-C)"
        } else {
            "Discharging"
        };
        hal.display
            .draw_string(4, 88, "Power:", Color565::YELLOW, None, 1);
        hal.display
            .draw_string(55, 88, charging, Color565::GREENYELLOW, None, 1);

        hal.display.draw_string(
            4,
            102,
            "Speaker: NS4168 1W (Ready)",
            Color565::DARKCYAN,
            None,
            1,
        );
        hal.display.draw_string(
            4,
            114,
            "Mic: SPM1423 (Simulated)",
            Color565::DARKCYAN,
            None,
            1,
        );
    }

    fn render_flashing(&self, hal: &mut CardputerHal) {
        hal.display.clear(Color565::BLACK);
        Self::draw_header(hal, "Firmware Flasher");

        // Flashing icon / header
        hal.display
            .draw_string(8, 18, "IMPORTING FIRMWARE...", Color565::YELLOW, None, 1);

        // Binary filename & size
        let info = format!(
            "File: {}",
            self.flash_file_name.chars().take(24).collect::<String>()
        );
        hal.display
            .draw_string(8, 32, &info, Color565::WHITE, None, 1);

        let size_kb = (self.flash_file_size as f32) / 1024.0;
        let size_str = format!("Size: {:.1} KB", size_kb);
        hal.display
            .draw_string(8, 44, &size_str, Color565::LIGHTGREY, None, 1);

        // Status text
        hal.display
            .draw_string(8, 60, &self.flash_status, Color565::CYAN, None, 1);

        // Progress bar background & fill
        let bar_x = 8;
        let bar_y = 76;
        let bar_w = (DISPLAY_WIDTH as i32) - 16;
        let bar_h = 14;

        hal.display
            .draw_rect(bar_x, bar_y, bar_w, bar_h, Color565::WHITE);
        hal.display
            .fill_rect(bar_x + 1, bar_y + 1, bar_w - 2, bar_h - 2, Color565::NAVY);

        let fill_w = (((bar_w - 2) as f32) * self.flash_progress.clamp(0.0, 1.0)) as i32;
        if fill_w > 0 {
            let bar_color = if self.flash_progress >= 1.0 {
                Color565::GREEN
            } else {
                Color565::from_rgb888(0xFA, 0x6A, 0x00) // Cardputer signature orange
            };
            hal.display
                .fill_rect(bar_x + 1, bar_y + 1, fill_w, bar_h - 2, bar_color);
        }

        // Percentage text centered
        let pct_text = format!("{:.0}%", (self.flash_progress * 100.0).clamp(0.0, 100.0));
        let pct_x = bar_x + (bar_w / 2) - ((pct_text.len() as i32 * 6) / 2);
        hal.display
            .draw_string(pct_x, bar_y + 3, &pct_text, Color565::WHITE, None, 1);

        // Target chip & memory info
        hal.display.draw_string(
            8,
            96,
            "Target: ESP32-S3 in QEMU",
            Color565::DARKGREY,
            None,
            1,
        );

        // Footer instructions
        hal.display
            .draw_line(0, 122, DISPLAY_WIDTH as i32, 122, Color565::DARKGREY);
        hal.display.draw_string(
            4,
            125,
            "Do not disconnect power! [Esc]=Cancel",
            Color565::RED,
            None,
            1,
        );
    }

    fn render_loaded_firmware(&self, hal: &mut CardputerHal) {
        hal.display.clear(Color565::BLACK);

        // Header with active status and firmware title
        let header_title = format!("▶ FW: {}", self.flash_file_name);
        let header_cropped = if header_title.len() > 24 {
            format!("{}...", header_title.chars().take(21).collect::<String>())
        } else {
            header_title
        };
        Self::draw_header(hal, &header_cropped);

        // Top info bar: status pill & active runtime metrics
        hal.display
            .fill_rect(4, 15, DISPLAY_WIDTH as i32 - 8, 16, Color565::NAVY);
        hal.display
            .draw_rect(4, 15, DISPLAY_WIDTH as i32 - 8, 16, Color565::DARKCYAN);

        let m5_orange = Color565::from_rgb888(0xFA, 0x6A, 0x00);
        hal.display
            .draw_string(8, 19, "NOT EXECUTED", Color565::GREENYELLOW, None, 1);

        let size_str = if self.flash_file_size > 0 {
            format!("{:.1} KB", self.flash_file_size as f32 / 1024.0)
        } else {
            "ROM Image".to_string()
        };
        hal.display
            .draw_string(60, 19, &size_str, Color565::LIGHTGREY, None, 1);

        let fw_run_time = format!("View: {:.1}s", self.fw_uptime);
        hal.display.draw_string(
            DISPLAY_WIDTH as i32 - (fw_run_time.len() as i32 * 6) - 10,
            19,
            &fw_run_time,
            m5_orange,
            None,
            1,
        );

        // Virtual UART / Console output terminal box
        hal.display.draw_rect(
            4,
            34,
            DISPLAY_WIDTH as i32 - 8,
            DISPLAY_HEIGHT as i32 - 48,
            Color565::DARKGREY,
        );
        hal.display.fill_rect(
            5,
            35,
            DISPLAY_WIDTH as i32 - 10,
            DISPLAY_HEIGHT as i32 - 50,
            Color565::from_rgb888(12, 16, 20),
        );

        // Title of console
        hal.display
            .draw_string(8, 37, "Simulator status:", Color565::CYAN, None, 1);

        // Render log lines
        let mut y = 49;
        for line in &self.fw_log_lines {
            let color = if line.starts_with("[SYS]") {
                Color565::DARKGREY
            } else if line.starts_with("[BOOT]") {
                Color565::YELLOW
            } else if line.starts_with("[APP]") {
                Color565::GREEN
            } else if line.starts_with("[KEY]") || line.starts_with("[INP]") {
                m5_orange
            } else {
                Color565::WHITE
            };
            hal.display.draw_string(8, y, line, color, None, 1);
            y += 11;
        }

        // Live blinking cursor at bottom of console
        let blink = (self.anim_tick * 3.0) as i32 % 2 == 0;
        if blink {
            hal.display.draw_string(8, y, "_", Color565::GREEN, None, 1);
        }

        // Footer instructions
        hal.display
            .draw_line(0, 123, DISPLAY_WIDTH as i32, 123, Color565::DARKGREY);
        let key_hint = if let Some(last) = &self.fw_last_key {
            format!("Last Key: {}  [Esc]=Exit", last)
        } else {
            "Type keys to interact  [Esc]=Exit".to_string()
        };
        hal.display
            .draw_string(4, 125, &key_hint, Color565::LIGHTGREY, None, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_app_navigation() {
        let mut app = DemoApp::new();
        let mut hal = CardputerHal::default();

        assert_eq!(app.screen, AppScreen::Splash);
        // Press enter to dismiss splash
        hal.input.press_key(2, 13);
        app.update(&mut hal, 0.1);
        hal.input.release_key(2, 13);
        assert_eq!(app.screen, AppScreen::Menu);

        // Select Text Editor
        hal.input.press_key(2, 13);
        app.update(&mut hal, 0.1);
        hal.input.release_key(2, 13);
        assert_eq!(app.screen, AppScreen::TextEditor);

        // Type 'H', 'i'
        hal.input.press_key(2, 7); // 'h'
        app.update(&mut hal, 0.1);
        hal.input.release_key(2, 7);
        assert!(app.editor_text.ends_with('h'));

        // Escape back to menu
        hal.input.press_key(2, 0); // Fn
        hal.input.press_key(0, 0); // Esc in Fn layer
        app.update(&mut hal, 0.1);
        hal.input.release_key(0, 0);
        hal.input.release_key(2, 0);
        assert_eq!(app.screen, AppScreen::Menu);
    }

    #[test]
    fn test_firmware_flashing_and_boot() {
        let mut app = DemoApp::new();
        let mut hal = CardputerHal::default();

        app.trigger_firmware_flash("custom_fw.bin".to_string(), 102400);
        assert_eq!(app.screen, AppScreen::FirmwareFlashing);

        // Advance flashing timer until completion (> 4.2s)
        app.update(&mut hal, 5.0);
        assert_eq!(app.screen, AppScreen::LoadedFirmware);
        assert_eq!(app.flash_file_name, "custom_fw.bin");
        assert!(app
            .fw_log_lines
            .iter()
            .any(|line| line.contains("not executed")));

        // Press a key in the loaded firmware
        hal.input.press_cardputer_key(CardputerKey::Enter);
        app.update(&mut hal, 0.1);
        assert!(app.fw_last_key.is_some());

        // Press Esc to exit loaded firmware
        hal.input.press_cardputer_key(CardputerKey::Esc);
        app.update(&mut hal, 0.1);
        assert_eq!(app.screen, AppScreen::Menu);
    }
}
