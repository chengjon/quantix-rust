#!/bin/bash
# quantix-rust 状态采集脚本
# 用法: ./scripts/zellij/status-collector.sh [format]

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FORMAT="${1:-text}"

# 缓存文件
CACHE_DIR="/tmp/quantix-status"
CACHE_TTL=5  # 秒
mkdir -p "$CACHE_DIR"

# 获取 CPU 使用率
get_cpu() {
    local cache_file="$CACHE_DIR/cpu"
    local now=$(date +%s)

    if [[ -f "$cache_file" ]]; then
        local cache_time=$(stat -c %Y "$cache_file" 2>/dev/null || echo 0)
        if (( now - cache_time < CACHE_TTL )); then
            cat "$cache_file"
            return
        fi
    fi

    local cpu="0.0"
    if command -v top &> /dev/null; then
        cpu=$(top -bn1 2>/dev/null | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1 || echo "0.0")
    fi
    echo "$cpu" > "$cache_file"
    echo "$cpu"
}

# 获取内存使用
get_memory() {
    local cache_file="$CACHE_DIR/memory"
    local now=$(date +%s)

    if [[ -f "$cache_file" ]]; then
        local cache_time=$(stat -c %Y "$cache_file" 2>/dev/null || echo 0)
        if (( now - cache_time < CACHE_TTL )); then
            cat "$cache_file"
            return
        fi
    fi

    local mem="0.0GB"
    if command -v free &> /dev/null; then
        mem=$(free -m 2>/dev/null | awk 'NR==2{printf "%.1fGB", $3/1024}' || echo "0.0GB")
    fi
    echo "$mem" > "$cache_file"
    echo "$mem"
}

# 获取数据库状态
get_db_status() {
    local cache_file="$CACHE_DIR/db"
    local now=$(date +%s)

    if [[ -f "$cache_file" ]]; then
        local cache_time=$(stat -c %Y "$cache_file" 2>/dev/null || echo 0)
        if (( now - cache_time < CACHE_TTL )); then
            cat "$cache_file"
            return
        fi
    fi

    local status="❌"
    if command -v pg_isready &> /dev/null; then
        if pg_isready -q 2>/dev/null; then
            status="✅"
        fi
    fi
    echo "$status" > "$cache_file"
    echo "$status"
}

# 获取任务数量
get_task_count() {
    local cache_file="$CACHE_DIR/tasks"
    local now=$(date +%s)

    if [[ -f "$cache_file" ]]; then
        local cache_time=$(stat -c %Y "$cache_file" 2>/dev/null || echo 0)
        if (( now - cache_time < CACHE_TTL )); then
            cat "$cache_file"
            return
        fi
    fi

    local count="0"
    if command -v quantix &> /dev/null; then
        count=$(quantix task status --count 2>/dev/null || echo "0")
    fi
    echo "$count" > "$cache_file"
    echo "$count"
}

# 获取当前时间
get_time() {
    date +"%H:%M:%S"
}

# JSON 格式输出
output_json() {
    cat <<EOF
{
    "cpu": "$(get_cpu)",
    "memory": "$(get_memory)",
    "database": "$(get_db_status)",
    "tasks": "$(get_task_count)",
    "timestamp": "$(date -Iseconds)"
}
EOF
}

# 文本格式输出
output_text() {
    echo "CPU: $(get_cpu)% | MEM: $(get_memory) | DB: $(get_db_status) | Tasks: $(get_task_count) | $(get_time)"
}

# 状态栏格式输出
output_statusbar() {
    local cpu=$(get_cpu)
    local mem=$(get_memory)
    local db=$(get_db_status)
    local tasks=$(get_task_count)
    local time=$(get_time)

    # 颜色编码
    local cpu_color="green"
    if (( $(echo "$cpu > 80" | bc -l 2>/dev/null || echo 0) )); then
        cpu_color="red"
    elif (( $(echo "$cpu > 50" | bc -l 2>/dev/null || echo 0) )); then
        cpu_color="yellow"
    fi

    echo "#[fg=$cpu_color]CPU: ${cpu}%#[default] | MEM: $mem | DB: $db | Tasks: $tasks | $time"
}

# 主函数
main() {
    case "$FORMAT" in
        json)
            output_json
            ;;
        statusbar)
            output_statusbar
            ;;
        *)
            output_text
            ;;
    esac
}

main
