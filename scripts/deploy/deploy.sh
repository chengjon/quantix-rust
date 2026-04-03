#!/bin/bash
# 部署脚本
#
# 用于部署应用到不同环境

set -euo pipefail

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
IMAGE_TAG="${IMAGE_TAG:-latest}"
ENVIRONMENT="${ENVIRONMENT:-production}"
HEALTH_URL="${HEALTH_URL:-http://localhost:8080/health}"
DEPLOY_ACCESS_URL="${DEPLOY_ACCESS_URL:-}"
PRODUCTION_DEPLOY_MODE="${PRODUCTION_DEPLOY_MODE:-auto}"
PRODUCTION_COMPOSE_FILE="${PRODUCTION_COMPOSE_FILE:-docker-compose.prod.yml}"
PRODUCTION_K8S_DIR="${PRODUCTION_K8S_DIR:-$PROJECT_ROOT/k8s/overlays/production}"
STAGING_COMPOSE_FILE="${STAGING_COMPOSE_FILE:-docker-compose.test.yml}"

if [[ "$HEALTH_URL" == */health ]]; then
    DEFAULT_LOCAL_ACCESS_URL="${HEALTH_URL%/health}"
else
    DEFAULT_LOCAL_ACCESS_URL="$HEALTH_URL"
fi

resolve_production_deploy_mode() {
    case "$PRODUCTION_DEPLOY_MODE" in
        compose|k8s)
            printf '%s\n' "$PRODUCTION_DEPLOY_MODE"
            ;;
        auto)
            if [ -f "$PROJECT_ROOT/$PRODUCTION_COMPOSE_FILE" ]; then
                printf 'compose\n'
            elif [ -d "$PRODUCTION_K8S_DIR" ]; then
                printf 'k8s\n'
            else
                return 1
            fi
            ;;
        *)
            return 1
            ;;
    esac
}

run_production_compose() {
    local compose_args=(-f docker-compose.yml -f "$PRODUCTION_COMPOSE_FILE")

    if [ "$DRY_RUN" = false ]; then
        cd "$PROJECT_ROOT"
        IMAGE_NAME="$IMAGE_NAME" VERSION="$IMAGE_TAG" docker-compose "${compose_args[@]}" up -d --force-recreate quantix
        IMAGE_NAME="$IMAGE_NAME" VERSION="$IMAGE_TAG" docker-compose "${compose_args[@]}" ps
    else
        log_info "[DRY RUN] IMAGE_NAME=$IMAGE_NAME VERSION=$IMAGE_TAG docker-compose ${compose_args[*]} up -d --force-recreate quantix"
    fi
}

require_compose_file() {
    local compose_file="$1"

    if [ ! -f "$PROJECT_ROOT/$compose_file" ]; then
        log_error "未找到 Compose 配置: $compose_file"
        exit 1
    fi
}

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

resolve_repo_slug() {
    if [ -n "${GITHUB_REPOSITORY:-}" ]; then
        printf '%s\n' "$GITHUB_REPOSITORY"
        return 0
    fi

    local remote_url
    remote_url="$(git -C "$PROJECT_ROOT" config --get remote.origin.url 2>/dev/null || true)"

    if [ -z "$remote_url" ]; then
        return 1
    fi

    remote_url="${remote_url#ssh://}"
    remote_url="${remote_url#https://}"
    remote_url="${remote_url#http://}"
    remote_url="${remote_url#git@}"
    remote_url="${remote_url#*@}"
    remote_url="${remote_url/:/\/}"

    if [[ "$remote_url" != github.com/* ]]; then
        return 1
    fi

    remote_url="${remote_url#github.com/}"
    remote_url="${remote_url%.git}"

    if [[ "$remote_url" != */* ]]; then
        return 1
    fi

    printf '%s\n' "$remote_url"
}

resolve_image_name() {
    if [ -n "${IMAGE_NAME:-}" ]; then
        printf '%s\n' "$IMAGE_NAME"
        return 0
    fi

    local repo_slug
    repo_slug="$(resolve_repo_slug)" || return 1
    printf 'ghcr.io/%s/quantix\n' "$repo_slug"
}

IMAGE_NAME="$(resolve_image_name)" || {
    log_error "无法推导默认镜像名称，请设置 IMAGE_NAME 或 GITHUB_REPOSITORY"
    exit 1
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

# 检查 kubectl（仅当生产环境实际执行 Kubernetes 部署）
if [ "$ENVIRONMENT" = "production" ] && [ "$DRY_RUN" = false ] && [ "$(resolve_production_deploy_mode)" = "k8s" ]; then
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
        require_compose_file "$STAGING_COMPOSE_FILE"
        if [ "$DRY_RUN" = false ]; then
            cd "$PROJECT_ROOT"
            # 使用测试环境配置
            docker-compose -f docker-compose.yml -f "$STAGING_COMPOSE_FILE" \
                up -d --force-recreate quantix
            docker-compose ps
        else
            log_info "[DRY RUN] docker-compose -f docker-compose.yml -f $STAGING_COMPOSE_FILE up -d"
        fi
        ;;

    production)
        log_info "部署到生产环境..."
        production_mode="$(resolve_production_deploy_mode)" || {
            log_error "无法确定生产部署方式，请设置 PRODUCTION_DEPLOY_MODE=compose|k8s"
            exit 1
        }

        if [ "$production_mode" = "compose" ]; then
            run_production_compose
        elif [ "$DRY_RUN" = false ]; then
            # 检查连接
            if ! kubectl cluster-info &> /dev/null; then
                log_error "无法连接到 Kubernetes 集群"
                exit 1
            fi

            # 部署到 Kubernetes
            cd "$PRODUCTION_K8S_DIR"

            # 应用配置
            kubectl apply -k .

            # 等待部署完成
            kubectl rollout status deployment/quantix --timeout=5m

            # 显示状态
            kubectl get pods -l app=quantix
        else
            log_info "[DRY RUN] kubectl apply -k $PRODUCTION_K8S_DIR"
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
                if curl -f "$HEALTH_URL" &> /dev/null; then
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
    if [ -n "$DEPLOY_ACCESS_URL" ]; then
        log_info "生产环境访问: $DEPLOY_ACCESS_URL"
    fi
else
    log_info "本地访问: ${DEPLOY_ACCESS_URL:-$DEFAULT_LOCAL_ACCESS_URL}"
fi
