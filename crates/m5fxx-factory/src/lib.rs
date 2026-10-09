//! Native port of the pinned M5Stack ADV factory firmware.
use m5fxx_core::{Color565, DisplayBuffer};
use std::sync::atomic::{AtomicBool, Ordering};
static OWNED: AtomicBool = AtomicBool::new(false);
unsafe extern "C" {
    fn m5fxx_factory_init();
    fn m5fxx_factory_step(ms: u32, out: *mut u16, brightness: *mut u8);
    fn m5fxx_factory_key(row: u8, col: u8, pressed: bool);
    fn m5fxx_factory_home(pressed: bool);
    fn m5fxx_factory_shutdown();
    fn m5fxx_factory_inputs(inputs: *const SimulationInputs);
    fn m5fxx_factory_sd_root(root: *const std::ffi::c_char);
}
pub struct FactoryFirmware {
    pub brightness: u8,
}
impl Default for FactoryFirmware {
    fn default() -> Self {
        assert!(
            !OWNED.swap(true, Ordering::AcqRel),
            "only one native firmware instance may run per process"
        );
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
        OWNED.store(false, Ordering::Release);
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct SimulationInputs {
    pub accel: [f32; 3],
    pub gyro: [f32; 3],
    pub audio_amplitude: f32,
    pub sd_bytes: u64,
    pub battery: u8,
    pub wifi: u8,
    pub ble: u8,
    pub usb: u8,
    pub sd: u8,
    pub cap: u8,
}
impl Default for SimulationInputs {
    fn default() -> Self {
        Self {
            accel: [0.0, 0.0, 1.0],
            gyro: [0.0; 3],
            audio_amplitude: 0.0,
            sd_bytes: 0,
            battery: 100,
            wifi: 0,
            ble: 0,
            usb: 0,
            sd: 1,
            cap: 0,
        }
    }
}
impl FactoryFirmware {
    pub fn inputs(&self, inputs: &SimulationInputs) {
        unsafe {
            m5fxx_factory_inputs(inputs);
        }
    }
    pub fn reset(&mut self) {
        unsafe {
            m5fxx_factory_shutdown();
            m5fxx_factory_init();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boots_navigates_and_preserves_controller_pixels() {
        let mut firmware = FactoryFirmware::default();
        let mut frame = DisplayBuffer::new();
        for _ in 0..100 {
            firmware.step(16, &mut frame);
        }
        assert!(
            frame.pixels.iter().any(|p| *p != Color565::BLACK),
            "boot screen must be drawn"
        );
        let mut controller = m5fxx_core::st7789::St7789::default();
        controller.command(0x11);
        controller.command(0x21);
        controller.command(0x29);
        controller.write_landscape(&frame.pixels);
        let mut presented = DisplayBuffer::new();
        controller.present(&mut presented);
        assert_eq!(frame.pixels, presented.pixels);
        firmware.key(2, 13, true);
        for _ in 0..4 {
            firmware.step(16, &mut frame);
        }
        firmware.key(2, 13, false);
        for _ in 0..30 {
            firmware.step(16, &mut frame);
        }
        assert_eq!(
            frame.pixels[0],
            Color565::from_rgb888(0x33, 0x33, 0x33),
            "original theme background"
        );
        let launcher = frame.pixels;
        firmware.key(3, 12, true);
        firmware.step(16, &mut frame);
        firmware.key(3, 12, false);
        for _ in 0..30 {
            firmware.step(16, &mut frame);
        }
        assert_ne!(frame.pixels, launcher, "launcher must react to key events");
        firmware.reset();
        for _ in 0..100 {
            firmware.step(16, &mut frame);
        }
        assert!(
            frame.pixels.iter().any(|p| *p != Color565::BLACK),
            "restart must boot again"
        );
        assert_reference(&frame, "boot");
        fn advance(f: &mut FactoryFirmware, d: &mut DisplayBuffer, n: usize) {
            for _ in 0..n {
                f.step(16, d);
            }
        }
        fn tap(f: &mut FactoryFirmware, d: &mut DisplayBuffer, row: u8, col: u8) {
            f.key(row, col, true);
            advance(f, d, 3);
            f.key(row, col, false);
            advance(f, d, 30);
        }
        tap(&mut firmware, &mut frame, 2, 13);
        assert_reference(&frame, "launcher");
        for i in 0..13 {
            tap(&mut firmware, &mut frame, 2, 13);
            advance(&mut firmware, &mut frame, 150);
            assert_reference(&frame, &format!("app-{i:02}"));
            controller.write_landscape(&frame.pixels);
            controller.present(&mut presented);
            assert_eq!(frame.pixels, presented.pixels);
            firmware.home(true);
            advance(&mut firmware, &mut frame, 3);
            firmware.home(false);
            advance(&mut firmware, &mut frame, 30);
            tap(&mut firmware, &mut frame, 3, 12);
        }
    }
}

impl FactoryFirmware {
    pub fn sd_root(&self, root: &std::path::Path) {
        if let Ok(root) = std::ffi::CString::new(root.to_string_lossy().as_bytes()) {
            unsafe {
                m5fxx_factory_sd_root(root.as_ptr());
            }
        }
    }
}

#[cfg(test)]
fn assert_reference(frame: &DisplayBuffer, name: &str) {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/frames")
            .join(format!("{name}.png")),
    )
    .unwrap();
    let reference = image::load_from_memory(&bytes).unwrap().to_rgba8();
    let mut rgba = vec![0; 240 * 135 * 4];
    frame.to_rgba8888(&mut rgba);
    assert_eq!(
        reference.as_raw(),
        &rgba,
        "pinned original rendering frame: {name}"
    );
}
