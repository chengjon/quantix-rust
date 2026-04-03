#!/bin/bash
# CI 数据库测试环境准备脚本
#
# 安装客户端工具并等待 PostgreSQL / ClickHouse 就绪

set -euo pipefail

wait_for_postgres() {
    for i in {1..30}; do
        if pg_isready -h localhost -p 5432 -U test; then
            echo "PostgreSQL is ready"
            return 0
        fi

        echo "Waiting for PostgreSQL... ($i/30)"
        sleep 2
    done

    echo "PostgreSQL did not become ready in time"
    return 1
}

wait_for_clickhouse() {
    for i in {1..30}; do
        if clickhouse-client --query "SELECT 1" 2>/dev/null; then
            echo "ClickHouse is ready"
            return 0
        fi

        echo "Waiting for ClickHouse... ($i/30)"
        sleep 2
    done

    echo "ClickHouse did not become ready in time"
    return 1
}

sudo apt-get update
sudo apt-get install -y \
    postgresql-client \
    clickhouse-client \
    wget

wait_for_postgres
wait_for_clickhouse
