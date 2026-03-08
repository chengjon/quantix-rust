-- PostgreSQL 初始化脚本

-- 创建扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- 创建表空间（可选）
-- CREATE TABLESPACE quantix_ts LOCATION '/var/lib/postgresql/data/tablespace';

-- 创建用户和权限
-- 注意：已经在 docker-compose.yml 中通过环境变量创建了用户
-- 这里只授权

GRANT ALL PRIVILEGES ON DATABASE quantix TO quantix;

-- 默认搜索路径
ALTER DATABASE quantix SET search_path TO public, quantix;

-- 创建模式（如果需要）
-- CREATE SCHEMA IF NOT EXISTS quantix AUTHORIZATION quantix;
-- GRANT ALL ON SCHEMA quantix TO quantix;

-- 性能优化配置
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;
ALTER SYSTEM SET work_mem = '2621kB';
ALTER SYSTEM SET min_wal_size = '1GB';
ALTER SYSTEM SET max_wal_size = '4GB';

-- 重载配置
SELECT pg_reload_conf();

-- 查询配置
-- SHOW ALL;
