#!/bin/bash

set -e

ARCHIVE="testcase.tar.xz"     # 写死的压缩包名
TARGET_DIR="testcase"         # 解压后文件夹名

case "$1" in
  extract)
    echo "unpacking $ARCHIVE to $TARGET_DIR ..."
    rm -rf "$TARGET_DIR"
    mkdir "$TARGET_DIR"
    tar -xJf "$ARCHIVE" -C "$TARGET_DIR"
    echo "finish!"
    ;;
  pack)
    echo "packing $TARGET_DIR to $ARCHIVE ..."
    tar -cJf "$ARCHIVE" -C "$TARGET_DIR" .
    echo "finish!"
    ;;
  *)
    echo "usage: $0 [extract|pack]"
    exit 1
    ;;
esac