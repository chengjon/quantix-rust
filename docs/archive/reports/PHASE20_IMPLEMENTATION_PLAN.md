# Phase 20: Zellij 集成和 CLI 增强 - 实施计划

**制定日期**: 2026-03-09
**状态**: 📋 规划中

> 状态源说明：本文是历史实施计划，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../FUNCTION_TREE.md) 的状态注册表行为准。

## 为什么选择 Zellij？

- **Rust 原生** - 与项目技术栈一致
- **现代 UI** - 更好的用户体验
- **插件系统** - 可扩展性强
- **KDL 配置** - 人类可读的配置格式
- **持久会话** - 支持 SSH 断开后恢复

## 目标

为 quantix-rust CLI 工具提供 Zellij 增强的运行环境：
1. ✅ **持久化会话** - CLI 服务后台运行，断开 SSH 不中断
2. ✅ **多窗格布局** - 同时监控多个 CLI 界面
3. ✅ **状态栏监控** - Zellij 状态栏实时显示系统状态
4. ✅ **预设布局** - 一键启动常用工作区
5. ✅ **插件集成** - 自定义状态栏插件

## 当前状态分析

### 已有基础设施

#### 现有脚本 (`scripts/runtime/`)
- `services.sh` - 服务管理脚本
- `install-services.sh` - 服务安装脚本

#### 现有功能
- CLI 命令处理器 (`src/cli/handlers.rs`)
- 任务调度器 (`src/tasks/scheduler.rs`)
- 实时监控 (`src/monitoring/`)

### 需要新增的功能

#### TMUX 会话管理
- ❌ TMUX 配置文件
- ❌ 会话启动脚本
- ❌ 预设布局配置

#### 状态监控
- ❌ TMUX 状态栏配置
- ❌ 实时数据采集脚本
- ❌ 状态栏格式化输出

#### 服务管理
- ❌ TMUX 服务启动/停止脚本
- ❌ 自动重启机制
- ❌ 日志聚合

## 实施计划

### 第 1 部分：TMUX 基础配置（第 1 周）

#### 任务 1.1：TMUX 配置文件
**文件**: `config/tmux/.tmux.conf`

**内容**:
```bash
# 基础设置
set -g default-terminal "screen-256color"
set -g history-limit 50000
set -g mouse on

# 快捷键
set -g prefix C-a
bind C-a send-prefix

# 窗口和面板
set -g base-index 1
setw -g pane-base-index 1
set -g renumber-windows on

# 状态栏
set -g status-position bottom
set -g status-justify left
set -g status-style 'bg=colour235 fg=colour136'
```

**验收标准**:
- ✅ 支持 256 色
- ✅ 鼠标操作
- ✅ 合理的快捷键

#### 任务 1.2：会话启动脚本
**文件**: `scripts/tmux/start-session.sh`

**功能**:
```bash
# 启动 quantix 主会话
./scripts/tmux/start-session.sh --session main

# 启动回测会话
./scripts/tmux/start-session.sh --session backtest

# 启动监控会话
./scripts/tmux/start-session.sh --session monitor
```

**验收标准**:
- ✅ 检测 TMUX 是否安装
- ✅ 创建命名会话
- ✅ 应用预设布局

#### 任务 1.3：预设布局配置
**文件**: `config/tmux/layouts/`

**布局类型**:
```
layouts/
├── main.layout        # 主工作区 (3面板)
├── monitor.layout     # 监控工作区 (4面板)
├── backtest.layout    # 回测工作区 (2面板)
└── dev.layout         # 开发工作区 (4面板)
```

**主工作区布局**:
```
┌─────────────────────────────────────┐
│          CLI 主界面                  │
├─────────────────┬───────────────────┤
│    实时行情      │     系统日志       │
└─────────────────┴───────────────────┘
```

### 第 2 部分：状态监控集成（第 1-2 周）

#### 任务 2.1：状态栏配置
**文件**: `config/tmux/statusline.conf`

**状态栏内容**:
```
[quantix] | CPU: 45% | MEM: 2.1GB | DB: ✅ | Tasks: 3 | 2026-03-09 14:30
```

**组件**:
- 项目名称
- CPU/内存使用率
- 数据库状态
- 任务数量
- 当前时间

**验收标准**:
- ✅ 实时更新（5秒间隔）
- ✅ 颜色编码（正常=绿色，警告=黄色，错误=红色）
- ✅ 可点击交互

#### 任务 2.2：状态采集脚本
**文件**: `scripts/tmux/status-collector.sh`

**采集数据**:
```bash
# CPU 使用率
cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}')

# 内存使用
mem_usage=$(free -m | awk 'NR==2{printf "%.1fGB", $3/1024}')

# 数据库状态
db_status=$(pg_isready -q && echo "✅" || echo "❌")

# 任务数量
task_count=$(quantix task status --count)
```

**验收标准**:
- ✅ 轻量级（<10ms 执行时间）
- ✅ 错误容忍（不会阻塞状态栏）
- ✅ 缓存机制（避免频繁查询）

#### 任务 2.3：状态栏格式化
**文件**: `scripts/tmux/status-formatter.sh`

**输出格式**:
```bash
# JSON 格式（供 TMUX 解析）
{
  "cpu": {"value": 45, "status": "ok"},
  "memory": {"value": 2.1, "unit": "GB", "status": "ok"},
  "database": {"status": "connected"},
  "tasks": {"active": 3, "total": 5}
}
```

**验收标准**:
- ✅ JSON 输出
- ✅ 状态码标准化
- ✅ 单位统一

### 第 3 部分：服务管理增强（第 2 周）

#### 任务 3.1：TMUX 服务管理脚本
**文件**: `scripts/tmux/service-manager.sh`

**功能**:
```bash
# 启动所有服务
./scripts/tmux/service-manager.sh start-all

# 停止所有服务
./scripts/tmux/service-manager.sh stop-all

# 重启单个服务
./scripts/tmux/service-manager.sh restart quantix

# 查看服务状态
./scripts/tmux/service-manager.sh status
```

**服务列表**:
- quantix-main - 主 CLI 服务
- quantix-monitor - 监控服务
- quantix-scheduler - 任务调度器
- quantix-data - 数据采集服务

**验收标准**:
- ✅ 服务状态跟踪
- ✅ 优雅停止（SIGTERM）
- ✅ 依赖管理（先启动数据库）

#### 任务 3.2：自动重启机制
**文件**: `scripts/tmux/watchdog.sh`

**功能**:
```bash
#!/bin/bash
# 监控服务进程，崩溃后自动重启

while true; do
    if ! tmux has-session -t quantix-main 2>/dev/null; then
        echo "[$(date)] quantix-main 崩溃，正在重启..." >> /var/log/quantix/watchdog.log
        ./scripts/tmux/service-manager.sh restart quantix-main
    fi
    sleep 30
done
```

**验收标准**:
- ✅ 进程存活检测
- ✅ 崩溃日志记录
- ✅ 重启次数限制（避免无限重启）

#### 任务 3.3：日志聚合
**文件**: `scripts/tmux/log-aggregator.sh`

**功能**:
```bash
# 聚合所有服务日志到统一位置
./scripts/tmux/log-aggregator.sh --output /var/log/quantix/combined.log

# 实时查看所有日志
./scripts/tmux/log-aggregator.sh --follow
```

**日志格式**:
```
[2026-03-09 14:30:15] [quantix-main] INFO: 启动成功
[2026-03-09 14:30:16] [quantix-monitor] INFO: 监控服务已启动
[2026-03-09 14:30:20] [quantix-main] ERROR: 数据库连接失败
```

**验收标准**:
- ✅ 统一日志格式
- ✅ 日志轮转（按天）
- ✅ 压缩旧日志

### 第 4 部分：用户体验优化（第 2-3 周）

#### 任务 4.1：交互式菜单
**文件**: `scripts/tmux/menu.sh`

**菜单选项**:
```
╔══════════════════════════════════════╗
║       Quantix TMUX 管理菜单          ║
╠══════════════════════════════════════╣
║ 1. 启动主工作区                       ║
║ 2. 启动监控工作区                     ║
║ 3. 启动回测工作区                     ║
║ 4. 查看服务状态                       ║
║ 5. 查看日志                          ║
║ 6. 重启所有服务                       ║
║ 7. 停止所有服务                       ║
║ 8. 退出                              ║
╚══════════════════════════════════════╝
```

**验收标准**:
- ✅ 键盘导航
- ✅ 颜色高亮
- ✅ 帮助提示

#### 任务 4.2：快捷键绑定
**文件**: `config/tmux/keybindings.conf`

**快捷键**:
```bash
# quantix 专用快捷键
bind Q run-shell "quantix status"
bind M run-shell "quantix menu"
bind R run-shell "quantix restart"
bind L run-shell "quantix logs"
bind S run-shell "quantix services"
```

**验收标准**:
- ✅ 与现有快捷键不冲突
- ✅ 可自定义
- ✅ 帮助文档

#### 任务 4.3：通知集成
**文件**: `scripts/tmux/notifier.sh`

**通知场景**:
- 服务崩溃
- 任务完成
- 告警触发
- 数据同步完成

**通知方式**:
- TMUX 消息（status-left）
- 系统通知（notify-send）
- 声音提示（可选）

**验收标准**:
- ✅ 非阻塞通知
- ✅ 通知历史
- ✅ 静默模式

## 文件结构

```
quantix-rust/
├── config/
│   └── tmux/
│       ├── .tmux.conf           # TMUX 主配置
│       ├── statusline.conf      # 状态栏配置
│       ├── keybindings.conf     # 快捷键绑定
│       └── layouts/
│           ├── main.layout      # 主工作区
│           ├── monitor.layout   # 监控工作区
│           ├── backtest.layout  # 回测工作区
│           └── dev.layout       # 开发工作区
├── scripts/
│   └── tmux/
│       ├── start-session.sh     # 会话启动
│       ├── service-manager.sh   # 服务管理
│       ├── status-collector.sh  # 状态采集
│       ├── status-formatter.sh  # 状态格式化
│       ├── watchdog.sh          # 自动重启
│       ├── log-aggregator.sh    # 日志聚合
│       ├── menu.sh              # 交互菜单
│       ├── notifier.sh          # 通知系统
│       └── install.sh           # 一键安装
└── docs/
    └── guides/
        └── TMUX_GUIDE.md        # TMUX 使用指南
```

## 使用示例

### 启动主工作区

```bash
# 方式 1: 使用启动脚本
./scripts/tmux/start-session.sh --session main

# 方式 2: 使用交互菜单
./scripts/tmux/menu.sh

# 方式 3: TMUX 命令
tmux new -s quantix -f config/tmux/.tmux.conf
```

### 服务管理

```bash
# 启动所有服务
./scripts/tmux/service-manager.sh start-all

# 查看状态
./scripts/tmux/service-manager.sh status

# 重启单个服务
./scripts/tmux/service-manager.sh restart quantix-main
```

### 监控和日志

```bash
# 实时查看日志
./scripts/tmux/log-aggregator.sh --follow

# 查看最近错误
./scripts/tmux/log-aggregator.sh --level ERROR --tail 50
```

## 测试计划

### 单元测试
- ✅ 状态采集脚本
- ✅ 状态格式化脚本
- ✅ 服务管理脚本

### 集成测试
- ✅ 会话启动流程
- ✅ 服务启动/停止
- ✅ 日志聚合

### 端到端测试
- ✅ 完整工作流
- ✅ 崩溃恢复
- ✅ 通知触发

## 成功标准

### 第 1 周（TMUX 基础）
- [x] TMUX 配置文件创建
- [x] 会话启动脚本完成
- [x] 预设布局配置完成

### 第 2 周（状态监控）
- [x] 状态栏配置完成
- [x] 状态采集脚本完成
- [x] 服务管理脚本完成

### 第 3 周（用户体验）
- [x] 交互菜单完成
- [x] 快捷键绑定完成
- [x] 通知集成完成
- [x] 完整文档

## 风险和缓解措施

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| TMUX 版本兼容性 | 功能不可用 | 低 | 版本检测，兼容模式 |
| 脚本执行权限 | 启动失败 | 中 | 自动 chmod +x |
| 资源占用过高 | 性能影响 | 低 | 轻量级采集，缓存机制 |
| 日志文件过大 | 磁盘占满 | 中 | 日志轮转和压缩 |

## 后续优化

### 短期（1-2周）
1. 添加更多预设布局
2. 集成告警系统
3. 优化状态栏性能

### 中期（1个月）
1. Web 状态面板（可选）
2. 远程管理接口
3. 配置热重载

### 长期（2-3个月）
1. 多节点支持
2. 集群管理
3. 自定义插件系统

---

**状态**: 📋 规划完成，等待实施
**下一步**: 开始第 1 部分 - TMUX 基础配置
**负责人**: MyStocks Team
**最后更新**: 2026-03-09
