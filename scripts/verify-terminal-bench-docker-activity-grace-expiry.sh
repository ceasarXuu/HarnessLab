#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SOURCE_TASK=".benchmarks/terminal-bench/terminal-bench-core-0.1.1/hello-world"
if [ ! -d "$SOURCE_TASK" ]; then
  echo "missing Terminal-Bench hello-world dataset at $SOURCE_TASK" >&2
  exit 2
fi

docker info >/dev/null
cargo build -p harnesslab-cli >/dev/null

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
WORK=".benchmarks/_harnesslab-docker-activity-grace-expiry-$STAMP"
HOME_DIR="$WORK/home"
DATASET_TASK="$WORK/benchmarks/terminal-bench/terminal-bench-core-0.1.1/hello-world"

mkdir -p "$HOME_DIR/agents" "$(dirname "$DATASET_TASK")"
cp -R "$SOURCE_TASK" "$DATASET_TASK"

DOCKERFILE="$DATASET_TASK/Dockerfile"
awk -v stamp="$STAMP" '
  { print }
  /^FROM / {
    print "RUN echo harnesslab-docker-grace-expiry-" stamp " >/tmp/harnesslab-grace-expiry && sleep 90"
  }
' "$DOCKERFILE" > "$DOCKERFILE.tmp"
mv "$DOCKERFILE.tmp" "$DOCKERFILE"

target/debug/harnesslab --home "$HOME_DIR" init >"$WORK/init.log"

cat >"$HOME_DIR/agents/docker-stale-activity.toml" <<EOF
schema_version = 1
name = "docker-stale-activity"
kind = "custom"
display_name = "Docker Stale Activity Agent"
command = "printf '%s\\n' 'printf \"Hello, world!\\\\n\" > hello.txt'"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 20

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[usage]
parser = "none"

[labels]
terminal_bench_agent_import_path = "ornnlab_tb_agent:OrnnLabCommandAgent"
terminal_bench_agent_pythonpath = "$ROOT/integrations/terminal_bench"
EOF

STARTED="$(python3 - <<'PY'
import time
print(time.monotonic())
PY
)"
set +e
HARNESSLAB_BENCHMARKS_DIR="$WORK/benchmarks" \
HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC=8 \
HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC=80 \
  target/debug/harnesslab \
  --home "$HOME_DIR" \
  run --agent docker-stale-activity --benchmark terminal-bench --split smoke \
  --concurrency 1 --timeout-sec 40 --json \
  >"$WORK/run.json" 2>"$WORK/run.stderr"
STATUS=$?
set -e
ELAPSED="$(python3 - "$STARTED" <<'PY'
import sys, time
print(time.monotonic() - float(sys.argv[1]))
PY
)"

if [ "$STATUS" -ne 1 ]; then
  echo "expected execution failure from stale docker activity, got exit code $STATUS" >&2
  cat "$WORK/run.stderr" >&2
  exit 1
fi

RUN_DIR="$(python3 - "$WORK/run.json" <<'PY'
import json, sys
print(json.load(open(sys.argv[1]))["run_dir"])
PY
)"

python3 - "$RUN_DIR/results.json" "$ELAPSED" <<'PY'
import json, sys
results = json.load(open(sys.argv[1]))
elapsed = float(sys.argv[2])
task = results["tasks"][0]
assert task["state"] == "failure", task
assert task["failure_class"] == "execution", task
assert task["failure_code"] == "external_runner_no_progress", task
assert task["agent"]["termination_reason"] == "no_progress", task
assert elapsed < 70, elapsed
print(f"result ok: stale docker activity stopped after {elapsed:.1f}s")
PY

if ! rg --fixed-strings "activity_grace_sec=8" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing activity grace configuration proof" >&2
  exit 1
fi
if ! rg --fixed-strings "external_runner_no_progress" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing no-progress event" >&2
  exit 1
fi
if ! rg --fixed-strings "activity_grace_exhausted=true" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing activity grace exhaustion proof" >&2
  exit 1
fi
if ! rg --fixed-strings "current_activity=pid=" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing current activity diagnostic" >&2
  exit 1
fi
if ! rg --fixed-strings "last_activity=pid=" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing last activity diagnostic" >&2
  exit 1
fi

RUN_ID="$(basename "$RUN_DIR" | tr '[:upper:]' '[:lower:]')"
if docker ps -a --format '{{.Names}} {{.Label "com.docker.compose.project"}}' | rg "$RUN_ID"; then
  echo "found residual compose container for run $RUN_ID" >&2
  exit 1
fi
if docker network ls --format '{{.Name}} {{.Label "com.docker.compose.project"}}' | rg "$RUN_ID"; then
  echo "found residual compose network for run $RUN_ID" >&2
  exit 1
fi

echo "PASS terminal-bench docker activity grace expiry"
echo "artifacts: $WORK"
