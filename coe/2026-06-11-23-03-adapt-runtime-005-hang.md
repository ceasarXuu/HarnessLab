## Problem P-001

Status: investigating

Symptom: `scripts/verify-planned-adapter-selectors.sh` and the focused selector `ADAPT-RUNTIME-005` can exceed the expected test duration and appear stuck around `terminal_bench_runtime_event_contract::adapt_runtime_005_terminal_bench_event_taxonomy_is_stable`.

Expected behavior: `ADAPT-RUNTIME-005` should finish deterministically, proving the terminal-bench runtime event taxonomy without leaving long-running subprocesses.

Actual behavior: prior runs exceeded two minutes and had to be interrupted. A focused rerun also exceeded the normal command wait window before compaction.

Known facts:
- The failing selector runs `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`.
- The test includes `run_timeout_case`, which writes an official result then executes `sleep 20` with `HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC=2`.
- The test includes `run_no_progress_activity_case`, which executes `docker-buildx 5` with `HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC=2` and `HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC=8`.
- The test helper `fake_uvx_and_docker_buildx` symlinks `docker-buildx` to `/bin/sleep` on Unix.
- `HostProcessExecutor` kills the registered process group on timeout/no-progress, waits for the shell child, then joins stdout/stderr streaming threads with a 2 second join timeout.

Fix criteria:
- Focused `ADAPT-RUNTIME-005` completes within its expected timing window.
- No residual `terminal_bench_runtime_event_contract`, `cargo test`, `uvx`, or fake `docker-buildx` process remains after the selector.
- The broader planned selector runner no longer hangs at `ADAPT-RUNTIME-005`.

Resolution basis: pending confirmed hypothesis and fix-validation evidence.

## Hypothesis H-001

Status: active

Claim: `ADAPT-RUNTIME-005` hangs because one of its fake external-runner subprocess scenarios leaves a child process alive or a pipe open after the runner records timeout/no-progress, causing the test process or assert command wrapper to wait longer than intended.

Predictions:
- A focused test run with an outer timeout will either time out or show elapsed time far beyond the configured runner timeouts.
- During the stuck window, the process tree will include the test binary plus a shell/sleep descendant associated with the fake terminal-bench command, or the streaming join path will report delayed completion.
- If subprocess cleanup is made deterministic, the focused selector will complete and no residual process will remain.

Diagnostic evidence plan:
- Run the focused selector under a bounded outer timeout and inspect process state while it is running.
- If it exceeds the configured timeout envelope, capture the descendant command tree.
- Only after the stuck mechanism is confirmed, design a fix in the smallest responsible layer.

## Evidence E-001

Type: code-path

Supports: P-001 known facts and H-001 plausibility.

Observation: `terminal_bench_runtime_event_contract.rs` runs both a hard timeout scenario using `sleep 20` and a no-progress scenario using `docker-buildx 5`. `support/terminal_bench.rs` maps fake `docker-buildx` to `/bin/sleep` on Unix. `crates/harnesslab-infra/src/process.rs` kills the process group before waiting and stream joining.

## Evidence E-002

Type: reproduction

Supports: H-001 is not deterministically reproducible in an isolated focused selector run.

Observation: `/opt/homebrew/bin/timeout 45s cargo test -p harnesslab-cli --test terminal_bench_runtime_event_contract adapt_runtime_005_terminal_bench_event_taxonomy_is_stable -- --exact --nocapture` passed in 14.39s with exit code 0. This keeps the hang investigation open but downgrades the likelihood of a stable local code deadlock in the focused test alone.

## Evidence E-003

Type: regression

Refutes: H-001 as an active deterministic blocker for the current workspace state.

Observation: `/opt/homebrew/bin/timeout 180s scripts/verify-planned-adapter-selectors.sh` completed successfully. It ran `ADAPT-RUNTIME-005` inside the full selector script, where the focused test passed in 13.76s. The script ended with `adapter selectors ok: active=20 planned=9`.
