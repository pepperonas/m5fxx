use m5fxx_esp32::Esp32Firmware;
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let path = args
        .get(1)
        .ok_or("Usage: run image.bin [seconds] [output.png]")?;
    let seconds: u64 = args.get(2).map(|s| s.parse()).transpose()?.unwrap_or(20);
    let mut cpu = Esp32Firmware::start(&std::fs::read(path)?)?;
    let mut display = m5fxx_core::DisplayBuffer::default();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(seconds) {
        cpu.poll(&mut display)?;
        if args.iter().any(|a| a == "--right") {
            let elapsed = start.elapsed().as_millis();
            if (25_000..25_150).contains(&elapsed) {
                cpu.key(3, 12, true)?;
            }
            if (25_150..25_300).contains(&elapsed) {
                cpu.key(3, 12, false)?;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    println!("{}; LCD writes={}", cpu.status, cpu.has_pixels);
    for line in &cpu.logs {
        println!("{line}");
    }
    if let Some(path) = args.get(3) {
        let mut rgba = vec![0; 240 * 135 * 4];
        display.to_rgba8888(&mut rgba);
        image::save_buffer(path, &rgba, 240, 135, image::ColorType::Rgba8)?;
    }
    if !cpu.has_pixels {
        return Err("Firmware has not written the LCD".into());
    }
    Ok(())
}
