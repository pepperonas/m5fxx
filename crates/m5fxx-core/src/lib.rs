//! Core types and abstractions for M5Stack Cardputer.

pub mod color;
pub mod display;
pub mod font;
pub mod hal;
pub mod input;
pub mod storage;

pub use color::Color565;
pub use display::{DisplayBuffer, DISPLAY_HEIGHT, DISPLAY_WIDTH};
pub use hal::{CardputerHal, CardputerModel, SystemStatus};
pub use input::{CardputerInputState, CardputerKey, KeyCoord, KeyModifier};
pub use storage::SdCardStorage;
