#!/bin/bash
# quantix-rust Zellij 会话启动脚本
# 用法: ./scripts/zellij/start-session.sh [会话名] [布局名]

set -euo pipefail

# 默认值
SESSION_NAME="${1:-quantix}"
LAYOUT_NAME="${2:-main}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LAYOUT_PATH="$PROJECT_ROOT/config/zellij/layouts/${LAYOUT_NAME}.kdl"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查 Zellij 是否安装
check_zellij() {
    if ! command -v zellij &> /dev/null; then
        log_error "Zellij 未安装"
        echo ""
        echo "安装方法:"
        echo "  cargo install zellij"
        echo ""
        echo "或使用包管理器:"
        echo "  # macOS"
        echo "  brew install zellij"
        echo ""
        echo "  # Linux"
        echo "  # Arch Linux"
        echo "  sudo pacman -S zellij"
        echo ""
        echo "  # Ubuntu/Debian"
        echo "  sudo apt install zellij"
        exit 1
    fi
    log_success "Zellij 已安装: $(zellij --version)"
}

# 检查布局文件是否存在
check_layout() {
    if [[ ! -f "$LAYOUT_PATH" ]]; then
        log_error "布局文件不存在: $LAYOUT_PATH"
        echo ""
        echo "可用布局:"
        ls -1 "$PROJECT_ROOT/config/zellij/layouts/"*.kdl 2>/dev/null | xargs -n1 basename | sed 's/\.kdl$//' || echo "  (无)"
        exit 1
    fi
    log_success "布局文件: $LAYOUT_PATH"
}

# 检查会话是否已存在
check_session() {
    if zellij list-sessions 2>/dev/null | grep -q "^$SESSION_NAME"; then
        return 0
    fi
    return 1
}

# 启动新会话
start_new_session() {
    log_info "创建新会话 '$SESSION_NAME' 使用布局 '$LAYOUT_NAME'..."

    cd "$PROJECT_ROOT"

    # 设置环境变量
    export ZELLIJ_CONFIG_DIR="$PROJECT_ROOT/config/zellij"

    # 启动 Zellij
    zellij --session "$SESSION_NAME" --layout-path "$LAYOUT_PATH"
}

# 连接到现有会话
attach_session() {
    log_warn "会话 '$SESSION_NAME' 已存在"
    echo ""
    echo "选项:"
    echo "  1. 连接到现有会话"
    echo "  2. 杀死现有会话并重新启动"
    echo "  3. 退出"
    echo ""
    read -p "请选择 [1-3]: " choice

    case $choice in
        1)
            log_info "连接到现有会话..."
            zellij attach "$SESSION_NAME"
            ;;
        2)
            log_info "杀死现有会话..."
            zellij kill-session "$SESSION_NAME" 2>/dev/null || true
            sleep 1
            start_new_session
            ;;
        3)
            log_info "退出"
            exit 0
            ;;
        *)
            log_error "无效选择"
            exit 1
            ;;
    esac
}

# 主函数
main() {
    echo ""
    echo "╔══════════════════════════════════════╗"
    echo "║    quantix-rust Zellij 会话管理      ║"
    echo "╠══════════════════════════════════════╣"
    echo "║  会话: $SESSION_NAME"
    echo "║  布局: $LAYOUT_NAME"
    echo "╚══════════════════════════════════════╝"
    echo ""

    check_zellij
    check_layout

    if check_session; then
        attach_session
    else
        start_new_session
    fi
}

# 帮助信息
show_help() {
    echo "quantix-rust Zellij 会话启动脚本"
    echo ""
    echo "用法: $0 [会话名] [布局名]"
    echo ""
    echo "参数:"
    echo "  会话名    Zellij 会话名称 (默认: quantix)"
    echo "  布局名    布局文件名称 (默认: main)"
    echo ""
    echo "示例:"
    echo "  $0                      # 使用默认设置启动"
    echo "  $0 quantix main         # 启动主工作区"
    echo "  $0 monitor monitor      # 启动监控工作区"
    echo "  $0 backtest backtest    # 启动回测工作区"
    echo "  $0 dev dev              # 启动开发工作区"
    echo ""
    echo "可用布局:"
    ls -1 "$PROJECT_ROOT/config/zellij/layouts/"*.kdl 2>/dev/null | xargs -n1 basename | sed 's/\.kdl$//' || echo "  (无)"
    echo ""
    echo "快捷键 (在 Zellij 中):"
    echo "  Alt+h/j/k/l   移动焦点"
    echo "  Alt+s         垂直分割"
    echo "  Alt+v         水平分割"
    echo "  Alt+w         关闭窗格"
    echo "  Alt+t         新建标签页"
    echo "  Alt+1-5       切换标签页"
    echo "  Alt+m         quantix 菜单"
    echo "  Alt+q         quantix 状态"
    echo "  Ctrl+q        退出 Zellij"
}

# 检查参数
if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
    show_help
    exit 0
fi

main
