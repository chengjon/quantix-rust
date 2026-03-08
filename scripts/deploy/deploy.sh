#!/bin/bash
# 部署脚本
#
# 用于部署应用到不同环境

set -euo pipefail

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-ghcr.io/chengjon/quantix-rust/quantix}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
ENVIRONMENT="${ENVIRONMENT:-production}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 显示帮助信息
show_help() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

部署 Quantix Rust CLI 到指定环境

OPTIONS:
    -e, --environment ENV    部署环境 (dev|staging|production)
    -t, --tag TAG            Docker 镜像标签 (默认: latest)
    -d, --dry-run            模拟运行，不实际部署
    -h, --help               显示帮助信息

EXAMPLES:
    # 部署到生产环境
    $(basename "$0") --environment production

    # 部署特定版本
    $(basename "$0") --environment production --tag v1.0.0

    # 模拟部署
    $(basename "$0") --environment staging --dry-run

ENVIRONMENTS:
    dev         开发环境（本地）
    staging     测试环境
    production  生产环境
EOF
}

# 解析参数
DRY_RUN=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -t|--tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
done

# 验证环境
case "$ENVIRONMENT" in
    dev|staging|production)
        ;;
    *)
        log_error "无效的环境: $ENVIRONMENT"
        show_help
        exit 1
        ;;
esac

log_info "部署 Quantix Rust CLI"
log_info "环境: $ENVIRONMENT"
log_info "镜像: $IMAGE_NAME:$IMAGE_TAG"
if [ "$DRY_RUN" = true ]; then
    log_warn "模拟运行模式（不会实际部署）"
fi

# 检查 Docker
if ! command -v docker &> /dev/null; then
    log_error "Docker 未安装"
    exit 1
fi

# 检查 kubectl（如果是生产环境）
if [ "$ENVIRONMENT" = "production" ]; then
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl 未安装（生产环境需要）"
        exit 1
    fi
fi

# 拉取最新镜像
log_info "拉取 Docker 镜像..."
if [ "$DRY_RUN" = false ]; then
    docker pull "$IMAGE_NAME:$IMAGE_TAG"
else
    log_info "[DRY RUN] docker pull $IMAGE_NAME:$IMAGE_TAG"
fi

# 根据环境部署
case "$ENVIRONMENT" in
    dev)
        log_info "部署到开发环境..."
        if [ "$DRY_RUN" = false ]; then
            cd "$PROJECT_ROOT"
            docker-compose up -d --force-recreate quantix
            docker-compose ps
        else
            log_info "[DRY RUN] docker-compose up -d --force-recreate quantix"
        fi
        ;;

    staging)
        log_info "部署到测试环境..."
        if [ "$DRY_RUN" = false ]; then
            cd "$PROJECT_ROOT"
            # 使用测试环境配置
            docker-compose -f docker-compose.yml -f docker-compose.test.yml \
                up -d --force-recreate quantix
            docker-compose ps
        else
            log_info "[DRY RUN] docker-compose -f docker-compose.yml -f docker-compose.test.yml up -d"
        fi
        ;;

    production)
        log_info "部署到生产环境..."
        if [ "$DRY_RUN" = false ]; then
            # 检查连接
            if ! kubectl cluster-info &> /dev/null; then
                log_error "无法连接到 Kubernetes 集群"
                exit 1
            fi

            # 部署到 Kubernetes
            cd "$PROJECT_ROOT/k8s/overlays/production"

            # 应用配置
            kubectl apply -k .

            # 等待部署完成
            kubectl rollout status deployment/quantix --timeout=5m

            # 显示状态
            kubectl get pods -l app=quantix
        else
            log_info "[DRY RUN] kubectl apply -k k8s/overlays/production"
            log_info "[DRY RUN] kubectl rollout status deployment/quantix"
        fi
        ;;
esac

# 健康检查
log_info "执行健康检查..."
if [ "$DRY_RUN" = false ]; then
    case "$ENVIRONMENT" in
        dev|staging)
            # 等待服务启动
            sleep 10

            # 检查健康端点
            if command -v curl &> /dev/null; then
                if curl -f http://localhost:8080/health &> /dev/null; then
                    log_info "健康检查通过 ✓"
                else
                    log_error "健康检查失败 ✗"
                    exit 1
                fi
            fi
            ;;
        production)
            # Kubernetes 健康检查
            if kubectl get pods -l app=quantix | grep -q "Running"; then
                log_info "Pods 运行正常 ✓"
            else
                log_error "Pods 未正常运行 ✗"
                exit 1
            fi
            ;;
    esac
else
    log_info "[DRY RUN] 跳过健康检查"
fi

# 完成
log_info "部署完成！"
if [ "$ENVIRONMENT" = "production" ]; then
    log_info "生产环境访问: https://quantix.example.com"
else
    log_info "本地访问: http://localhost:8080"
fi
