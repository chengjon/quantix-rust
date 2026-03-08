#!/bin/bash
# quantix-rust 开发环境启动脚本
# 使用 Zellij 启动多窗口开发环境

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SESSION_NAME="quantix-dev"

echo "🚀 启动 quantix-rust 开发环境..."

# 检查 Zellij 是否安装
if ! command -v zellij &> /dev/null; then
    echo "❌ Zellij 未安装"
    echo ""
    echo "安装方法："
    echo "  cargo install zellij --locked"
    echo "  或"
    echo "  sudo apt install zellij  # Ubuntu/Debian"
    exit 1
fi

# 检查配置文件
CONFIG_FILE="$PROJECT_ROOT/config/zellij.kdl"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ 配置文件不存在: $CONFIG_FILE"
    exit 1
fi

# 尝试附加到现有会话
if zellij list-sessions 2>/dev/null | grep -q "^$SESSION_NAME"; then
    echo "📱 附加到现有会话: $SESSION_NAME"
    zellij attach "$SESSION_NAME"
else
    echo "✨ 创建新会话: $SESSION_NAME"
    zellij --layout "$CONFIG_FILE" --session-name "$SESSION_NAME"
fi
