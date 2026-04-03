#!/bin/bash
# quantix-rust Zellij 一键安装脚本
# 用法: ./scripts/zellij/install.sh

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# 检测操作系统
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        *)          echo "unknown";;
    esac
}

# 检测包管理器
detect_package_manager() {
    if command -v apt-get &> /dev/null; then
        echo "apt"
    elif command -v pacman &> /dev/null; then
        echo "pacman"
    elif command -v dnf &> /dev/null; then
        echo "dnf"
    elif command -v brew &> /dev/null; then
        echo "brew"
    else
        echo "unknown"
    fi
}

# 检查 Rust 和 Cargo
check_rust() {
    if command -v cargo &> /dev/null; then
        log_success "Rust/Cargo 已安装: $(cargo --version)"
        return 0
    else
        log_warn "Rust/Cargo 未安装"
        return 1
    fi
}

# 安装 Rust
install_rust() {
    log_info "正在安装 Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    log_success "Rust 安装完成"
}

# 使用 cargo 安装 Zellij
install_zellij_cargo() {
    log_info "正在使用 cargo 安装 Zellij..."
    cargo install zellij
    log_success "Zellij 安装完成: $(zellij --version)"
}

# 使用包管理器安装 Zellij
install_zellij_package() {
    local pm=$(detect_package_manager)

    case $pm in
        apt)
            log_info "正在使用 apt 安装 Zellij..."
            sudo apt update
            sudo apt install -y zellij
            ;;
        pacman)
            log_info "正在使用 pacman 安装 Zellij..."
            sudo pacman -S --noconfirm zellij
            ;;
        dnf)
            log_info "正在使用 dnf 安装 Zellij..."
            sudo dnf install -y zellij
            ;;
        brew)
            log_info "正在使用 brew 安装 Zellij..."
            brew install zellij
            ;;
        *)
            log_warn "未检测到支持的包管理器，使用 cargo 安装"
            install_zellij_cargo
            return
            ;;
    esac

    log_success "Zellij 安装完成: $(zellij --version)"
}

# 创建配置链接
setup_config() {
    log_info "设置 Zellij 配置..."

    local config_dir="$HOME/.config/zellij"

    # 创建配置目录
    mkdir -p "$config_dir"

    # 备份现有配置
    if [[ -f "$config_dir/config.kdl" ]]; then
        log_warn "备份现有配置到 $config_dir/config.kdl.bak"
        mv "$config_dir/config.kdl" "$config_dir/config.kdl.bak"
    fi

    # 创建符号链接
    ln -sf "$PROJECT_ROOT/config/zellij/config.kdl" "$config_dir/config.kdl"
    ln -sf "$PROJECT_ROOT/config/zellij/layouts" "$config_dir/layouts"

    log_success "配置链接创建完成"
}

# 设置脚本权限
setup_scripts() {
    log_info "设置脚本执行权限..."

    chmod +x "$PROJECT_ROOT/scripts/zellij/"*.sh

    log_success "脚本权限设置完成"
}

# 创建别名
setup_aliases() {
    log_info "创建 shell 别名..."

    local shell_rc=""
    local start_session_script="$PROJECT_ROOT/scripts/zellij/start-session.sh"
    if [[ -f "$HOME/.bashrc" ]]; then
        shell_rc="$HOME/.bashrc"
    elif [[ -f "$HOME/.zshrc" ]]; then
        shell_rc="$HOME/.zshrc"
    fi

    if [[ -n "$shell_rc" ]]; then
        cat >> "$shell_rc" << EOF

# quantix-rust Zellij aliases
alias qz="$start_session_script"
alias qz-main="$start_session_script quantix main"
alias qz-monitor="$start_session_script monitor monitor"
alias qz-backtest="$start_session_script backtest backtest"
alias qz-dev="$start_session_script dev dev"
EOF
        log_success "别名已添加到 $shell_rc"
        log_info "请运行 'source $shell_rc' 使别名生效"
    else
        log_warn "未找到 shell 配置文件，跳过别名创建"
    fi
}

# 验证安装
verify_installation() {
    log_info "验证安装..."

    echo ""
    echo "安装状态:"
    echo "  - Zellij: $(command -v zellij &> /dev/null && echo "✅ $(zellij --version)" || echo "❌ 未安装")"
    echo "  - 配置文件: $([[ -f $HOME/.config/zellij/config.kdl ]] && echo "✅" || echo "❌")"
    echo "  - 布局文件: $([[ -d $HOME/.config/zellij/layouts ]] && echo "✅" || echo "❌")"
    echo ""
}

# 显示使用说明
show_usage() {
    echo ""
    echo "╔══════════════════════════════════════╗"
    echo "║       Zellij 安装完成！              ║"
    echo "╠══════════════════════════════════════╣"
    echo "║                                      ║"
    echo "║  快速启动:                           ║"
    echo "║    qz              # 默认工作区      ║"
    echo "║    qz-main         # 主工作区        ║"
    echo "║    qz-monitor      # 监控工作区      ║"
    echo "║    qz-backtest     # 回测工作区      ║"
    echo "║    qz-dev          # 开发工作区      ║"
    echo "║                                      ║"
    echo "║  或使用完整命令:                     ║"
    echo "║    ./scripts/zellij/start-session.sh ║"
    echo "║                                      ║"
    echo "║  常用快捷键:                         ║"
    echo "║    Alt+h/j/k/l   移动焦点            ║"
    echo "║    Alt+s         垂直分割            ║"
    echo "║    Alt+v         水平分割            ║"
    echo "║    Ctrl+q        退出                ║"
    echo "║                                      ║"
    echo "╚══════════════════════════════════════╝"
    echo ""
}

# 主函数
main() {
    echo ""
    echo "╔══════════════════════════════════════╗"
    echo "║   quantix-rust Zellij 安装脚本       ║"
    echo "╚══════════════════════════════════════╝"
    echo ""

    # 检查并安装 Zellij
    if ! command -v zellij &> /dev/null; then
        log_info "Zellij 未安装，正在安装..."

        # 优先使用 cargo
        if check_rust; then
            install_zellij_cargo
        else
            read -p "是否安装 Rust? [y/N]: " install_rust_choice
            if [[ "$install_rust_choice" =~ ^[Yy]$ ]]; then
                install_rust
                install_zellij_cargo
            else
                install_zellij_package
            fi
        fi
    else
        log_success "Zellij 已安装: $(zellij --version)"
    fi

    # 设置配置
    setup_config
    setup_scripts
    setup_aliases

    # 验证
    verify_installation

    # 显示使用说明
    show_usage
}

# 运行主函数
main
