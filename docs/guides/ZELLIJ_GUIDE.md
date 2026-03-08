# Zellij 使用指南

本文档介绍如何使用 Zellij 增强 quantix-rust CLI 工具的运行体验。

## 为什么选择 Zellij？

| 特性 | Zellij | TMUX |
|------|--------|------|
| **实现语言** | Rust | C |
| **配置格式** | KDL (人类可读) | 纯文本 |
| **插件系统** | WASM 原生 | 脚本 |
| **UI/UX** | 现代、美观 | 传统 |
| **Rust 生态** | 原生集成 | 需要绑定 |

## 快速开始

### 1. 安装

```bash
# 方式 1: 使用 cargo
cargo install zellij

# 方式 2: 使用安装脚本
./scripts/zellij/install.sh

# 方式 3: 使用包管理器
# macOS
brew install zellij

# Arch Linux
sudo pacman -S zellij

# Ubuntu/Debian
sudo apt install zellij
```

### 2. 启动工作区

```bash
# 主工作区
./scripts/zellij/start-session.sh quantix main

# 监控工作区
./scripts/zellij/start-session.sh monitor monitor

# 回测工作区
./scripts/zellij/start-session.sh backtest backtest

# 开发工作区
./scripts/zellij/start-session.sh dev dev
```

## 布局说明

### 主工作区 (main)

```
┌─────────────────────────────────────┐
│          [Tab Bar]                  │
├─────────────────────────────────────┤
│          CLI 主界面 (60%)            │
├─────────────────┬───────────────────┤
│    实时状态      │     系统日志       │
│    (20%)        │      (20%)        │
├─────────────────┴───────────────────┤
│          [Status Bar]               │
└─────────────────────────────────────┘
```

**用途**: 日常操作、菜单导航、状态监控

### 监控工作区 (monitor)

```
┌─────────────────┬───────────────────┐
│   实时行情监控   │    性能指标        │
├─────────────────┼───────────────────┤
│   持仓监控      │    告警日志        │
└─────────────────┴───────────────────┘
```

**用途**: 实时监控交易状态、性能指标、告警信息

### 回测工作区 (backtest)

```
┌───────────────────────────┬─────────┐
│                          │  实时   │
│    回测控制台 (70%)       │  结果   │
│                          ├─────────┤
│                          │  日志   │
└───────────────────────────┴─────────┘
```

**用途**: 策略回测、结果查看、调试分析

### 开发工作区 (dev)

```
┌───────────────────────────┬─────────┐
│                          │ 构建测试 │
│    代码编辑 (60%)         ├─────────┤
│                          │ 代码检查 │
│                          ├─────────┤
│                          │Git 状态  │
└───────────────────────────┴─────────┘
```

**用途**: 代码开发、自动构建、Git 操作

## 快捷键

### 窗格管理

| 快捷键 | 功能 |
|--------|------|
| `Alt+h` | 移动焦点到左边窗格 |
| `Alt+j` | 移动焦点到下边窗格 |
| `Alt+k` | 移动焦点到上边窗格 |
| `Alt+l` | 移动焦点到右边窗格 |
| `Alt+s` | 垂直分割窗格 |
| `Alt+v` | 水平分割窗格 |
| `Alt+w` | 关闭当前窗格 |

### 标签页管理

| 快捷键 | 功能 |
|--------|------|
| `Alt+t` | 新建标签页 |
| `Alt+x` | 关闭当前标签页 |
| `Alt+1-5` | 切换到第 N 个标签页 |
| `Alt+h` | 上一个标签页 |
| `Alt+l` | 下一个标签页 |

### quantix 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Alt+m` | 打开 quantix 菜单 |
| `Alt+q` | 显示 quantix 状态 |
| `Alt+r` | 显示任务列表 |

### 通用

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+q` | 退出 Zellij |
| `Ctrl+p` | 进入命令模式 |

## 命令行操作

### 会话管理

```bash
# 列出所有会话
zellij list-sessions

# 连接到现有会话
zellij attach quantix

# 杀死会话
zellij kill-session quantix

# 杀死所有会话
zellij kill-all-sessions
```

### 布局操作

```bash
# 使用指定布局启动
zellij --layout-path config/zellij/layouts/main.kdl

# 使用会话名称启动
zellij --session quantix --layout-path config/zellij/layouts/main.kdl
```

## 状态栏

状态栏显示以下信息：

```
CPU: 45% | MEM: 2.1GB | DB: ✅ | Tasks: 3 | 14:30:00
```

### 颜色编码

- **CPU**: 绿色 (<50%), 黄色 (50-80%), 红色 (>80%)
- **DB**: ✅ (连接), ❌ (断开)

### 自定义状态栏

```bash
# 获取 JSON 格式状态
./scripts/zellij/status-collector.sh json

# 获取文本格式状态
./scripts/zellij/status-collector.sh text
```

## 高级功能

### 会话持久化

Zellij 会自动保存会话状态，断开 SSH 后可以恢复：

```bash
# 分离会话 (不退出)
# 按 Ctrl+p 然后按 d

# 重新连接
zellij attach quantix
```

### 插件扩展

Zellij 支持 WASM 插件：

```bash
# 安装插件
zellij plugin --install <plugin-url>

# 运行插件
zellij plugin <plugin-name>
```

### 自定义配置

编辑 `config/zellij/config.kdl`：

```kdl
// 修改主题
theme "catppuccin-mocha"

// 修改历史记录大小
scroll_history 50000

// 禁用鼠标
mouse_mode false
```

## 常见问题

### Q: 如何退出 Zellij？

A: 按 `Ctrl+q` 退出，或按 `Ctrl+p` 然后输入 `quit`。

### Q: 如何在窗格之间复制文本？

A: 使用鼠标选择文本（如果启用鼠标模式），或进入复制模式（`Ctrl+p` 然后 `[`）。

### Q: 会话丢失了怎么办？

A: Zellij 会话默认存储在 `/tmp`，重启后会丢失。可以使用 `zellij setup --dump-config > ~/.config/zellij/config.kdl` 保存配置。

### Q: 如何更改窗格大小？

A: 按 `Ctrl+p` 进入命令模式，然后使用 `Alt+h/j/k/l` 调整大小。

## 与 quantix CLI 集成

### 自动启动服务

在布局文件中配置自动启动命令：

```kdl
pane {
    command "quantix"
    args "task" "start" "--all"
}
```

### 实时监控

使用 `watch` 命令定期刷新：

```kdl
pane {
    command "watch"
    args "-n" "5" "quantix" "status" "--json"
}
```

### 日志查看

使用 `--follow` 参数实时查看日志：

```kdl
pane {
    command "quantix"
    args "logs" "--follow"
}
```

## 最佳实践

### 1. 使用命名会话

```bash
# 好的做法
./scripts/zellij/start-session.sh quantix-main main
./scripts/zellij/start-session.sh quantix-monitor monitor

# 避免
zellij  # 随机会话名
```

### 2. 定期清理会话

```bash
# 查看所有会话
zellij list-sessions

# 清理不需要的会话
zellij kill-session <session-name>
```

### 3. 使用配置文件

将常用配置保存在 `config/zellij/config.kdl`，避免每次手动设置。

### 4. 利用自动布局

使用预设布局可以快速切换工作场景。

## 参考资源

- [Zellij 官方文档](https://zellij.dev/documentation/)
- [Zellij GitHub](https://github.com/zellij-org/zellij)
- [KDL 配置格式](https://kdl.dev/)
- [Catppuccin 主题](https://github.com/catppuccin/zellij)

---

**最后更新**: 2026-03-09
**作者**: MyStocks Team
