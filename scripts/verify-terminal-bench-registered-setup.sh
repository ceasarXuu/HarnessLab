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
WORK=".benchmarks/_harnesslab-registered-setup-$STAMP"
WORK_ABS="$ROOT/$WORK"
HOME_DIR="$WORK/home"
AGENT_SCRIPT="$WORK/registered-setup-agent.py"
SETUP_MARKER="$WORK_ABS/setup-marker.txt"

mkdir -p "$HOME_DIR/agents" "$WORK"

target/debug/harnesslab --home "$HOME_DIR" init >"$WORK/init.log"
target/debug/harnesslab agent schema --json >"$WORK/agent-schema.json"

cat >"$AGENT_SCRIPT" <<'PY'
import os
import sys
from pathlib import Path

marker = Path(os.environ["HARNESSLAB_E2E_SETUP_MARKER"])
try:
    setup_value = marker.read_text()
except FileNotFoundError:
    print("setup marker missing", file=sys.stderr)
    sys.exit(42)
if setup_value != "setup-ran\n":
    print(f"unexpected setup marker: {setup_value!r}", file=sys.stderr)
    sys.exit(43)
print('printf "Hello, world!\\n" > hello.txt')
PY

cat >"$HOME_DIR/agents/registered-setup.toml" <<EOF
schema_version = 1
name = "registered-setup"
kind = "custom"
display_name = "Registered Setup Agent"
command = "python3 $ROOT/$AGENT_SCRIPT"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 30

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "custom"
required_commands = ["python3"]
run_as = "current"
commands = [
  'test -n "\${HARNESSLAB_E2E_SETUP_MARKER:-}"',
  'printf "setup-ran\n" > "\$HARNESSLAB_E2E_SETUP_MARKER"',
  'printf "setup-marker=%s\n" "\$HARNESSLAB_E2E_SETUP_MARKER"',
]

[skills]
inherit = true
allow = []
deny = []
include_paths = []

[tools]
inherit = true
allow = []
deny = []

[hooks]
inherit = true
allow = []
deny = []

[usage]
parser = "none"

[labels]
terminal_bench_agent_import_path = "harnesslab_tb_agent:HarnessLabCommandAgent"
terminal_bench_agent_pythonpath = "$ROOT/integrations/terminal_bench"
EOF

HARNESSLAB_BENCHMARKS_DIR=".benchmarks" \
  target/debug/harnesslab --home "$HOME_DIR" doctor --json >"$WORK/doctor.json"

python3 - "$WORK/doctor.json" <<'PY'
import json
import sys

doctor = json.load(open(sys.argv[1]))
assert doctor["status"] != "error", doctor
checks = {check["id"]: check for check in doctor["checks"]}
validation = checks["agent.registered-setup.validation"]
assert validation["status"] == "ok", validation
materialization = checks["agent.registered-setup.capabilities.materialization"]
assert materialization["status"] == "warning", materialization
warnings = materialization["details"]["warnings"]
assert "advanced_custom_setup" in warnings, materialization
print("doctor ok: registered setup materialized with explicit custom setup warning")
PY

set +e
HARNESSLAB_BENCHMARKS_DIR=".benchmarks" \
HARNESSLAB_E2E_SETUP_MARKER="$SETUP_MARKER" \
HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC=120 \
HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC=30 \
  target/debug/harnesslab \
  --home "$HOME_DIR" \
  run --agent registered-setup --benchmark terminal-bench --split smoke \
  --concurrency 1 --timeout-sec 90 --json \
  >"$WORK/run.json" 2>"$WORK/run.stderr"
STATUS=$?
set -e

if [ "$STATUS" -ne 0 ] && [ "$STATUS" -ne 2 ]; then
  echo "expected success or benchmark failure exit code, got $STATUS" >&2
  cat "$WORK/run.stderr" >&2
  exit 1
fi

RUN_DIR="$(python3 - "$WORK/run.json" <<'PY'
import json
import sys
print(json.load(open(sys.argv[1]))["run_dir"])
PY
)"
printf '%s\n' "$RUN_DIR" >"$WORK/run-dir.txt"

if [ ! -f "$SETUP_MARKER" ]; then
  echo "registered setup did not create marker: $SETUP_MARKER" >&2
  exit 1
fi
if [ "$(<"$SETUP_MARKER")" != "setup-ran" ]; then
  echo "registered setup marker has unexpected content" >&2
  exit 1
fi

python3 - "$RUN_DIR/results.json" <<'PY'
import json
import sys

results = json.load(open(sys.argv[1]))
task = results["tasks"][0]
assert task["task_id"] == "hello-world", task
assert task["failure_class"] != "execution", task
assert task["failure_code"] not in {
    "external_runner_setup_failed",
    "agent_output_parse_error",
    "agent_timeout",
    "agent_cleanup_failed",
}, task
print("result ok:", task["state"], task["failure_class"], task["failure_code"])
PY

python3 - "$RUN_DIR/agent-runtime.materialized.json" <<'PY'
import json
import sys

snapshot = json.load(open(sys.argv[1]))
setup = snapshot["setup_script"]
assert "HARNESSLAB_E2E_SETUP_MARKER" in setup, snapshot
assert "setup-ran" in setup, snapshot
assert snapshot["setup_summary"].startswith("preset=Custom"), snapshot
assert "advanced_custom_setup" in snapshot["warnings"], snapshot
print("materialized runtime snapshot ok")
PY

OFFICIAL_DIR="$RUN_DIR/tasks/hello-world/attempts/1/official"
SETUP_HASH="$(find "$OFFICIAL_DIR" -name agent_setup_command.sha256 -print -quit)"
SETUP_STDOUT="$(find "$OFFICIAL_DIR" -name agent_setup_stdout.log -print -quit)"
SETUP_STDERR="$(find "$OFFICIAL_DIR" -name agent_setup_stderr.log -print -quit)"
if [ -z "$SETUP_HASH" ] || [ ! -f "$SETUP_HASH" ]; then
  echo "missing official agent_setup_command.sha256 under $OFFICIAL_DIR" >&2
  exit 1
fi
if [ -z "$SETUP_STDOUT" ] || [ ! -f "$SETUP_STDOUT" ]; then
  echo "missing official agent_setup_stdout.log under $OFFICIAL_DIR" >&2
  exit 1
fi
if [ -z "$SETUP_STDERR" ] || [ ! -f "$SETUP_STDERR" ]; then
  echo "missing official agent_setup_stderr.log under $OFFICIAL_DIR" >&2
  exit 1
fi
if ! rg --fixed-strings "setup-marker=" "$SETUP_STDOUT" >/dev/null; then
  echo "missing setup stdout proof in $SETUP_STDOUT" >&2
  exit 1
fi
python3 - "$RUN_DIR/agent-runtime.materialized.json" "$SETUP_HASH" <<'PY'
import hashlib
import json
import sys

snapshot = json.load(open(sys.argv[1]))
expected = hashlib.sha256(snapshot["setup_script"].encode()).hexdigest()
actual = open(sys.argv[2]).read().strip()
assert actual == expected, {"expected": expected, "actual": actual}
print("bridge setup command hash matches materialized snapshot")
PY
if ! find "$OFFICIAL_DIR" -name agent_output.txt -exec rg --fixed-strings 'printf "Hello, world!\n" > hello.txt' {} \; | rg . >/dev/null; then
  echo "missing registered agent output proof under $OFFICIAL_DIR" >&2
  exit 1
fi

if ! rg --fixed-strings "agent_materialized_snapshot=agent-runtime.materialized.json" "$RUN_DIR/command.txt" >/dev/null; then
  echo "missing materialized snapshot pointer in root command artifact" >&2
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

echo "PASS terminal-bench registered setup"
echo "artifacts: $WORK"
