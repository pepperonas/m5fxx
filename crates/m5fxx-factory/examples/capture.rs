//! Deterministic native firmware frames for visual verification.
use m5fxx_core::DisplayBuffer;
use m5fxx_factory::FactoryFirmware;
fn advance(f: &mut FactoryFirmware, d: &mut DisplayBuffer, n: usize) {
    for _ in 0..n {
        f.step(16, d);
    }
}
fn key(f: &mut FactoryFirmware, d: &mut DisplayBuffer, row: u8, col: u8) {
    f.key(row, col, true);
    advance(f, d, 3);
    f.key(row, col, false);
    advance(f, d, 30);
}
fn save(d: &DisplayBuffer, name: &str) {
    let mut rgba = vec![0; 240 * 135 * 4];
    d.to_rgba8888(&mut rgba);
    image::save_buffer(
        format!("target/factory-captures/{name}.png"),
        &rgba,
        240,
        135,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
fn main() {
    std::fs::create_dir_all("target/factory-captures").unwrap();
    let mut f = FactoryFirmware::default();
    let mut d = DisplayBuffer::new();
    advance(&mut f, &mut d, 100);
    save(&d, "boot");
    key(&mut f, &mut d, 2, 13);
    save(&d, "launcher");
    for i in 0..13 {
        key(&mut f, &mut d, 2, 13);
        advance(&mut f, &mut d, 150);
        save(&d, &format!("app-{i:02}"));
        f.home(true);
        advance(&mut f, &mut d, 3);
        f.home(false);
        advance(&mut f, &mut d, 30);
        key(&mut f, &mut d, 3, 12);
    }
}
