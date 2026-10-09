//! Actual ESP32-S3 execution in a child process, with an explicit peripheral bridge.
use m5fxx_core::{st7789::St7789, DisplayBuffer};
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX_PACKET: usize = 65536;
enum Event {
    Packet(u8, Vec<u8>),
    Log(String),
    Closed(String),
}

/// A merged ESP32-S3 flash image. Input bytes are never modified.
pub fn prepare_flash(bytes: &[u8]) -> io::Result<Vec<u8>> {
    let invalid = |s: &str| io::Error::new(io::ErrorKind::InvalidData, s);
    if bytes.len() < 24 || bytes[0] != 0xe9 {
        return Err(invalid(
            "Expected an ESP32-S3 .bin image (HEX and ELF are not flash images)",
        ));
    }
    if u16::from_le_bytes([bytes[12], bytes[13]]) != 9 {
        return Err(invalid(
            "Firmware image targets a different chip; ESP32-S3 required",
        ));
    }
    let size = match bytes[3] >> 4 {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        4 => 16,
        _ => return Err(invalid("Unsupported flash capacity")),
    } * 1024
        * 1024;
    if size < 2 * 1024 * 1024 || bytes.len() > size {
        return Err(invalid("Image exceeds its declared flash capacity"));
    }
    if bytes.len() <= 0x10018
        || bytes[0x10000] != 0xe9
        || bytes.get(0x8000..0x8002) != Some(&[0xaa, 0x50][..])
    {
        return Err(invalid("A merged image with bootloader, partition table and application is required; an application-only .bin cannot boot directly"));
    }
    let mut flash = vec![0xff; size];
    flash[..bytes.len()].copy_from_slice(bytes);
    Ok(flash)
}

fn read_packet(reader: &mut impl Read) -> io::Result<(u8, Vec<u8>)> {
    let mut header = [0; 5];
    reader.read_exact(&mut header)?;
    let len = u32::from_le_bytes(header[1..].try_into().unwrap()) as usize;
    if len > MAX_PACKET || header[0] > 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid Cardputer bridge packet",
        ));
    }
    let mut data = vec![0; len];
    reader.read_exact(&mut data)?;
    Ok((header[0], data))
}

pub struct Esp32Firmware {
    child: Child,
    bridge: TcpStream,
    events: Option<Receiver<Event>>,
    threads: Vec<JoinHandle<()>>,
    directory: PathBuf,
    pub panel: St7789,
    pub logs: VecDeque<String>,
    pub status: String,
    pub has_pixels: bool,
    pub connected: bool,
    paused: bool,
    stopped: bool,
    lcd_command: u8,
}
impl Esp32Firmware {
    pub fn qemu_path() -> PathBuf {
        std::env::var_os("M5FXX_QEMU")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tools/qemu/bin/qemu-system-xtensa")
            })
    }
    pub fn start(bytes: &[u8]) -> io::Result<Self> {
        let flash = prepare_flash(bytes)?;
        let qemu = Self::qemu_path();
        if !qemu.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Cardputer QEMU missing: run tools/qemu/build.sh or set M5FXX_QEMU",
            ));
        }
        let directory = std::env::temp_dir().join(format!(
            "m5fxx-esp32-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir(&directory)?;
        let result = Self::spawn(&qemu, &directory, &flash);
        if result.is_err() {
            let _ = std::fs::remove_dir_all(&directory);
        }
        result
    }
    fn spawn(qemu: &Path, directory: &Path, flash: &[u8]) -> io::Result<Self> {
        let flash_path = directory.join("flash.bin");
        std::fs::write(&flash_path, flash)?;
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let mut command = Command::new(qemu);
        if let Some(parent) = qemu.parent() {
            for bios in [
                parent.to_path_buf(),
                parent.join("pc-bios"),
                parent.join("../pc-bios"),
            ] {
                if bios.join("esp32s3_rev0_rom.bin").is_file() {
                    command.arg("-L").arg(bios);
                    break;
                }
            }
        }
        let mut child = command
            .args([
                "-machine",
                "esp32s3",
                "-nographic",
                "-monitor",
                "none",
                "-serial",
                "stdio",
                "-no-reboot",
            ])
            .arg("-drive")
            .arg(format!("file={},if=mtd,format=raw", flash_path.display()))
            .args([
                "-global",
                "driver=timer.esp32s3.timg,property=wdt_disable,value=true",
            ])
            .arg("-chardev")
            .arg(format!(
                "socket,id=cardputer,host=127.0.0.1,port={}",
                address.port()
            ))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let (tx, rx) = mpsc::sync_channel(256);
        let threads = vec![
            log_reader(child.stdout.take().unwrap(), tx.clone()),
            log_reader(child.stderr.take().unwrap(), tx.clone()),
        ];
        let deadline = Instant::now() + Duration::from_secs(5);
        let bridge = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                    if child.try_wait()?.is_some() {
                        let _ = child.kill();
                        return Err(io::Error::other(
                            "QEMU exited before connecting the Cardputer bridge",
                        ));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(e);
                }
            }
        };
        bridge.set_nonblocking(false)?;
        bridge.set_nodelay(true)?;
        let mut reader = bridge.try_clone()?;
        let mut threads = threads;
        threads.push(thread::spawn(move || loop {
            match read_packet(&mut reader) {
                Ok((kind, data)) => {
                    if tx.send(Event::Packet(kind, data)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Event::Closed(e.to_string()));
                    break;
                }
            }
        }));
        Ok(Self {
            child,
            bridge,
            events: Some(rx),
            threads,
            directory: directory.to_owned(),
            panel: St7789::default(),
            logs: VecDeque::new(),
            status: "ESP32-S3 CPU booting".into(),
            has_pixels: false,
            connected: false,
            paused: false,
            stopped: false,
            lcd_command: 0,
        })
    }
    pub fn poll(&mut self, display: &mut DisplayBuffer) -> io::Result<()> {
        let mut changed = false;
        for _ in 0..4096 {
            match self.events.as_ref().unwrap().try_recv() {
                Ok(Event::Packet(0, data)) => {
                    for c in data {
                        self.lcd_command = c;
                        self.panel.command(c);
                    }
                    changed = true;
                }
                Ok(Event::Packet(1, data)) => {
                    self.panel.data(&data);
                    self.has_pixels |= matches!(self.lcd_command, 0x2c | 0x3c) && !data.is_empty();
                    changed = true;
                }
                Ok(Event::Packet(2, data)) => {
                    if let Some(v) = data.first() {
                        self.panel.brightness = *v;
                        changed = true;
                    }
                }
                Ok(Event::Packet(3, data)) => {
                    if data != b"M5FXXCP1" {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Incompatible QEMU bridge",
                        ));
                    }
                    self.connected = true;
                    self.status = "ESP32-S3 CPU running · awaiting LCD writes".into();
                }
                Ok(Event::Log(line)) => {
                    if line.contains("Guru Meditation") || line.contains("assert failed") {
                        self.status = format!("Firmware fault: {line}");
                    }
                    self.logs.push_back(line);
                    while self.logs.len() > 100 {
                        self.logs.pop_front();
                    }
                }
                Ok(Event::Closed(error)) => {
                    self.stopped = true;
                    self.status = format!("Emulator stopped: {error}");
                    break;
                }
                Ok(_) => {}
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        if changed {
            self.panel.present(display);
        }
        if let Some(exit) = self.child.try_wait()? {
            self.stopped = true;
            self.status = format!("Emulator exited ({exit}); see UART log");
        } else if self.has_pixels && !self.stopped && !self.status.starts_with("Firmware fault") {
            self.status = "ESP32-S3 CPU running · live LCD".into();
        }
        Ok(())
    }
    pub fn key(&mut self, row: u8, col: u8, pressed: bool) -> io::Result<()> {
        if row >= 4 || col >= 14 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid matrix key",
            ));
        }
        self.bridge.write_all(&[b'K', row, col, u8::from(pressed)])
    }
    pub fn home(&mut self, pressed: bool) -> io::Result<()> {
        self.bridge.write_all(&[b'H', u8::from(pressed)])
    }
    pub fn set_paused(&mut self, paused: bool) -> io::Result<()> {
        if paused != self.paused {
            self.bridge.write_all(&[b'P', u8::from(paused)])?;
            self.paused = paused;
        }
        Ok(())
    }
    pub fn reset(&mut self) -> io::Result<()> {
        self.bridge.write_all(b"R")
    }
}
fn log_reader(mut reader: impl Read + Send + 'static, tx: SyncSender<Event>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0; 1024];
        let mut line = Vec::new();
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            for &byte in &buf[..n] {
                if byte == b'\n' || line.len() >= 1024 {
                    if !line.is_empty()
                        && tx
                            .send(Event::Log(String::from_utf8_lossy(&line).trim().to_owned()))
                            .is_err()
                    {
                        return;
                    }
                    line.clear();
                } else if byte != b'\r' {
                    line.push(byte);
                }
            }
        }
    })
}
impl Drop for Esp32Firmware {
    fn drop(&mut self) {
        self.events.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = self.bridge.shutdown(std::net::Shutdown::Both);
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn merged() -> Vec<u8> {
        let mut b = vec![0xff; 0x10020];
        b[0] = 0xe9;
        b[3] = 0x3f;
        b[12] = 9;
        b[13] = 0;
        b[0x8000] = 0xaa;
        b[0x8001] = 0x50;
        b[0x10000] = 0xe9;
        b
    }
    #[test]
    fn merged_image_is_padded_without_modifying_input() {
        let b = merged();
        let f = prepare_flash(&b).unwrap();
        assert_eq!(f.len(), 8 * 1024 * 1024);
        assert_eq!(&f[..b.len()], &b);
        assert!(f[b.len()..].iter().all(|v| *v == 0xff));
    }
    #[test]
    fn reject_wrong_chip_and_application_only_images() {
        let mut b = merged();
        b[12] = 0;
        assert!(prepare_flash(&b).is_err());
        b[12] = 9;
        b[0x8000] = 0;
        assert!(prepare_flash(&b).is_err());
        assert!(prepare_flash(b":HEX").is_err());
    }
    #[test]
    fn bridge_handles_fragmented_rgb565_packets_and_rejects_oversize() {
        let packet = [1, 2, 0, 0, 0, 0xf8, 0];
        let (kind, pixels) = read_packet(&mut &packet[..]).unwrap();
        assert_eq!(kind, 1);
        assert_eq!(pixels, [0xf8, 0]);
        assert!(read_packet(&mut &[1, 0xff, 0xff, 0xff, 0xff][..]).is_err());
    }
}
