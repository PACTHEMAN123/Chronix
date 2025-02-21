# Chronix

Chronix 是一个用Rust语言编写的RISC-V架构操作系统内核。它专注于提供一个稳定、高效且易于理解的系统实现。

## 特性

- 基于RISC-V架构
- 完全由Rust语言实现
- 支持虚拟内存管理
- 提供基础进程调度
- 实现了Unix风格的系统调用

## 快速开始

### 环境要求

1. Rust工具链
```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装相关组件
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils --vers =0.3.3
rustup component add llvm-tools-preview
rustup component add rust-src
```

2. QEMU模拟器
```bash
# Ubuntu/Debian系统安装QEMU
sudo apt install qemu-system-riscv64
```

3. RISC-V调试工具
```bash
# 安装GDB调试器
sudo apt install gdb-multiarch
```

### 构建与运行

1. 克隆仓库
```bash
git clone [your-repository-url]
cd chronix
```

2. 构建内核
```bash
cd os
make build
```

3. 运行内核
```bash
make run
```

4. 调试内核
```bash
# 启动调试会话
make debug

# 或者分别启动服务器和客户端
make gdbserver
make gdbclient
```

## 开发

### 目录结构

```
chronix/
├── os/             # 内核源代码
├── bootloader/     # 引导加载程序
└── docs/          # 文档
```

### 构建选项

- `make build`: 构建内核
- `make run`: 运行内核
- `make debug`: 启动调试会话
- `make clean`: 清理构建产物
- `make doc`: 生成文档

## 贡献

欢迎提交Issue和Pull Request！

## 许可证

[您的许可证类型]

## 致谢

感谢所有为Chronix项目做出贡献的开发者。
