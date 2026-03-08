#!/bin/bash
# 安装 quantix-rust systemd 服务
# 将服务配置安装到系统

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SERVICE_DIR="$PROJECT_ROOT/config/systemd"
SYSTEMD_DIR="/etc/systemd/system"

echo "📦 安装 quantix-rust systemd 服务..."
echo ""

# 检查权限
if [[ $EUID -ne 0 ]]; then
   echo "❌ 此脚本需要 root 权限"
   echo "请使用: sudo $0"
   exit 1
fi

# 检查服务文件
if [ ! -d "$SERVICE_DIR" ]; then
    echo "❌ 服务配置目录不存在: $SERVICE_DIR"
    exit 1
fi

# 复制服务文件
echo "📄 复制服务文件..."
cp "$SERVICE_DIR"/*.service "$SYSTEMD_DIR/"

# 重载 systemd
echo "🔄 重载 systemd 配置..."
systemctl daemon-reload

# 启用服务（不启动）
echo "✅ 启用服务..."
systemctl enable quantix-data-collector.service 2>/dev/null || true
systemctl enable quantix-strategy-runner.service 2>/dev/null || true
systemctl enable quantix-task-scheduler.service 2>/dev/null || true

echo ""
echo "✅ 服务安装完成！"
echo ""
echo "可用服务："
echo "  - quantix-data-collector.service"
echo "  - quantix-strategy-runner.service"
echo "  - quantix-task-scheduler.service"
echo ""
echo "使用 ./scripts/services.sh 管理服务："
echo "  ./scripts/services.sh {start|stop|restart|status|logs} <service>"
echo ""
echo "示例："
echo "  ./scripts/services.sh start data-collector"
echo "  ./scripts/services.sh status data-collector"
echo "  ./scripts/services.sh logs data-collector"
