# 多阶段构建 - 生产环境 Dockerfile
# Stage 1: 构建
FROM rust:1.75-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    wget \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# 复制 Cargo 配置
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 source 来缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs

# 构建依赖（缓存层）
RUN cargo build --release --bins && \
    rm -rf src

# 复制实际源代码
COPY src ./src

# 构建应用
RUN cargo build --release --bins && \
    # 清理不必要的文件以减小镜像大小
    rm -rf target/release/deps/target/release/build

# Stage 2: 运行时
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    postgresql-client \
    wget \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 quantix

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /build/target/release/quantix /usr/local/bin/quantix

# 复制配置文件
COPY config /app/config
COPY .env.example /app/.env.example

# 创建必要的目录
RUN mkdir -p /app/logs /app/data && \
    chown -R quantix:quantix /app

# 切换到非 root 用户
USER quantix

# 暴露端口（如果需要 HTTP API）
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD quantix health || exit 1

# 设置环境变量
ENV RUST_LOG=info \
    QUANTIX_CONFIG_DIR=/app/config

# 启动应用
CMD ["quantix"]
