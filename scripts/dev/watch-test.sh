#!/bin/bash
# 测试监控脚本
# 专注于运行测试并显示结果

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo "🧪 启动测试监控..."
echo ""
echo "按 Ctrl+C 停止"
echo ""

# 检查 cargo-watch
if ! command -v cargo-watch &> /dev/null; then
    echo "❌ cargo-watch 未安装"
    echo "正在安装..."
    cargo install cargo-watch
fi

# 启动测试监控
cargo watch \
    -x 'test --all-features' \
    --delay 0.5 \
    --clear
