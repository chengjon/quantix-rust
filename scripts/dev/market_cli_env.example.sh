#!/usr/bin/env bash

# Example environment template for the market CLI acceptance workflow.
# Copy, adjust for your environment, then:
#   source scripts/dev/market_cli_env.example.sh

# ClickHouse runtime path for market strength queries.
export CLICKHOUSE_URL="http://127.0.0.1:8123"
export CLICKHOUSE_DB="quantix"

# Upstream MySQL sync path for `quantix risk sync industry --standard shenwan`.
export QUANTIX_UPSTREAM_MYSQL_URL="mysql://127.0.0.1:3306"
export QUANTIX_UPSTREAM_MYSQL_DB="mystocks"
export QUANTIX_UPSTREAM_MYSQL_USER="root"
export QUANTIX_UPSTREAM_MYSQL_PASSWORD="replace-me"

# Optional overrides for local risk / industry state.
export QUANTIX_RISK_DIR="$HOME/.quantix/risk"
export QUANTIX_INDUSTRY_DB_PATH="$QUANTIX_RISK_DIR/industry_reference.db"
