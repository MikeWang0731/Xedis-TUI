# Xedis-TUI 🚀

> **现代化终端 Redis / 兼容协议交互式管理与排障工具**  
> 基于 Rust 构建，兼具极速启动、极低资源开销、高危命令拦截与集群多维观测能力。

[![Rust](https://img.shields.io/badge/Language-Rust_2021-DEA584.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Ratatui](https://img.shields.io/badge/TUI-Ratatui_0.29-blue.svg?style=flat-square)](https://github.com/ratatui/ratatui)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-green.svg?style=flat-square)](LICENSE)

---
<p align="center">
  <img src="project_docs/pics/quick-demo.gif" alt="Dual Themes" width="700" /><br />
  <sub><b>Xedis-TUI</b> · A Terminal-based Redis-compatible Management Tool</sub>
</p>

## 📖 项目简介

**Xedis-TUI** 是一款专为开发者、DevOps 工程师与初学者打造的现代化 Redis 终端交互工具。

传统 CLI 工具往往缺乏直观的可视化反馈，而桌面 GUI 客户端在轻量排障、远程服务器和跳板机场景下部署繁琐。Xedis-TUI 采用单一二进制跨平台分发，拥有极低内存占用（< 15MB）与毫秒级冷启动，集成了**交互式命令流**、**初学者生产安全守卫**、**多维性能遥测看板**与**集群拓扑管理**，让 Redis 的日常操作与问题排查在终端内高效完成。

---

## ✨ 核心特性

- 🛡️ **生产安全守卫 (Safety Guard)**：
  - 自动识别并拦截高危指令（如 `FLUSHALL`、`FLUSHDB`、`SHUTDOWN`、`KEYS *` 等）。
  - 弹窗展示风险等级、原理解析与替代建议（如引导使用非阻塞的 `/scan`），防止误操作导致线上事故。
- ⚡ **实用快捷宏 (Macro Engine)**：
  - 内置 `/scan`（安全分页扫描）、`/bigkeys`（大 Key 内存排行探测）、`/slowlog`（慢日志排查与优化建议）、`/clients`（连接分析）等内置宏。
  - 支持 `/interval` 动态调节遥测采样频率、`/theme` 实时换肤等运行时控制。
- 🌐 **集群拓扑与定向路由**：
  - 自动发现 Redis Cluster 拓扑结构，直观查看主从节点（Master / Replica）、健康状态与槽位分布（Slot Range）。
  - 支持节点定向路由：使用 `@<node_id>` 或 `@<ip:port>` 将命令精准派发至指定节点，支持 `@all` 广播执行。
- 📊 **多维性能看板与慢日志**：
  - **Telemetry 看板**：实时内存水位（Gauge）、QPS 历史趋势图（Sparkline）、命中率、网络吞吐与连接数监控。
  - **Slowlog 分析**：慢查询日志列表展示，标注执行耗时与优化提示。
- 💡 **智能上下文补全与即时文档**：
  - 支持 Redis 原生命令、子命令、快捷宏与集群节点的上下文智能联想。
  - 实时高亮语法与输入提示；按 `F1` 可随时唤起内置的“排障与知识库手册”。
- 🎨 **双主题与多布局自适应**：
  - 预设 4 种分栏布局：`Balanced` (平衡)、`Focus` (专注命令行)、`Monitor` (侧重监控)、`Zen` (全屏沉浸)。
  - 支持深色（Dark）与浅色（Light）配色，支持动态微调分栏比例。

---

## 🖥️ 快速开始

> 在 [Github Release](https://github.com/MikeWang0731/Xedis-TUI/releases) 内下载对应平台的文件，解压后可以直接运行

### 前置要求

- 安装 [Rust 工具链](https://rustup.rs/) (Rust 1.75+)
- 可选：本地或远程正在运行的 Redis 服务（支持 Redis 6/7+、KeyDB、Valkey、Dragonfly 等兼容协议）

### 安装与构建
本机编译
```bash
# 克隆仓库
git clone https://github.com/MikeWang0731/Xedis-TUI.git
cd Xedis-TUI

# 编译并运行 (以 Demo 模式运行)
cargo run

# 连接本地 Redis (127.0.0.1:6379)
cargo run -- -H 127.0.0.1 -p 6379

# 连接本地 Redis 集群 (其中一个 node 是 127.0.0.1:6379)
cargo run -- -H 127.0.0.1 -p 6379 -c
```
本机构建 (以 macOS 为例)
```bash
# 本地构建 (macOS)
# 安装 Target
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# 构建
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# （可选）利用 macOS lipo 工具合并成通用二进制（Universal Binary）
lipo -create -output xedis-tui-universal \
        target/aarch64-apple-darwin/release/xedis-tui \
        target/x86_64-apple-darwin/release/xedis-tui

# 本地构建 (Linux & Windows)
# 1. 安装 cross
cargo install cross --git https://github.com/cross-rs/cross

# 2. 确保 Docker (如 Docker Desktop / OrbStack 等) 正在运行
# 3. 编译 Linux x86_64 静态二进制
cross build --release --target x86_64-unknown-linux-musl

# 4. 编译 Linux ARM64 静态二进制
cross build --release --target aarch64-unknown-linux-musl

# 5. 编译 Windows x86_64 二进制 (GNU 工具链)
cross build --release --target x86_64-pc-windows-gnu
```

### 常用启动参数(假设编译好的二进制文件叫 `xedis-tui`)

```bash
# 以 Demo 模式运行
xedis-tui

# 打开帮助
xedis-tui -h

# 指定远程主机与端口
xedis-tui -H 192.168.1.100 -p 6379

# 密码认证与集群模式
xedis-tui -H 10.0.0.1 -p 7000 -a "your_password" --cluster
```

---

## ⌨️ 常用快捷键与命令

### 全局与导航快捷键

| 快捷键 | 功能描述 |
| :--- | :--- |
| `Tab` | 在左侧交互流窗口与右侧看板之间切换焦点 |
| `F1` | 打开 / 关闭内置“新手秘籍与排障手册”弹窗 |
| `F2` / `F3` / `F4` | 快速切换右侧看板：遥测监控 (`Telemetry`) / 集群拓扑 (`Cluster`) / 慢日志 (`Slowlog`) |
| `F5` / `Ctrl + B` | 循环切换 4 种预设布局 (`Balanced` $\rightarrow$ `Focus` $\rightarrow$ `Monitor` $\rightarrow$ `Zen`) |
| `Ctrl + C` / `Ctrl + Q` | 退出程序 |

### 常用快捷宏 (Macros)

在输入框中直接输入 `/` 即可触发宏命令补全：

- `/scan [pattern] [count]`：通过非阻塞游标安全遍历 Key（如 `/scan user:* 50`）
- `/bigkeys [count]`：分析并统计大 Key 及内存排行（如 `/bigkeys 10`）
- `/slowlog [count]`：拉取最新慢查询日志及建议
- `/clients`：查看当前连接的活跃客户端状态
- `/interval [500ms|1s|2s|pause]`：动态调整遥测指标轮询刷新间隔
- `/theme [dark|light|toggle]`：切换深色 / 浅色主题
- `/help`：查看所有宏与快捷键对照表

---

## 🧱 核心代码结构

项目采用典型的分层与组件化设计，业务逻辑、网络驱动与 UI 渲染解耦：

```text
src/
├── main.rs                 # 程序入口：CLI 参数解析、终端生命周期初始化与主事件循环
├── app.rs                  # 核心应用状态机：输入缓冲、视图状态、焦点管理与事件调度
├── config.rs               # 用户配置管理：支持 ~/.config/xedis/config.toml 自定义配置
├── lib.rs                  # 库模块导出声明
│
├── backend/                # 驱动与通信层
│   ├── client.rs           # Redis 异步客户端（单机 / 集群适配、拓扑探测与命令派发）
│   ├── cluster_info.rs     # 集群元数据、节点状态与 Slot 槽位映射
│   └── formatter.rs        # RESP 响应解析与格式化（纯文本、JSON、表格高亮呈现）
│
├── core/                   # 核心业务引擎
│   ├── autocomplete.rs     # 智能补全引擎：命令字典、宏、集群节点名自动提示
│   ├── guard.rs            # 初学者安全护栏：高危命令识别、拦截与风险等级判定
│   ├── macro_engine.rs     # `/` 快捷宏执行与解析引擎
│   ├── router.rs           # 输入解析器与 `@node` 定向路由派发
│   ├── telemetry.rs        # 后台指标采集器（QPS 历史趋势、内存、吞吐滑动窗口）
│   └── history.rs          # 命令历史记录与持久化管理器
│
└── ui/                     # 终端用户界面（基于 Ratatui 构建）
    ├── layout.rs           # 动态布局切割器（Balanced / Focus / Monitor / Zen）
    ├── stream_view.rs      # 左侧命令交互流与执行卡片渲染
    ├── telemetry_view.rs   # 性能遥测监控看板（Sparkline 图表、Gauge 进度条）
    ├── cluster_view.rs     # 集群拓扑树状结构与节点列表展示
    ├── slowlog_view.rs     # 慢查询日志分析面板
    ├── theme.rs            # 配色主题系统（深色 / 浅色）
    └── popups/             # 浮层与弹窗组件
        ├── autocomplete.rs # 自动补全浮窗
        ├── guard_popup.rs  # 高危命令拦截二次确认弹窗
        └── help_popup.rs   # F1 帮助手册与排障指南弹窗
```

---

## 🤝 参与贡献

欢迎提交 Issue 或 Pull Request！
无论是新功能建议、代码优化、文档完善还是 Bug 反馈，都非常感谢您的支持。

1. Fork 本仓库
2. 创建您的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的修改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到远程分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📄 开源许可证

本项目采用 [MIT License](LICENSE) 开源许可。
