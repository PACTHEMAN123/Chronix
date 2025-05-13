#!/bin/bash
set -e

VENDOR_DIR="vendor"
CONFIG_DIR="cargo"
CONFIG_FILE="$CONFIG_DIR/config.toml"

function usage() {
    echo "usage:"
    echo "  $0            init vendor and config"
    echo "  $0 --clean    delete vendor and config"
}

function vendor() {
    echo "[1/4] create config dir: $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"

    echo "[2/4] cargo vendor to $VENDOR_DIR"
    cargo vendor "$VENDOR_DIR" > "$CONFIG_FILE"

    echo "[3/4] write a config file $CONFIG_FILE ✅"

    echo "[4/4] add default target to $CONFIG_FILE"
    cat >> "$CONFIG_FILE" <<EOF
# set default build target
[build]
target = "riscv64gc-unknown-none-elf"
[profile.release]
debug = true
EOF

    echo "✅ finish vendor"
}

function clean() {
    echo "[1/2] delete dir: $VENDOR_DIR"
    rm -rf "$VENDOR_DIR"

    echo "[2/2] replace config file: $CONFIG_FILE"
    cat > "$CONFIG_FILE" <<EOF
# set default build target
[build]
target = "riscv64gc-unknown-none-elf"
[profile.release]
debug = true
EOF
    rm -rf .cargo



    echo "finish un-vendor ✅"
}

case "$1" in
    "" )
        vendor
        ;;
    --clean )
        clean
        ;;
    * )
        usage
        ;;
esac