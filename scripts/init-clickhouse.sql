-- ClickHouse 初始化脚本

-- 创建数据库
CREATE DATABASE IF NOT EXISTS quantix;

-- 使用数据库
USE quantix;

-- 创建股票信息表
CREATE TABLE IF NOT EXISTS stock_info (
    code String,
    name String,
    market String,
    sector String,
    industry String,
    list_date Date,
    updated_at DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (market, code)
PARTITION BY toYYYYMM(list_date);

-- 创建实时行情表
CREATE TABLE IF NOT EXISTS stock_realtime_quotes (
    timestamp DateTime,
    code String,
    price Decimal(10, 2),
    volume UInt64,
    amount Decimal(20, 2),
    bid_price Decimal(10, 2),
    ask_price Decimal(10, 2),
    updated_at DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (timestamp, code)
PARTITION BY toYYYYMM(timestamp)
TTL timestamp + INTERVAL 30 DAY;

-- 创建 K线数据表
CREATE TABLE IF NOT EXISTS kline_data (
    timestamp DateTime,
    code String,
    period String,  -- 1m, 5m, 15m, 30m, 60m, 1d
    open Decimal(10, 2),
    high Decimal(10, 2),
    low Decimal(10, 2),
    close Decimal(10, 2),
    volume UInt64,
    amount Decimal(20, 2),
    updated_at DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (timestamp, code, period)
PARTITION BY (toYYYYMM(timestamp), period)
TTL timestamp + INTERVAL 365 DAY;

-- 创建除权除息事件表
CREATE TABLE IF NOT EXISTS gbbq_events (
    ex_date Date,
    code String,
    event_type String,  -- dividend, split, bonus
    bonus_ratio Decimal(10, 4),
    dividend_amount Decimal(10, 4),
    split_ratio Decimal(10, 4),
    updated_at DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (ex_date, code)
PARTITION BY toYYYYMM(ex_date);

-- 创建涨停板事件表
CREATE TABLE IF NOT EXISTS limit_up_events (
    trade_date Date,
    code String,
    limit_price Decimal(10, 2),
    open_price Decimal(10, 2),
    close_price Decimal(10, 2),
    volume UInt64,
    turnover_ratio Decimal(10, 4),
    reasons Array(String),
    updated_at DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (trade_date, code)
PARTITION BY toYYYYMM(trade_date)
TTL trade_date + INTERVAL 730 DAY;

-- 创建用户（如果需要）
-- CREATE USER IF NOT EXISTS quantix IDENTIFIED BY '';
-- GRANT ALL ON quantix.* TO quantix;

-- 优化设置
SET max_threads = 4;
SET max_memory_usage = 10000000000;

-- 查看表
-- SHOW TABLES;
-- SHOW CREATE TABLE stock_info;
