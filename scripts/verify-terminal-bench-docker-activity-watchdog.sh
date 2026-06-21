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
WORK=".benchmarks/_harnesslab-docker-activity-watchdog-$STAMP"
HOME_DIR="$WORK/home"
DATASET_TASK="$WORK/benchmarks/terminal-bench/terminal-bench-core-0.1.1/hello-world"

mkdir -p "$HOME_DIR/agents" "$(dirname "$DATASET_TASK")"
cp -R "$SOURCE_TASK" "$DATASET_TASK"

DOCKERFILE="$DATASET_TASK/Dockerfile"
awk -v stamp="$STAMP" '
  { print }
  /^FROM / {
    print "RUN echo harnesslab-docker-activity-" stamp " >/tmp/harnesslab-activity && sleep 30"
  }
' "$DOCKERFILE" > "$DOCKERFILE.tmp"
mv "$DOCKERFILE.tmp" "$DOCKERFILE"
cat > "$DATASET_TASK/tests/setup-uv-pytest.sh" <<'EOF'
#!/bin/bash
set -euo pipefail
EOF
cat > "$DATASET_TASK/tests/run-uv-pytest.sh" <<'EOF'
#!/bin/bash
set -euo pipefail
python3 - <<'PY'
from pathlib import Path

hello_path = Path("/app/hello.txt")
assert hello_path.exists(), f"File {hello_path} does not exist"
assert hello_path.read_text() == "Hello, world!\n"
PY
printf '=========================== short test summary info ============================\n'
printf 'PASSED tests/test_outputs.py::test_hello_file_exists\n'
printf 'PASSED tests/test_outputs.py::test_hello_file_content\n'
EOF

target/debug/harnesslab --home "$HOME_DIR" init >"$WORK/init.log"

cat >"$HOME_DIR/agents/docker-activity.toml" <<EOF
schema_version = 1
name = "docker-activity"
kind = "custom"
display_name = "Docker Activity Agent"
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

set +e
HARNESSLAB_BENCHMARKS_DIR="$WORK/benchmarks" \
HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC=20 \
HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC=120 \
  target/debug/harnesslab \
  --home "$HOME_DIR" \
  run --agent docker-activity --benchmark terminal-bench --split smoke \
  --concurrency 1 --timeout-sec 80 --json \
  >"$WORK/run.json" 2>"$WORK/run.stderr"
STATUS=$?
set -e

if [ "$STATUS" -ne 0 ]; then
  echo "expected successful real docker activity run, got exit code $STATUS" >&2
  cat "$WORK/run.stderr" >&2
  exit 1
fi

RUN_DIR="$(python3 - "$WORK/run.json" <<'PY'
import json, sys
print(json.load(open(sys.argv[1]))["run_dir"])
PY
)"

python3 - "$RUN_DIR/results.json" <<'PY'
import json, sys
results = json.load(open(sys.argv[1]))
task = results["tasks"][0]
assert task["state"] == "success", task
assert task["failure_class"] == "none", task
assert task["failure_code"] is None, task
print("result ok: success")
PY

if rg --fixed-strings "external_runner_no_progress" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "real docker activity was misclassified as external_runner_no_progress" >&2
  exit 1
fi
if ! rg --fixed-strings "external_runner_activity" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing no-output activity proof event" >&2
  exit 1
fi
if ! rg --fixed-strings "no-output watchdog deferred by" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing no-output defer detail" >&2
  exit 1
fi
if ! rg --fixed-strings "no_output_timeout_sec=20" "$RUN_DIR/events.jsonl" >/dev/null; then
  echo "missing shortened watchdog configuration proof" >&2
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

echo "PASS terminal-bench docker activity watchdog"
echo "artifacts: $WORK"
