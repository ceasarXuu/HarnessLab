#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/ornnlab-run-dev-api.XXXXXX")"
RUN_PID=""

find_free_port() {
  uv run python - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

show_logs() {
  if [[ -d "$TEST_ROOT/logs" ]]; then
    tail -n 80 "$TEST_ROOT/logs/backend.log" 2>/dev/null || true
    tail -n 80 "$TEST_ROOT/logs/frontend.log" 2>/dev/null || true
  fi
}

cleanup() {
  local exit_code=$?
  if [[ -n "$RUN_PID" ]] && kill -0 "$RUN_PID" 2>/dev/null; then
    kill -TERM "$RUN_PID" 2>/dev/null || true
    for _ in {1..40}; do
      if ! kill -0 "$RUN_PID" 2>/dev/null; then
        break
      fi
      sleep 0.25
    done
    if kill -0 "$RUN_PID" 2>/dev/null; then
      echo "run_dev.sh did not stop after SIGTERM" >&2
      kill -KILL "$RUN_PID" 2>/dev/null || true
      exit_code=1
    fi
    wait "$RUN_PID" 2>/dev/null || true
  fi
  if [[ "$exit_code" -ne 0 ]]; then
    show_logs
  fi
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

cd "$REPO_ROOT"

INVALID_LOG="$TEST_ROOT/invalid-mode.log"
if VITE_ORNNLAB_DATA_MODE=invalid bash run_dev.sh >"$INVALID_LOG" 2>&1; then
  echo "run_dev.sh accepted an invalid API data mode" >&2
  exit 1
fi
rg -q 'VITE_ORNNLAB_DATA_MODE 必须为 api 或 mock' "$INVALID_LOG"

BACKEND_PORT="$(find_free_port)"
FRONTEND_PORT="$(find_free_port)"
ORNNLAB_HOME="$TEST_ROOT/home" \
ORNNLAB_PORT="$BACKEND_PORT" \
ORNNLAB_FRONTEND_PORT="$FRONTEND_PORT" \
ORNNLAB_DEV_LOG_DIR="$TEST_ROOT/logs" \
bash run_dev.sh >"$TEST_ROOT/run-dev.log" 2>&1 &
RUN_PID=$!

BACKEND_HEALTH="http://127.0.0.1:${BACKEND_PORT}/api/webui/v1/system/health"
FRONTEND_HEALTH="http://127.0.0.1:${FRONTEND_PORT}/api/webui/v1/system/health"
for _ in {1..120}; do
  if curl -sf "$BACKEND_HEALTH" >/dev/null && curl -sf "$FRONTEND_HEALTH" >/dev/null; then
    break
  fi
  if ! kill -0 "$RUN_PID" 2>/dev/null; then
    echo "run_dev.sh exited before API mode became ready" >&2
    exit 1
  fi
  sleep 0.25
done

curl -sf "$BACKEND_HEALTH" | uv run python -c 'import json, sys; assert json.load(sys.stdin)["data"]["items"]'
curl -sf "$FRONTEND_HEALTH" | uv run python -c 'import json, sys; assert json.load(sys.stdin)["data"]["items"]'

# run_dev.sh prints its summary box after its own proxy health check;
# wait for the patterns to appear rather than racing the log write.
for _ in {1..40}; do
  if rg -q '前端模式 : api' "$TEST_ROOT/run-dev.log" 2>/dev/null; then
    break
  fi
  sleep 0.25
done
rg -q '前端模式 : api' "$TEST_ROOT/run-dev.log"
rg -q '前端 proxy' "$TEST_ROOT/run-dev.log"
