#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DATASET=".benchmarks/terminal-bench/terminal-bench-core-0.1.1"
if [ ! -d "$DATASET/hello-world" ]; then
  echo "missing Terminal-Bench hello-world dataset at $DATASET/hello-world" >&2
  exit 2
fi

docker info >/dev/null
cargo build -p harnesslab-cli >/dev/null

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
WORK=".benchmarks/_harnesslab-import-timeout-cleanup-$STAMP"
HOME_DIR="$WORK/home"
AGENT_SCRIPT="$WORK/agent-timeout-spawner.py"
MARKER="harnesslab-real-import-timeout-$STAMP"

mkdir -p "$HOME_DIR/agents" "$WORK"

target/debug/harnesslab --home "$HOME_DIR" init >"$WORK/init.log"

cat >"$AGENT_SCRIPT" <<'PY'
import os
import subprocess
import sys
import time

marker = os.environ["HARNESSLAB_TEST_ORPHAN_MARKER"]
child = (
    "import sys, time; "
    "print('child-ready', flush=True); "
    "time.sleep(60)"
)
subprocess.Popen(
    [sys.executable, "-c", child, marker],
    start_new_session=True,
)
print("agent-ready", flush=True)
time.sleep(60)
PY

cat >"$HOME_DIR/agents/timeout-import.toml" <<EOF
schema_version = 1
name = "timeout-import"
kind = "custom"
display_name = "Timeout Import Agent"
command = "python3 $ROOT/$AGENT_SCRIPT"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 3

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
terminal_bench_agent_import_path = "harnesslab_tb_agent:HarnessLabCommandAgent"
terminal_bench_agent_pythonpath = "$ROOT/integrations/terminal_bench"
EOF

set +e
HARNESSLAB_BENCHMARKS_DIR=".benchmarks" \
HARNESSLAB_TEST_ORPHAN_MARKER="$MARKER" \
  target/debug/harnesslab \
  --home "$HOME_DIR" \
  run --agent timeout-import --benchmark terminal-bench --split smoke \
  --concurrency 1 --timeout-sec 3 --json \
  >"$WORK/run.json" 2>"$WORK/run.stderr"
STATUS=$?
set -e

if [ "$STATUS" -ne 0 ]; then
  echo "expected successful CLI exit for benchmark verdict, got $STATUS" >&2
  cat "$WORK/run.stderr" >&2
  exit 1
fi

RUN_DIR="$(python3 - "$WORK/run.json" <<'PY'
import json, sys
payload = json.load(open(sys.argv[1]))
assert payload["verdict"] == "benchmark_failure", payload
print(payload["run_dir"])
PY
)"

python3 - "$RUN_DIR/results.json" <<'PY'
import json, sys
results = json.load(open(sys.argv[1]))
task = results["tasks"][0]
assert task["state"] == "failure", task
assert task["failure_class"] == "benchmark", task
assert task["failure_code"] == "agent_timeout", task
print("result ok: benchmark/agent_timeout")
PY

COMMAND_SNAPSHOT="$RUN_DIR/tasks/hello-world/attempts/1/agent/command.txt"
if ! rg --fixed-strings -- "--agent-import-path" "$COMMAND_SNAPSHOT" >/dev/null; then
  echo "missing import-path flag in command snapshot: $COMMAND_SNAPSHOT" >&2
  exit 1
fi
if ! rg --fixed-strings "harnesslab_tb_agent:HarnessLabCommandAgent" "$COMMAND_SNAPSHOT" >/dev/null; then
  echo "missing import-path proof in command snapshot: $COMMAND_SNAPSHOT" >&2
  exit 1
fi
if ! rg --fixed-strings -- "--global-agent-timeout-sec 33" "$COMMAND_SNAPSHOT" >/dev/null; then
  echo "missing import agent cleanup grace proof in command snapshot: $COMMAND_SNAPSHOT" >&2
  exit 1
fi
if ! rg --fixed-strings "external_runner_configured" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing runner configuration event" >&2
  exit 1
fi
if ! rg --fixed-strings "process_timeout_sec=1863" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing expected real runner timeout configuration" >&2
  exit 1
fi
if ! find "$RUN_DIR/tasks/hello-world/attempts/1/official" -name agent_error.log \
  -exec rg --fixed-strings "agent command timed out;" {} \; | rg . >/dev/null; then
  echo "missing adapter timeout agent_error.log proof" >&2
  exit 1
fi
if ! find "$RUN_DIR/tasks/hello-world/attempts/1/official" -name agent_error.log \
  -exec rg --fixed-strings "succeeded=True" {} \; | rg . >/dev/null; then
  echo "missing adapter cleanup success proof" >&2
  exit 1
fi
if ! find "$RUN_DIR/tasks/hello-world/attempts/1/official" -name agent_cleanup.log \
  -exec rg --fixed-strings "harnesslab agent cleanup after timeout:" {} \; | rg . >/dev/null; then
  echo "missing adapter timeout cleanup log proof" >&2
  exit 1
fi

if ps -axo pid=,stat=,command= | rg "$MARKER" | rg -v 'rg '; then
  echo "found live marker process after HarnessLab run: $MARKER" >&2
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

echo "PASS terminal-bench import timeout cleanup"
echo "artifacts: $WORK"
