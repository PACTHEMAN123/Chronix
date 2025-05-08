#!/bin/bash
set -e

VENDOR_DIR="vendor/crates"
CONFIG_DIR="cargo-config"
CONFIG_FILE="$CONFIG_DIR/config.toml"

function usage() {
    echo "用法:"
    echo "  $0            初始化 vendor 和 config"
    echo "  $0 --clean    删除 vendor 和 config"
}

function vendor() {
    echo "[1/3] 创建目录: $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"

    echo "[2/3] 执行 cargo vendor 到 $VENDOR_DIR"
    cargo vendor "$VENDOR_DIR" > "$CONFIG_FILE"

    echo "[3/3] 写入配置文件到 $CONFIG_FILE ✅"
}

function clean() {
    echo "[1/2] 删除目录: $VENDOR_DIR"
    rm -rf "$VENDOR_DIR"

    echo "[2/2] 删除配置文件: $CONFIG_FILE"
    rm -f "$CONFIG_FILE"

    echo "清理完成 ✅"
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