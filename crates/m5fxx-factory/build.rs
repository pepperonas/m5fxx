use std::path::{Path, PathBuf};
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "cpp" || e == "c") {
            out.push(path);
        }
    }
}
fn main() {
    let vendor = Path::new("../../vendor");
    let mut b = cc::Build::new();
    b.cpp(true)
        .std("c++17")
        .warnings(false)
        .define("M5FXX_NATIVE", None)
        .define("FMT_HEADER_ONLY", None);
    for path in [
        "native",
        "../../vendor/M5GFX/src",
        "../../vendor/mooncake/src",
        "../../vendor/mooncake_log/src",
        "../../vendor/smooth_ui_toolkit/src",
        "../../vendor/cardputer-adv/main",
        "../../vendor/cardputer-adv/main/apps",
        "../../vendor/cardputer-adv/main/assets",
        "../../vendor/cardputer-adv/main/hal/keyboard",
        "../../vendor/cardputer-adv/main/hal/cap_lora868/TinyGPSPlus",
        "../../vendor/cardputer-adv/main/apps/app_repl/PikaScript/pikascript-api",
        "../../vendor/cardputer-adv/main/apps/app_repl/PikaScript/pikascript-core",
    ] {
        b.include(path);
    }
    let mut files = Vec::new();
    for lib in ["mooncake", "mooncake_log", "smooth_ui_toolkit"] {
        sources(&vendor.join(lib).join("src"), &mut files);
    }
    for p in [
        "lgfx/v1/LGFXBase.cpp",
        "lgfx/v1/LGFX_Sprite.cpp",
        "lgfx/v1/lgfx_fonts.cpp",
        "lgfx/v1/misc/common_function.cpp",
        "lgfx/v1/misc/SpriteBuffer.cpp",
        "lgfx/v1/misc/pixelcopy.cpp",
    ] {
        files.push(vendor.join("M5GFX/src").join(p));
    }
    sources(
        &vendor.join("cardputer-adv/main/apps/utils/raylib"),
        &mut files,
    );
    files.push(PathBuf::from("native/adapter.cpp"));
    files.push(PathBuf::from("native/key_conversion.cpp"));
    files.push(vendor.join("cardputer-adv/main/hal/cap_lora868/TinyGPSPlus/TinyGPS++.cpp"));
    // Native build excludes ESP-IDF HAL and supplies the same application-facing services.
    for app in [
        "app_launcher",
        "app_wifi_scan",
        "app_record",
        "app_chat",
        "app_remote",
        "app_set_wifi",
        "app_clock",
        "app_keyboard",
        "app_imu",
        "app_sdcard",
        "app_stringir_toolkit",
        "app_lora_chat",
        "app_gps",
    ] {
        sources(
            &vendor.join("cardputer-adv/main/apps").join(app),
            &mut files,
        );
    }
    sources(
        &vendor.join("cardputer-adv/main/apps/app_repl/view"),
        &mut files,
    );
    files.push(vendor.join("cardputer-adv/main/apps/app_repl/app_repl.cpp"));
    for f in &files {
        println!("cargo:rerun-if-changed={}", f.display());
        b.file(f);
    }
    b.compile("m5fxx_factory");
    let mut c = cc::Build::new();
    c.flag_if_supported("-mmacosx-version-min=11.0")
        .warnings(false)
        .define("M5FXX_NATIVE", None)
        .include("native")
        .include(vendor.join("M5GFX/src"));
    for f in [
        "lgfx_miniz.c",
        "lgfx_qoi.c",
        "lgfx_tjpgd.c",
        "lgfx_pngle.c",
        "lgfx_qrcode.c",
    ] {
        c.file(vendor.join("M5GFX/src/lgfx/utility").join(f));
    }
    c.file(vendor.join("M5GFX/src/lgfx/Fonts/efont/lgfx_efont_cn.c"));
    c.compile("m5fxx_fonts");
    let mut pika = cc::Build::new();
    pika.flag_if_supported("-mmacosx-version-min=11.0")
        .warnings(false);
    for p in ["pikascript-api", "pikascript-core"] {
        pika.include(
            vendor
                .join("cardputer-adv/main/apps/app_repl/PikaScript")
                .join(p),
        );
    }
    let mut pf = Vec::new();
    sources(
        &vendor.join("cardputer-adv/main/apps/app_repl/PikaScript"),
        &mut pf,
    );
    for f in pf {
        if f.extension().is_some_and(|e| e == "c") {
            pika.file(f);
        }
    }
    pika.compile("m5fxx_pika");
    println!("cargo:rerun-if-changed=native");
}
