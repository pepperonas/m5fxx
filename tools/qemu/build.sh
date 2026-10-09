#!/usr/bin/env bash
set -euo pipefail
# Public Espressif QEMU, pinned to esp-develop-9.2.2-20260417.
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
project_dir="$(cd -- "$script_dir/../.." && pwd)"
source_dir="$project_dir/target/cardputer-qemu-src"
tools_dir="$project_dir/target/cardputer-qemu-tools"
commit=40edccac415693c5130f91c01d84176ae6008566
if [[ ! -e "$source_dir/.m5fxx-patched" ]]; then
    if [[ -e "$source_dir" ]]; then
        echo "Unfinished source tree exists at $source_dir; rename it before retrying." >&2
        exit 1
    fi
    mkdir -p "$source_dir"
    curl --fail --location "https://github.com/espressif/qemu/archive/$commit.tar.gz" | tar -xz -C "$source_dir" --strip-components=1
    python3 "$script_dir/patch.py" "$source_dir"
    touch "$source_dir/.m5fxx-patched"
fi
if [[ ! -x "$tools_dir/bin/ninja" ]]; then
    python3 -m venv "$tools_dir"
    "$tools_dir/bin/pip" install ninja==1.13.2
fi
export PATH="$tools_dir/bin:$PATH"
cd "$source_dir"
if [[ ! -e build/build.ninja ]]; then
    ./configure --ninja="$tools_dir/bin/ninja" --target-list=xtensa-softmmu \
        --enable-gcrypt --disable-docs --disable-gtk --disable-sdl --disable-vnc \
        --disable-cocoa --disable-user --disable-tools --disable-guest-agent \
        --disable-capstone --disable-slirp --disable-debug-info \
        --disable-gnutls --disable-nettle
fi
ninja -C build -j "${M5FXX_BUILD_JOBS:-4}" qemu-system-xtensa
mkdir -p "$script_dir/bin"
cp build/qemu-system-xtensa "$script_dir/bin/"
cp pc-bios/esp32s3_rev0_rom.bin "$script_dir/bin/"
echo "Cardputer QEMU installed at $script_dir/bin/qemu-system-xtensa"
