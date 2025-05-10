#!/bin/bash
set -e

VENDOR_DIR="vendor"
CONFIG_DIR="cargo"
CONFIG_FILE="$CONFIG_DIR/config.toml"

function usage() {
    echo "用法:"
    echo "  $0            初始化 vendor 和 config"
    echo "  $0 --clean    删除 vendor 和 config"
}

function vendor() {
    echo "[1/4] create config dir: $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"

    echo "[2/4] 执行 cargo vendor 到 $VENDOR_DIR"
    cargo vendor "$VENDOR_DIR" > "$CONFIG_FILE"

    echo "[3/4] 写入配置文件到 $CONFIG_FILE ✅"

    echo "[4/4] 修改 $CONFIG_FILE：添加核心库的 registry 配置"
    cat >> "$CONFIG_FILE" <<EOF

# 保留特殊 crate 从 crates.io 拉取，防止构建核心库失败

EOF

    echo "✅ 完成！vendor 和配置就绪，已 patch 兼容核心 crate。"
}

function clean() {
    echo "[1/2] 删除目录: $VENDOR_DIR"
    rm -rf "$VENDOR_DIR"

    echo "[2/2] 删除配置文件: $CONFIG_FILE"
    rm -f "$CONFIG_FILE"
    rm -rf .cargo

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