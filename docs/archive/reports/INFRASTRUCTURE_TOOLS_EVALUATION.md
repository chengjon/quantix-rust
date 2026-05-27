# quantix-rust 基础设施工具评估报告

## 📋 评估时间
2026-03-07

## 🎯 项目需求分析

### quantix-rust 项目特点
- **Rust CLI 量化交易工具**
- 长时间运行任务（回测、数据同步、任务调度）
- 需要监控多个进程（数据采集、策略运行）
- 实时日志查看需求
- 进程守护和自动重启需求
- 开发调试便利性

---

## 1. 终端复用工具评估

### 需求场景
- 同时监控多个进程输出
- 开发时查看多个终端窗口
- 生产环境监控多个任务状态

### 方案对比

| 工具 | 语言 | 优势 | 劣势 | 推荐度 |
|------|------|------|------|--------|
| **Zellij** | Rust | ✅ Rust 原生<br>✅ 现代化 UI<br>✅ 布局系统<br>✅ 插件系统<br>✅ 配置简单（KDL） | ⚠️ 相对较新<br>⚠️ 生态不如 tmux | ⭐⭐⭐⭐⭐ |
| **tmux** | C | ✅ 成熟稳定<br>✅ 功能强大<br>✅ 广泛使用 | ❌ 配置复杂<br>❌ 学习曲线陡<br>❌ 默认布局简陋 | ⭐⭐⭐ |
| **screen** | C | ✅ 几乎所有 Unix 系统预装 | ❌ 功能过时<br>❌ 无现代特性 | ⭐⭐ |
| **abducat** | Rust | ✅ 交互式终端 | ❌ 不是复用工具<br>❌ 功能单一 | ⭐ |

### ✅ 推荐：Zellij

**理由**：
1. **Rust 原生** - 与项目技术栈一致
2. **现代化** - 更好的用户体验
3. **易用性** - 配置简单，学习曲线平缓
4. **功能强大** - 支持布局、插件、会话管理

**关键特性**：
```bash
# 1. 会话管理
zellij attach session-name    # 附加到会话
zellij list-sessions          # 列出所有会话
zellij kill-session session   # 关闭会话

# 2. 布局系统
# 支持自定义布局（TUI 开发、监控、数据分析）
zellij layout layout.kdl

# 3. 标签页
Ctrl + t    # 新建标签页
Ctrl + q    # 关闭标签页

# 4. 窗格管理
Ctrl + p    # 垂直分割
Ctrl + w    # 水平分割
Ctrl + h/j/k/l  # 切换窗格
```

---

## 2. 进程管理工具评估

### 需求场景
- **开发阶段**：快速重启、自动重载
- **生产环境**：进程守护、自动重启、日志管理
- **多进程管理**：数据采集、策略运行、任务调度

### 方案对比

| 工具 | 语言 | 类型 | 优势 | 劣势 | 推荐度 |
|------|------|------|------|------|--------|
| **systemd** | C | 系统服务 | ✅ Linux 标准<br>✅ 功能最强大<br>✅ 稳定可靠<br>✅ 开箱即用 | ⚠️ 学习曲线 | ⭐⭐⭐⭐⭐ |
| **cargo-watch** | Rust | 开发工具 | ✅ Rust 生态<br>✅ 简单易用<br>✅ 自动重载 | ❌ 仅开发阶段 | ⭐⭐⭐⭐ |
| **smoke** | Rust | 进程监控 | ✅ Rust 原生<br>✅ 简单轻量 | ⚠️ 功能单一<br>⚠️ 不适合生产 | ⭐⭐⭐ |
| **supervisord-rs** | Rust | 进程管理 | ❌ 非官方<br>❌ 功能不完整 | ⚠️ 缺乏维护 | ⭐ |
| **pm2-rs** | Rust | PM2 移植 | ✅ PM2 兼容 | ⚠️ 功能不完整<br>⚠️ 活跃度低 | ⭐⭐ |

### ❌ 不推荐：supervisord-rs / pm2-rs

**supervisord-rs**:
- ❌ 不是官方项目
- ❌ 功能不完整（缺少很多核心特性）
- ❌ 最后更新时间久远
- ❌ 社区活跃度低

**pm2-rs**:
- ⚠️ 仅实现部分 PM2 功能
- ⚠️ 缺少关键特性（集群模式、日志管理）
- ⚠️ 不如直接使用 PM2（Node.js 版本）
- ⚠️ 社区支持有限

### ✅ 推荐：混合方案

#### 开发阶段：cargo-watch

```bash
# 安装
cargo install cargo-watch

# 使用
cargo watch -x run -- -x test -x clippy

# 功能：
# - 文件变化时自动重新编译
# - 自动运行测试
# - 自动运行 clippy
# - 适合快速迭代开发
```

#### 生产环境：systemd

**优势**：
- ✅ Linux 标准，无需额外安装
- ✅ 功能最强大（守护进程、自动重启、日志管理）
- ✅ 系统集成好（开机自启、资源限制）
- ✅ 成熟稳定，企业级可靠性

**配置示例**：
```ini
[Unit]
Description=Quantix Data Collector
After=network.target clickhouse.service

[Service]
Type=simple
User=quantix
WorkingDirectory=/opt/quantix-rust
ExecStart=/opt/quantix-rust/target/release/quantix task start
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

---

## 3. 最佳实践方案

### 🎯 推荐架构

```
┌─────────────────────────────────────────────┐
│           开发阶段 (Development)            │
├─────────────────────────────────────────────┤
│                                             │
│  ┌──────────────┐      ┌─────────────────┐ │
│  │   Zellij     │      │  cargo-watch    │ │
│  │              │      │                 │ │
│  │  多窗口监控   │      │  自动重载编译   │ │
│  │  - 终端1      │      │  - 快速反馈     │ │
│  │  - 终端2      │      │  - 开发效率     │ │
│  │  - 日志       │      │                 │ │
│  └──────────────┘      └─────────────────┘ │
│                                             │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│          生产阶段 (Production)              │
├─────────────────────────────────────────────┤
│                                             │
│  ┌─────────────────────────────────────┐   │
│  │           systemd                   │   │
│  │                                     │   │
│  │  quantix-data-collector.service     │   │
│  │  quantix-strategy-runner.service    │   │
│  │  quantix-task-scheduler.service     │   │
│  │  quantix-websocket-client.service   │   │
│  │                                     │   │
│  │  特性：                             │   │
│  │  - 进程守护                         │   │
│  │  - 自动重启                         │   │
│  │  - 日志管理（journald）             │   │
│  │  - 开机自启                         │   │
│  │  - 资源限制                         │   │
│  └─────────────────────────────────────┘   │
│                                             │
└─────────────────────────────────────────────┘
```

---

## 4. 实施方案

### Phase 1: Zellij 集成（开发阶段）

#### 4.1 安装 Zellij

```bash
# 方法1：Cargo 安装
cargo install zellij --locked

# 方法2：二进制下载
wget https://github.com/zellij-org/zellij/releases/latest/download/zellij-x86_64-unknown-linux-musl.tar.gz
tar -xzf zellij-x86_64-unknown-linux-musl.tar.gz
sudo mv zellij /usr/local/bin/

# 方法3：包管理器
sudo apt install zellij  # Ubuntu/Debian
brew install zellij      # macOS
```

#### 4.2 创建 Zellij 配置

**文件**: `config/zellij.kdl`

```kdl
// quantix-rust Zellij 配置

// 主题设置
theme "catppuccin-mocha"

// 默认布局
default_layout "quantix-dev" (cwd: "/opt/claude/quantix-rust") {
    tab name="Main" {
        pane size=1 split_direction="Vertical" {
            pane name="Editor" {
                command "nvim" { args "src/main.rs" }
            }
            pane name="Terminal" {
                command "zsh"
            }
        }
        pane size=2 split_direction="Horizontal" {
            pane name="Logs" {
                command "tail" { args "-f" "target/quantix.log" }
            }
        }
    }

    tab name="Testing" {
        pane name="Unit Tests" {
            command "cargo" { args "test" "--lib" }
        }
    }

    tab name="Monitoring" {
        pane name="System" {
            command "htop"
        }
    }
}

// 快捷键配置
keybinds {
    shared_except "normal" "locked" {
        bind "Ctrl p" { NewPane; SwitchToMode "Normal"; }
        bind "Ctrl w" { NewPane; SwitchToMode "Normal"; }
        bind "Ctrl h" { MoveFocusOrTab "Left"; }
        bind "Ctrl l" { MoveFocusOrTab "Right"; }
        bind "Ctrl k" { MoveFocus "Up"; }
        bind "Ctrl j" { MoveFocus "Down"; }
        bind "Ctrl q" { ClosePane; }
        bind "Ctrl t" { NewTab; SwitchToMode "Normal"; }
    }
}

// 插件配置
plugins {
    tab-bar {
        location "top"
    }
    status-bar {
        location "bottom"
    }
}
```

#### 4.3 开发脚本

**文件**: `scripts/dev.zsh`

```bash
#!/bin/bash
# quantix-rust 开发环境启动脚本

# 启动 Zellij 开发环境
zellij attach -l quantix-dev || zellij --layout config/zellij.kdl
```

**使用**:
```bash
# 启动开发环境
./scripts/dev.zsh

# 或直接使用
zellij --layout config/zellij.kdl
```

### Phase 2: cargo-watch 集成（开发阶段）

#### 2.1 安装

```bash
cargo install cargo-watch
```

#### 2.2 创建开发脚本

**文件**: `scripts/watch.sh`

```bash
#!/bin/bash
# 自动重载开发脚本

echo "🔍 启动开发监控..."

cargo watch \
  -w 'src/' \
  -x 'run -- menu' \
  -x 'test --lib' \
  -x 'clippy' \
  --ignore-nothing \
  --delay 0.5 \
  --clear
```

**文件**: `scripts/watch-test.sh`

```bash
#!/bin/bash
# 测试监控脚本

cargo watch -x 'test --all-features' --delay 0.5
```

### Phase 3: systemd 集成（生产阶段）

#### 3.1 创建 systemd 服务

**文件**: `config/systemd/quantix-data-collector.service`

```ini
[Unit]
Description=Quantix Data Collector Service
Documentation=https://github.com/chengjon/quantix-rust
After=network-online.target clickhouse.service postgresql.service
Wants=network-online.target

[Service]
Type=simple
User=quantix
Group=quantix
WorkingDirectory=/opt/quantix-rust

# 环境变量
Environment="RUST_LOG=info"
Environment="CLICKHOUSE_URL=http://localhost:8123"
Environment="CLICKHOUSE_DB=quantix"

# 主进程
ExecStart=/opt/quantix-rust/target/release/quantix task start

# 重启策略
Restart=always
RestartSec=10s
StartLimitInterval=60s
StartLimitBurst=3

# 资源限制
MemoryMax=2G
CPUQuota=200%

# 日志
StandardOutput=journal
StandardError=journal
SyslogIdentifier=quantix-collector

# 安全
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

**文件**: `config/systemd/quantix-strategy-runner.service`

```ini
[Unit]
Description=Quantix Strategy Runner Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=quantix
WorkingDirectory=/opt/quantix-rust
Environment="RUST_LOG=info"
ExecStart=/opt/quantix-rust/target/release/quantix strategy run --daemon
Restart=always
RestartSec=10s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

**文件**: `config/systemd/quantix-task-scheduler.service`

```ini
[Unit]
Description=Quantix Task Scheduler Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=quantix
WorkingDirectory=/opt/quantix-rust
Environment="RUST_LOG=info"
ExecStart=/opt/quantix-rust/target/release/quantix task start --daemon
Restart=always
RestartSec=10s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

#### 3.2 安装脚本

**文件**: `scripts/install-services.sh`

```bash
#!/bin/bash
# 安装 systemd 服务

set -e

PROJECT_ROOT="/opt/quantix-rust"
SERVICE_DIR="$PROJECT_ROOT/config/systemd"
SYSTEMD_DIR="/etc/systemd/system"

echo "📦 安装 quantix-rust systemd 服务..."

# 复制服务文件
sudo cp "$SERVICE_DIR"/*.service "$SYSTEMD_DIR/"

# 重载 systemd
sudo systemctl daemon-reload

# 启用服务（不启动）
sudo systemctl enable quantix-data-collector.service
sudo systemctl enable quantix-strategy-runner.service
sudo systemctl enable quantix-task-scheduler.service

echo "✅ 服务安装完成！"
echo ""
echo "使用命令管理服务："
echo "  启动: sudo systemctl start quantix-data-collector"
echo "  停止: sudo systemctl stop quantix-data-collector"
echo "  状态: sudo systemctl status quantix-data-collector"
echo "  日志: sudo journalctl -u quantix-data-collector -f"
```

#### 3.3 管理脚本

**文件**: `scripts/services.sh`

```bash
#!/bin/bash
# systemd 服务管理脚本

ACTION=$1
SERVICE=$2

case "$ACTION" in
    start)
        sudo systemctl start "quantix-$SERVICE"
        ;;
    stop)
        sudo systemctl stop "quantix-$SERVICE"
        ;;
    restart)
        sudo systemctl restart "quantix-$SERVICE"
        ;;
    status)
        sudo systemctl status "quantix-$SERVICE"
        ;;
    logs)
        sudo journalctl -u "quantix-$SERVICE" -f
        ;;
    enable)
        sudo systemctl enable "quantix-$SERVICE"
        ;;
    disable)
        sudo systemctl disable "quantix-$SERVICE"
        ;;
    *)
        echo "用法: $0 {start|stop|restart|status|logs|enable|disable} <service>"
        echo ""
        echo "可用服务："
        echo "  - data-collector"
        echo "  - strategy-runner"
        echo "  - task-scheduler"
        exit 1
        ;;
esac
```

---

## 5. 集成到 Cargo.toml

### 添加开发依赖

```toml
[dev-dependencies]
# ... 其他依赖

# 开发工具
cargo-watch = "8.5"

[package.metadata.deb]
systemd-units = { unit-name = "quantix", unit-scripts = "config/systemd/" }
```

---

## 6. 使用文档

### 开发阶段

```bash
# 1. 启动 Zellij 开发环境
./scripts/dev.zsh

# 2. 启动自动监控
./scripts/watch.sh

# 3. 监控测试
./scripts/watch-test.sh
```

### 生产环境

```bash
# 1. 安装服务
./scripts/install-services.sh

# 2. 启动所有服务
./scripts/services.sh start data-collector
./scripts/services.sh start strategy-runner
./scripts/services.sh start task-scheduler

# 3. 查看状态
./scripts/services.sh status data-collector

# 4. 查看日志
./scripts/services.sh logs data-collector

# 5. 重启服务
./scripts/services.sh restart data-collector
```

---

## 7. 监控和管理

### 创建监控仪表板

**文件**: `scripts/monitor.zsh`

```bash
#!/bin/bash
# 启动 Zellij 监控布局

zellij --layout - <<EOF
layout {
    tab name="Services" {
        pane split_direction="Vertical" {
            pane split_direction="Horizontal" {
                pane command "systemctl" { args "status" "quantix-data-collector.service" }
                pane command "systemctl" { args "status" "quantix-strategy-runner.service" }
            }
            pane command "journalctl" { args "-f" "-u" "quantix-task-scheduler.service" }
        }
    }
    tab name="Logs" {
        pane split_direction="Horizontal" {
            pane command "tail" { args "-f" "/var/log/quantix/collector.log" }
            pane command "tail" { args "-f" "/var/log/quantix/strategy.log" }
        }
    }
}
EOF
```

---

## 8. 总结

### ✅ 推荐方案

| 场景 | 工具 | 理由 |
|------|------|------|
| **终端复用** | **Zellij** | Rust 原生、现代化、易用 |
| **开发监控** | **cargo-watch** | 自动重载、提高开发效率 |
| **生产进程** | **systemd** | 企业级、稳定可靠、功能强大 |
| ❌ supervisord-rs | | 非官方、功能不完整 |
| ❌ pm2-rs | | 功能不全、社区支持有限 |

### 🎯 核心优势

1. **Rust 生态一致性** - Zellij + cargo-watch 都是 Rust 原生
2. **零学习成本** - 不需要学习其他语言（如 Python 的 PM2）
3. **企业级可靠性** - systemd 是 Linux 标准
4. **开发体验** - Zellij 现代化 UI，cargo-watch 自动化

### 📦 实施优先级

1. **Phase 1（立即）**: Zellij 配置 + 开发脚本
2. **Phase 2（本周）**: cargo-watch 集成
3. **Phase 3（下周）**: systemd 服务配置
4. **Phase 4（持续）**: 监控仪表板

---

**评估人**: Claude Code (Opus 4.6)
**评估日期**: 2026-03-07
**版本**: v1.0
