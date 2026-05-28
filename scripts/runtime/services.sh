#!/bin/bash
# systemd 服务管理脚本
# 简化 quantix-rust 服务的管理操作

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 服务名称映射
declare -A SERVICES
SERVICES=(
    ["data-collector"]="quantix-data-collector.service"
    ["strategy-runner"]="quantix-strategy-runner.service"
    ["task-scheduler"]="quantix-task-scheduler.service"
)

# 显示使用说明
show_usage() {
    echo "用法: $0 {start|stop|restart|status|logs|enable|disable} <service>"
    echo ""
    echo "操作："
    echo "  start    - 启动服务"
    echo "  stop     - 停止服务"
    echo "  restart  - 重启服务"
    echo "  status   - 查看服务状态"
    echo "  logs     - 查看服务日志（实时）"
    echo "  enable   - 启用服务（开机自启）"
    echo "  disable  - 禁用服务"
    echo ""
    echo "可用服务："
    echo "  - data-collector      数据采集服务"
    echo "  - strategy-runner    策略运行服务"
    echo "  - task-scheduler     任务调度服务"
    echo ""
    echo "示例："
    echo "  $0 start data-collector"
    echo "  $0 status data-collector"
    echo "  $0 logs data-collector"
    echo ""
    echo "管理所有服务："
    echo "  $0 start-all"
    echo "  $0 stop-all"
    echo "  $0 status-all"
}

# 获取完整服务名
get_service_name() {
    local short_name=$1
    echo "${SERVICES[$short_name]}"
}

require_valid_service() {
    local service_name=$1

    if [ -z "$service_name" ]; then
        echo -e "${RED}错误: 请指定服务名称${NC}"
        show_usage
        exit 1
    fi

    if [ -z "${SERVICES[$service_name]}" ]; then
        echo -e "${RED}错误: 未知服务 '$service_name'${NC}"
        show_usage
        exit 1
    fi
}

# 启动服务
start_service() {
    local service=$(get_service_name "$1")
    echo -e "${GREEN}▶ 启动服务: $service${NC}"
    systemctl start "$service"
}

# 停止服务
stop_service() {
    local service=$(get_service_name "$1")
    echo -e "${YELLOW}■ 停止服务: $service${NC}"
    systemctl stop "$service"
}

# 重启服务
restart_service() {
    local service=$(get_service_name "$1")
    echo -e "${GREEN}↻ 重启服务: $service${NC}"
    systemctl restart "$service"
}

# 查看状态
show_status() {
    local service=$(get_service_name "$1")
    systemctl status "$service"
}

# 查看日志
show_logs() {
    local service=$(get_service_name "$1")
    echo -e "${GREEN}📋 实时日志: $service${NC}"
    echo "按 Ctrl+C 退出"
    echo ""
    journalctl -u "$service" -f
}

# 启用服务
enable_service() {
    local service=$(get_service_name "$1")
    echo -e "${GREEN}✓ 启用服务: $service${NC}"
    systemctl enable "$service"
}

# 禁用服务
disable_service() {
    local service=$(get_service_name "$1")
    echo -e "${YELLOW}✗ 禁用服务: $service${NC}"
    systemctl disable "$service"
}

# 启动所有服务
start_all() {
    echo -e "${GREEN}▶ 启动所有服务...${NC}"
    for short_name in "${!SERVICES[@]}"; do
        start_service "$short_name"
        sleep 1
    done
}

# 停止所有服务
stop_all() {
    echo -e "${YELLOW}■ 停止所有服务...${NC}"
    for short_name in "${!SERVICES[@]}"; do
        stop_service "$short_name"
        sleep 1
    done
}

# 显示所有服务状态
status_all() {
    echo -e "${GREEN}📊 所有服务状态:${NC}"
    echo ""
    for short_name in "${!SERVICES[@]}"; do
        local service=$(get_service_name "$short_name")
        echo -e "${GREEN}=== $service ===${NC}"
        systemctl is-active "$service" && echo -e "状态: ${GREEN}运行中${NC}" || echo -e "状态: ${RED}已停止${NC}"
        echo ""
    done
}

# 主逻辑
ACTION=$1
SERVICE=$2

case "$ACTION" in
    start)
        require_valid_service "$SERVICE"
        start_service "$SERVICE"
        ;;
    stop)
        require_valid_service "$SERVICE"
        stop_service "$SERVICE"
        ;;
    restart)
        require_valid_service "$SERVICE"
        restart_service "$SERVICE"
        ;;
    status)
        require_valid_service "$SERVICE"
        show_status "$SERVICE"
        ;;
    logs)
        require_valid_service "$SERVICE"
        show_logs "$SERVICE"
        ;;
    enable)
        require_valid_service "$SERVICE"
        enable_service "$SERVICE"
        ;;
    disable)
        require_valid_service "$SERVICE"
        disable_service "$SERVICE"
        ;;
    start-all)
        start_all
        ;;
    stop-all)
        stop_all
        ;;
    status-all)
        status_all
        ;;
    *)
        show_usage
        exit 1
        ;;
esac
