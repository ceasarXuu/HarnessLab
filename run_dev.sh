#!/usr/bin/env bash
# OrnnLab 本地开发联调启动器
#
# 同时启动 FastAPI 后端（默认 8765）和 Vite 前端 dev server，自动捕获
# 前端打印的真实 Local URL 并高亮显示。Ctrl-C 一次性停掉两个子进程。
#
# 用法：
#   bash run_dev.sh                       # 后端 8765，前端 API 模式 5173
#   ORNNLAB_PORT=9000 ORNNLAB_FRONTEND_PORT=9001 bash run_dev.sh
#   VITE_ORNNLAB_DATA_MODE=mock bash run_dev.sh  # 显式启动 mock 前端
#
# 依赖：uv、npm、Node.js 22+（详见 docs/playbooks/install-quickstart.md）

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT"

BACKEND_HOST="${ORNNLAB_HOST:-127.0.0.1}"
BACKEND_PORT="${ORNNLAB_PORT:-8765}"
BACKEND_URL="http://${BACKEND_HOST}:${BACKEND_PORT}"
FRONTEND_PORT="${ORNNLAB_FRONTEND_PORT:-5173}"
WEBUI_DATA_MODE="${VITE_ORNNLAB_DATA_MODE:-api}"

case "$WEBUI_DATA_MODE" in
  api|mock) ;;
  *)
    echo "[run_dev] ✗ VITE_ORNNLAB_DATA_MODE 必须为 api 或 mock，当前为：$WEBUI_DATA_MODE" >&2
    exit 2
    ;;
esac

LOG_DIR="${ORNNLAB_DEV_LOG_DIR:-${TMPDIR:-/tmp}/ornnlab-dev}"
mkdir -p "$LOG_DIR"
BACKEND_LOG="$LOG_DIR/backend.log"
FRONTEND_LOG="$LOG_DIR/frontend.log"
: > "$BACKEND_LOG"
: > "$FRONTEND_LOG"

BACKEND_PID=""
FRONTEND_PID=""
TAIL_PID=""
CLEANUP_DONE="false"

stop_process_tree() {
  local pid="$1"
  local child_pid

  # uv/npm 都可能再派生实际服务进程；只终止最外层 PID 会遗留监听端口。
  while IFS= read -r child_pid; do
    [[ -n "$child_pid" ]] && stop_process_tree "$child_pid"
  done < <(pgrep -P "$pid" 2>/dev/null || true)

  if kill -0 "$pid" 2>/dev/null; then
    kill -TERM "$pid" 2>/dev/null || true
  fi
}

wait_for_process_exit() {
  local pid="$1"
  local attempts=0

  while kill -0 "$pid" 2>/dev/null && [[ "$attempts" -lt 40 ]]; do
    sleep 0.25
    attempts=$((attempts + 1))
  done

  if kill -0 "$pid" 2>/dev/null; then
    kill -KILL "$pid" 2>/dev/null || true
  fi
  wait "$pid" 2>/dev/null || true
}

cleanup() {
  local exit_code=$?
  if [[ "$CLEANUP_DONE" == "true" ]]; then
    return
  fi
  CLEANUP_DONE="true"
  trap - INT TERM EXIT
  echo
  echo "[run_dev] 收到退出信号，停止子进程..."
  if [[ -n "$FRONTEND_PID" ]] && kill -0 "$FRONTEND_PID" 2>/dev/null; then
    stop_process_tree "$FRONTEND_PID"
    wait_for_process_exit "$FRONTEND_PID"
  fi
  if [[ -n "$BACKEND_PID" ]] && kill -0 "$BACKEND_PID" 2>/dev/null; then
    stop_process_tree "$BACKEND_PID"
    wait_for_process_exit "$BACKEND_PID"
  fi
  if [[ -n "$TAIL_PID" ]] && kill -0 "$TAIL_PID" 2>/dev/null; then
    stop_process_tree "$TAIL_PID"
    wait_for_process_exit "$TAIL_PID"
  fi
  echo "[run_dev] 已停止。日志保留在 $LOG_DIR/"
  exit "$exit_code"
}
trap cleanup INT TERM EXIT

# ---- 启动后端 ----
echo "[run_dev] 启动后端 FastAPI @ ${BACKEND_URL} ..."
(
  exec uv run ornnlab web --host "$BACKEND_HOST" --port "$BACKEND_PORT" \
    >>"$BACKEND_LOG" 2>&1
) &
BACKEND_PID=$!

# 等后端进入可用状态（最多 30s）
for i in {1..60}; do
  if curl -sf "${BACKEND_URL}/api/webui/v1/system/health" >/dev/null 2>&1; then
    echo "[run_dev] ✓ 后端 ${BACKEND_URL}/api/webui/v1/system/health 已就绪"
    break
  fi
  if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
    echo "[run_dev] ✗ 后端进程异常退出，日志："
    tail -n 50 "$BACKEND_LOG" || true
    exit 1
  fi
  sleep 0.5
  if [[ "$i" -eq 60 ]]; then
    echo "[run_dev] ✗ 后端 30s 内未就绪，日志："
    tail -n 50 "$BACKEND_LOG" || true
    exit 1
  fi
done

# ---- 启动前端 ----
# 通过 ORNNLAB_API_TARGET 让 Vite proxy 指向当前后端；全栈联调默认 API 模式。
echo "[run_dev] 启动前端 Vite dev server ..."
(
  cd "$REPO_ROOT/frontend"
  VITE_ORNNLAB_DATA_MODE="$WEBUI_DATA_MODE" ORNNLAB_API_TARGET="$BACKEND_URL" ORNNLAB_FRONTEND_PORT="$FRONTEND_PORT" \
    exec npm run dev -- --host "$BACKEND_HOST" --port "$FRONTEND_PORT" --strictPort \
    >>"$FRONTEND_LOG" 2>&1
) &
FRONTEND_PID=$!

# 从 Vite stdout 解析真实 Local URL（避免端口被换时硬编码错）。最多等 30s。
# Vite 输出包含 ANSI 颜色码，需要先剥离再 grep，否则 URL 中嵌入转义码导致解析失败。
FRONTEND_URL=""
ANSI_RE=$'\x1b\\[[0-9;]*m'
for i in {1..60}; do
  if [[ -s "$FRONTEND_LOG" ]]; then
    # Vite 输出形如 "  ➜  Local:   http://127.0.0.1:4173/"
    FRONTEND_URL="$(sed "s/${ANSI_RE}//g" "$FRONTEND_LOG" \
      | grep -Eo 'http://[^[:space:]]+' \
      | grep -v "$BACKEND_URL" \
      | head -n 1 || true)"
    if [[ -n "$FRONTEND_URL" ]]; then
      break
    fi
  fi
  if ! kill -0 "$FRONTEND_PID" 2>/dev/null; then
    echo "[run_dev] ✗ 前端进程异常退出，日志："
    tail -n 50 "$FRONTEND_LOG" || true
    exit 1
  fi
  sleep 0.5
done

if [[ -z "$FRONTEND_URL" ]]; then
  echo "[run_dev] ✗ 未能在 30s 内解析到前端 Local URL，日志："
  tail -n 50 "$FRONTEND_LOG" || true
  exit 1
fi

FRONTEND_HEALTH_URL="${FRONTEND_URL%/}/api/webui/v1/system/health"
for i in {1..60}; do
  if curl -sf "$FRONTEND_HEALTH_URL" >/dev/null 2>&1; then
    echo "[run_dev] ✓ 前端 proxy ${FRONTEND_HEALTH_URL} 已就绪"
    break
  fi
  if ! kill -0 "$FRONTEND_PID" 2>/dev/null; then
    echo "[run_dev] ✗ 前端进程异常退出，日志："
    tail -n 50 "$FRONTEND_LOG" || true
    exit 1
  fi
  sleep 0.5
  if [[ "$i" -eq 60 ]]; then
    echo "[run_dev] ✗ 前端 proxy 30s 内未就绪，日志："
    tail -n 50 "$FRONTEND_LOG" || true
    exit 1
  fi
done

# ---- 输出摘要 ----
cat <<EOF

╔══════════════════════════════════════════════════════════════╗
║  OrnnLab dev 已启动                                          ║
╠══════════════════════════════════════════════════════════════╣
║  前端主页 : ${FRONTEND_URL}
║  前端模式 : ${WEBUI_DATA_MODE}
║  后端 API : ${BACKEND_URL}/api
║  后端日志 : ${BACKEND_LOG}
║  前端日志 : ${FRONTEND_LOG}
╚══════════════════════════════════════════════════════════════╝

Ctrl-C 一次性停掉前后端。

EOF

# 把两个进程的日志 tail 到当前终端，方便观察请求
tail -n 0 -F "$BACKEND_LOG" "$FRONTEND_LOG" &
TAIL_PID=$!

# 任意一个子进程退出 → 整体退出（兼容 macOS 默认 bash 3.2，未使用 wait -n）
while kill -0 "$BACKEND_PID" 2>/dev/null && kill -0 "$FRONTEND_PID" 2>/dev/null; do
  sleep 1
done

kill "$TAIL_PID" 2>/dev/null || true
