//! Native port of the pinned M5Stack ADV factory firmware.
use m5fxx_core::{Color565, DisplayBuffer};
unsafe extern "C" {
    fn m5fxx_factory_init();
    fn m5fxx_factory_step(ms: u32, out: *mut u16, brightness: *mut u8);
    fn m5fxx_factory_key(row: u8, col: u8, pressed: bool);
    fn m5fxx_factory_home(pressed: bool);
    fn m5fxx_factory_shutdown();
}
pub struct FactoryFirmware {
    pub brightness: u8,
}
impl Default for FactoryFirmware {
    fn default() -> Self {
        unsafe {
            m5fxx_factory_init();
        }
        Self { brightness: 255 }
    }
}
impl FactoryFirmware {
    pub fn step(&mut self, ms: u32, display: &mut DisplayBuffer) {
        let mut frame = [0u16; 240 * 135];
        unsafe {
            m5fxx_factory_step(ms, frame.as_mut_ptr(), &mut self.brightness);
        }
        for (p, v) in display.pixels.iter_mut().zip(frame) {
            *p = Color565(v);
        }
        display.mark_dirty();
    }
    pub fn key(&self, row: u8, col: u8, pressed: bool) {
        unsafe {
            m5fxx_factory_key(row, col, pressed);
        }
    }
    pub fn home(&self, pressed: bool) {
        unsafe {
            m5fxx_factory_home(pressed);
        }
    }
}
impl Drop for FactoryFirmware {
    fn drop(&mut self) {
        unsafe {
            m5fxx_factory_shutdown();
        }
    }
}
