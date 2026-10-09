//! Hardware Abstraction Layer (HAL) for the M5Stack Cardputer.

use crate::display::DisplayBuffer;
use crate::input::{CardputerInputState, CardputerKey};
use crate::storage::SdCardStorage;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardputerModel {
    #[default]
    CardputerOriginal,
    CardputerAdv,
}

impl CardputerModel {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CardputerOriginal => "M5Stack Cardputer (Standard)",
            Self::CardputerAdv => "M5Stack Cardputer ADV",
        }
    }

    pub fn keyboard_controller_desc(&self) -> &'static str {
        match self {
            Self::CardputerOriginal => "Software GPIO Matrix (74HC138 + Direct GPIOs)",
            Self::CardputerAdv => "TCA8418 I2C Keyboard Scanner (Addr: 0x34)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub model: CardputerModel,
    pub battery_mv: u16,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub uptime_secs: u64,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            model: CardputerModel::CardputerOriginal,
            battery_mv: 4150,
            battery_percent: 92,
            is_charging: true,
            uptime_secs: 0,
        }
    }
}

/// The Hardware Abstraction Layer trait or struct provided to the firmware application.
pub struct CardputerHal {
    pub display: DisplayBuffer,
    pub input: CardputerInputState,
    pub storage: Option<SdCardStorage>,
    pub status: SystemStatus,
    start_time: Instant,
    last_tick: Instant,
    pub logs: Vec<String>,
}

impl Default for CardputerHal {
    fn default() -> Self {
        Self::new(None)
    }
}

impl CardputerHal {
    pub fn new(storage: Option<SdCardStorage>) -> Self {
        let now = Instant::now();
        Self {
            display: DisplayBuffer::new(),
            input: CardputerInputState::new(),
            storage,
            status: SystemStatus::default(),
            start_time: now,
            last_tick: now,
            logs: Vec::new(),
        }
    }

    pub fn set_storage(&mut self, storage: SdCardStorage) {
        self.storage = Some(storage);
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        let s = msg.into();
        self.logs.push(s);
        if self.logs.len() > 200 {
            self.logs.drain(0..50);
        }
    }

    /// Monotonic elapsed time in milliseconds
    pub fn millis(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// Monotonic elapsed time in microseconds
    pub fn micros(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        let dt = (now - self.last_tick).as_secs_f32();
        self.last_tick = now;
        self.status.uptime_secs = self.start_time.elapsed().as_secs();
        dt
    }

    pub fn reset_uptime(&mut self) {
        self.start_time = Instant::now();
        self.last_tick = self.start_time;
        self.status.uptime_secs = 0;
    }

    pub fn take_keys(&mut self) -> Vec<CardputerKey> {
        self.input.take_keys()
    }

    pub fn take_chars(&mut self) -> Vec<char> {
        self.input.take_chars()
    }
}
