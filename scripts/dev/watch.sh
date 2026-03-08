#!/bin/bash
# 自动重载开发脚本
# 监控文件变化并自动重新编译、运行测试

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo "🔍 启动开发监控..."
echo ""
echo "监控目录: src/"
echo "自动操作:"
echo "  - 重新编译"
echo "  - 运行测试"
echo "  - 运行 clippy"
echo ""
echo "按 Ctrl+C 停止"
echo ""

# 检查 cargo-watch
if ! command -v cargo-watch &> /dev/null; then
    echo "❌ cargo-watch 未安装"
    echo "正在安装..."
    cargo install cargo-watch
fi

# 启动监控
cargo watch \
    -w 'src/' \
    -x 'build' \
    -x 'test --lib --all-features' \
    -x 'clippy --all-targets -- -D warnings' \
    --ignore-nothing \
    --delay 0.5 \
    --clear
