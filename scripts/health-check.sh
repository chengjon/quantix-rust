#!/bin/bash
# 容器健康检查脚本

set -euo pipefail

# 健康检查端点
HEALTH_URL="${HEALTH_URL:-http://localhost:8080/health}"

# 超时时间（秒）
TIMEOUT="${HEALTH_CHECK_TIMEOUT:-10}"

# 执行健康检查
if command -v curl >/dev/null 2>&1; then
    # 使用 curl
    response=$(curl -fsSL --max-time "$TIMEOUT" "$HEALTH_URL" || echo "failed")

    if [ "$response" = "failed" ]; then
        echo "Health check failed: Cannot connect to $HEALTH_URL"
        exit 1
    fi

    # 检查响应
    if echo "$response" | grep -q '"status":"ok"' || echo "$response" | grep -q '"healthy":true'; then
        echo "Health check passed"
        exit 0
    else
        echo "Health check failed: Invalid response"
        echo "Response: $response"
        exit 1
    fi
elif command -v wget >/dev/null 2>&1; then
    # 使用 wget 作为后备
    if wget -q --timeout="$TIMEOUT" -O - "$HEALTH_URL" | grep -q '"status":"ok"\|"healthy":true'; then
        echo "Health check passed"
        exit 0
    else
        echo "Health check failed"
        exit 1
    fi
else
    # 如果没有 curl 或 wget，尝试直接运行 quantix health 命令
    if command -v quantix >/dev/null 2>&1; then
        if quantix health >/dev/null 2>&1; then
            echo "Health check passed"
            exit 0
        else
            echo "Health check failed: quantix health command failed"
            exit 1
        fi
    else
        echo "Health check failed: No tools available"
        exit 1
    fi
fi
