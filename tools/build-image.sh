#!/bin/bash
# Build a bootable BIOS/UEFI disk image from the exokernel ELF binary.
# The builder runs from /tmp to avoid the exokernel's .cargo/config.toml
# which forces bare-metal compilation.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
KERNEL_ELF="$PROJECT_DIR/target/x86_64-unknown-none/release/exokernel"
OUT_DIR="$PROJECT_DIR/target/boot"

mkdir -p "$OUT_DIR"

# Build kernel if not already built
if [ ! -f "$KERNEL_ELF" ]; then
    echo "Building kernel..."
    . "$HOME/.cargo/env"
    cd "$PROJECT_DIR"
    cargo build --release
fi

echo "Creating bootable disk image..."

# Build from /tmp to avoid .cargo/config.toml interference
BUILDER_DIR="/tmp/paryanta-image-builder"
rm -rf "$BUILDER_DIR"
mkdir -p "$BUILDER_DIR/src"

cat > "$BUILDER_DIR/Cargo.toml" << 'EOF'
[package]
name = "image-builder"
version = "0.1.0"
edition = "2021"

[dependencies]
bootloader = "0.11"
EOF

# Use absolute paths in the builder
cat > "$BUILDER_DIR/src/main.rs" << EOF
use std::path::Path;

fn main() {
    let kernel_path = Path::new("${KERNEL_ELF}");
    let out_dir = Path::new("${OUT_DIR}");

    println!("Kernel ELF: {}", kernel_path.display());

    // Create BIOS boot image
    let bios_path = out_dir.join("paryantaos-bios.img");
    bootloader::BiosBoot::new(kernel_path)
        .create_disk_image(&bios_path)
        .expect("Failed to create BIOS disk image");
    println!("BIOS image: {}", bios_path.display());

    // Create UEFI boot image
    let uefi_path = out_dir.join("paryantaos-uefi.img");
    bootloader::UefiBoot::new(kernel_path)
        .create_disk_image(&uefi_path)
        .expect("Failed to create UEFI disk image");
    println!("UEFI image: {}", uefi_path.display());
}
EOF

echo "Building image builder (from /tmp, host target)..."
. "$HOME/.cargo/env"
cd "$BUILDER_DIR"
cargo run --release 2>&1

echo ""
echo "Done! Boot images at:"
echo "  BIOS: $OUT_DIR/paryantaos-bios.img"
echo "  UEFI: $OUT_DIR/paryantaos-uefi.img"
echo ""
echo "Run with QEMU:"
echo "  qemu-system-x86_64 -drive format=raw,file=$OUT_DIR/paryantaos-bios.img -serial stdio -display none"
