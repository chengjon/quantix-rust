#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DEFAULT_CLICKHOUSE_URL="http://localhost:8123"
DEFAULT_CLICKHOUSE_DB="quantix"
DEFAULT_UPSTREAM_MYSQL_URL="mysql://127.0.0.1:3306"
DEFAULT_UPSTREAM_MYSQL_DB="mystocks"
DEFAULT_UPSTREAM_MYSQL_USER="root"

DOTENV_PATH="$ROOT_DIR/.env"
LOCAL_ENV_PATH="$ROOT_DIR/.env.market.local"

if [[ -f "$DOTENV_PATH" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$DOTENV_PATH"
  set +a
fi

if [[ -f "$LOCAL_ENV_PATH" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$LOCAL_ENV_PATH"
  set +a
fi

read_value_from_file() {
  local file="$1"
  local key="$2"
  if [[ -f "$file" ]]; then
    grep -E "^${key}=" "$file" | tail -n 1 | sed "s/^${key}=//" || true
  fi
}

effective_value() {
  local key="$1"
  local fallback="$2"
  if [[ -n "${!key:-}" ]]; then
    printf '%s' "${!key}"
  else
    printf '%s' "$fallback"
  fi
}

mask_if_secret() {
  local key="$1"
  local value="$2"
  if [[ "$key" == *PASSWORD* ]]; then
    if [[ -n "$value" ]]; then
      printf '***'
    else
      printf ''
    fi
  else
    printf '%s' "$value"
  fi
}

print_row() {
  local key="$1"
  local fallback="$2"
  local dotenv_value local_value effective
  dotenv_value="$(read_value_from_file "$DOTENV_PATH" "$key")"
  local_value="$(read_value_from_file "$LOCAL_ENV_PATH" "$key")"
  effective="$(effective_value "$key" "$fallback")"

  echo "$key"
  echo "  default : $(mask_if_secret "$key" "$fallback")"
  echo "  .env    : $(mask_if_secret "$key" "${dotenv_value:-}")"
  echo "  local   : $(mask_if_secret "$key" "${local_value:-}")"
  echo "  runtime : $(mask_if_secret "$key" "$effective")"

  if [[ -n "${dotenv_value:-}" && -n "${local_value:-}" && "$dotenv_value" != "$local_value" ]]; then
    echo "  note    : .env.market.local overrides .env"
  fi
}

echo "== Market CLI Env Doctor =="
echo "repo: $ROOT_DIR"
echo ".env: $DOTENV_PATH"
echo ".env.market.local: $LOCAL_ENV_PATH"
echo

print_row "CLICKHOUSE_URL" "$DEFAULT_CLICKHOUSE_URL"
print_row "CLICKHOUSE_DB" "$DEFAULT_CLICKHOUSE_DB"
print_row "CLICKHOUSE_USER" "default"
print_row "CLICKHOUSE_PASSWORD" ""
print_row "QUANTIX_UPSTREAM_MYSQL_URL" "$DEFAULT_UPSTREAM_MYSQL_URL"
print_row "QUANTIX_UPSTREAM_MYSQL_DB" "$DEFAULT_UPSTREAM_MYSQL_DB"
print_row "QUANTIX_UPSTREAM_MYSQL_USER" "$DEFAULT_UPSTREAM_MYSQL_USER"
print_row "QUANTIX_UPSTREAM_MYSQL_PASSWORD" ""
