# Chronix

Chronix 是一个用Rust语言编写的RISC-V架构操作系统内核。它旨在提供一个稳定、高效且易于理解的系统实现，适合学习和研究操作系统的开发者。

## 特性

- **基于RISC-V架构**：支持现代RISC-V指令集，适合嵌入式和高性能计算。
- **完全由Rust语言实现**：利用Rust的安全性和并发性特性，确保内核的稳定性和安全性。
- **支持虚拟内存管理**：实现了基本的内存分页和管理功能。
- **提供基础进程调度**：支持多任务调度，提升系统响应能力。
- **实现了Unix风格的系统调用**：提供标准的系统调用接口，便于应用程序开发。

## 环境配置

1. **设置QEMU RISC-V**：下载并编译QEMU 7.0.0，支持RISC-V架构的模拟。
2. 安装 **musl-gcc**：用于编译 lwext4 。
3. 安装Rust工具链：使用Rustup安装Rust nightly版本，并添加必要的组件和目标。

## 快速开始

### 构建与运行

1. **克隆仓库**
   ```bash
   git clone https://github.com/PACTHEMAN123/Chronix.git
   cd Chronix
   ```

2. **构建内核**
   ```bash
   make build
   ```

3. **运行内核**
   ```bash
   make run
   ```

4. **调试内核**
   ```bash
   make debug
   ```

### 构建选项

- `make build`: 构建内核和用户程序。
- `make run`: 在QEMU中运行内核。
- `make debug`: 启动调试会话。
- `make clean`: 清理构建产物。

## 目录结构

```
Chronix/
├── os/             # 内核源代码
├── bootloader/     # 引导加载程序
├── user/           # 用户程序
├── tests/          # 测试用例
```

## 贡献

欢迎提交Issue和Pull Request，帮助我们改进 Chronix。

## 许可证

使用 GNU General Public License v3.0 许可证。

## 致谢

感谢所有为 Chronix 项目做出贡献的开发者。



