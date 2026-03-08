#!/bin/bash
# quantix-rust 性能测试脚本
#
# Phase 18: 性能基准测试与优化工具
#
# 使用方法:
#   ./scripts/dev/run_benchmarks.sh [选项]
#
# 选项:
#   --baseline    保存当前结果为基线
#   --compare     与基线对比
#   --flamegraph   生成火焰图
#   --dhat        堆分配分析
#   --html        生成HTML报告
#
# 示例:
#   ./scripts/dev/run_benchmarks.sh              # 运行所有基准测试
#   ./scripts/dev/run_benchmarks.sh --baseline  # 保存为基线
#   ./scripts/dev/run_benchmarks.sh --compare   # 与基线对比

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# 输出带颜色的消息
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# 检查依赖
check_dependencies() {
    info "检查依赖工具..."

    if ! command -v cargo &> /dev/null; then
        error "cargo 未安装"
    fi

    if ! command -v git &> /dev/null; then
        error "git 未安装"
    fi

    # 检查 Criterion 是否安装
    if ! grep -q "criterion" Cargo.toml; then
        warning "Criterion 未在 Cargo.toml 中配置"
    fi

    success "依赖检查通过"
}

# 运行基准测试
run_benchmarks() {
    info "运行基准测试..."

    cargo bench --all-features "$@"

    success "基准测试完成"
}

# 保存基线
save_baseline() {
    local baseline_name="${1:-main}"

    info "保存基线结果: $baseline_name"

    # Criterion 会自动保存到 target/criterion
    cargo bench -- --save-baseline "$baseline_name"

    success "基线已保存: $baseline_name"
}

# 与基线对比
compare_baseline() {
    local baseline_name="${1:-main}"

    info "与基线对比: $baseline_name"

    cargo bench -- --baseline "$baseline_name"

    success "对比完成"
}

# 生成火焰图
generate_flamegraph() {
    info "生成火焰图..."

    if ! cargo install --list | grep -q "flamegraph"; then
        warning "flamegraph 未安装，正在安装..."
        cargo install flamegraph
    fi

    cargo flamegraph --bench bench_main

    if [ -f "flamegraph.svg" ]; then
        success "火焰图已生成: flamegraph.svg"
        info "可以用浏览器打开查看: firefox flamegraph.svg"
    else
        error "火焰图生成失败"
    fi
}

# 堆分配分析
run_dhat_analysis() {
    info "运行堆分配分析..."

    if ! cargo install --list | grep -q "dhat"; then
        warning "dhat 未安装，正在安装..."
        cargo install dhat
    fi

    # 运行测试并启用 DHAT
    DHAT=1 cargo test --bench bench_main -- --test-threads=1

    success "DHAT 分析完成"
    info "使用 dhat/ddhat 查看: dhat-ddhat target/dhat.heap"
}

# 生成HTML报告
generate_html_report() {
    info "生成 HTML 报告..."

    local report_dir="$PROJECT_ROOT/target/benchmark-reports"
    mkdir -p "$report_dir"

    # Criterion 会生成 HTML 报告
    cargo bench -- --output-format html > "$report_dir/index.html"

    success "HTML 报告已生成: $report_dir/index.html"
}

# 性能建议
show_performance_tips() {
    cat << 'EOF'

🔧 性能优化建议:

1. **查看瓶颈**
   - 使用 cargo flamegraph 生成火焰图
   - 使用 criterion 的详细输出分析热点

2. **优化方向**
   - 批量操作: 增大批次大小
   - 并行处理: 使用 rayon 或 tokio 并发
   - 内存优化: 预分配、对象池、避免克隆
   - 算法优化: 选择更高效的数据结构

3. **持续监控**
   - 定期运行基准测试
   - 在 CI/CD 中集成性能回归检测
   - 保存历史基线进行对比

4. **文档参考**
   - Criterion Book: https://bheisler.github.io/criterion.rs/book/
   - Flamegraph Guide: https://nnethercote.github.io/perf-book/flamegraphs.html

EOF
}

# 主函数
main() {
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}   quantix-rust 性能测试工具${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""

    check_dependencies

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --baseline)
                shift
                save_baseline "$@"
                exit 0
                ;;
            --compare)
                shift
                compare_baseline "$@"
                exit 0
                ;;
            --flamegraph)
                generate_flamegraph
                exit 0
                ;;
            --dhat)
                run_dhat_analysis
                exit 0
                ;;
            --html)
                generate_html_report
                exit 0
                ;;
            --tips)
                show_performance_tips
                exit 0
                ;;
            -h|--help)
                cat << EOF
用法: $0 [选项]

选项:
  --baseline [name]    保存当前结果为基线
  --compare [name]     与基线对比
  --flamegraph         生成火焰图
  --dhat              堆分配分析
  --html              生成HTML报告
  --tips              显示性能优化建议
  -h, --help          显示此帮助信息

无参数时运行完整的基准测试套件。
EOF
                exit 0
                ;;
            *)
                error "未知选项: $1 (使用 --help 查看帮助)"
                ;;
        esac
    done

    # 默认: 运行基准测试
    run_benchmarks

    echo ""
    success "所有性能测试完成！"
    echo ""
    info "使用 --baseline 保存基线，使用 --compare 对比性能变化"
    echo ""
}
