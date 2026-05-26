Resolution status after follow-up fixes:

- Blocking finding 1 fixed by wiring `RunSandboxCleanup` into `execute_plan` with pre-run and post-run best-effort cleanup events.
- Blocking finding 2 fixed by writing file-mode instructions into the mounted workspace and passing `/workspace/instruction.txt` to Docker agents.
- Blocking finding 3 fixed by validating `run.json` and `agent-profile.snapshot.json` before `run resume` execution.
- Docker container handle cleanup now uses a local RAII guard so `exec` errors and early returns still attempt destroy without masking successful task results.
- Final gate after fixes: `scripts/test-after-change.sh` passed with 101 tests, 96.08% line coverage, 86.61% branch coverage, and new-file coverage enabled.

Original adversarial review follows.

---

## Adversarial Code Review: Uncommitted Changes

### BLOCKING

**1. Container lifecycle leak — `cleanup_orphans` never wired into any run entrypoint**
`crates/harnesslab-cli/src/runner.rs` — all entrypoints (`execute_new_run`, `resume_run`, `replay_run`)
`crates/harnesslab-infra/src/docker.rs:187-222` — `cleanup_orphans` exists and is tested

`DockerCliProvider::cleanup_orphans` is implemented with full test coverage (C-SBOX-008, C-SBOX-009) but is **never called** from `execute_new_run`, `resume_run`, `replay_run`, or `execute_plan`. If the process is killed (SIGTERM, OOM, panic propagation), all running `sleep infinity` containers remain on the host indefinitely. There is no startup pre-run cleanup either — if a previous run crashed, those containers are still running when the next run starts. The label-based filtering (`harnesslab.run_id=`) is already in place but unused.

**Reproduction**: Start a run against `terminal-bench` smoke with Docker available, kill the process before it completes, then `docker ps --filter label=harnesslab.run_id=<id>`. Containers will still be running.

---

**2. `InputMode::File` is silently broken for Docker sandboxes**
`crates/harnesslab-cli/src/runner/sandbox.rs:114-119` (render_command, File variant)
`crates/harnesslab-cli/src/runner.rs:283` (workspace is `attempt_dir.join("workspace")`)

In `render_command`, `InputMode::File` writes the instruction file to `attempt_dir.join("instruction.txt")` — the parent of the workspace. For Docker tasks, `DockerCliProvider::create_args` only mounts `workspace_host_path:/workspace` (docker.rs:238-243). The instruction file at `attempt_dir/instruction.txt` is **not mounted** into the container. The agent inside the container receives a command like `agent '/path/on/host/.../instruction.txt'` but that host path does not exist inside the container.

`InputMode::Stdin`, `InputMode::Argument`, and `InputMode::Tty` are unaffected because they don't rely on a file path.

**Reproduction**: Configure an agent with `input_mode = "file"`, run against `terminal-bench` smoke or `swe-bench-pro` smoke (both use Docker), observe agent reading a nonexistent file.

---

**3. `resume_run` skips profile validation**
`crates/harnesslab-cli/src/runner.rs:99-103` (resume_run)
Contrast with `replay_run` at line 121 and `execute_new_run` at line 47.

`resume_run` reads the profile from `agent-profile.snapshot.json` but never calls `profile.validate()?`. Every other entrypoint does. If the validation rules have changed between when the profile was snapshotted and when resume runs, an invalid profile (e.g., unsupported schema version, missing required fields) would be used without detection.

---

### HIGH

**4. No signal/interrupt handler for Docker container cleanup**
`crates/harnesslab-cli/src/runner.rs` — no `ctrlc` handler or panic hook

A `SIGINT`/`SIGTERM` during a run leaves containers running. With `concurrency > 1` and long-running agent tasks, this is the most likely leak scenario. The `sleep infinity` container model means leaked containers consume CPU/memory until manually removed.

---

**5. `SandboxHandle` has no `Drop` implementation**
`crates/harnesslab-infra/src/docker.rs:59-65`

The struct is a plain data holder. Correctness depends entirely on every code path remembering to call `DockerCliProvider::destroy`. Currently `sandbox.rs` does this in all paths, but any future code that acquires a handle and doesn't destroy it (including via `?` propagation that skips cleanup) silently leaks. An RAII guard or at minimum a `Drop` impl that logs a warning would catch regressions.

---

**6. `replay_run` no longer validates global config**
`crates/harnesslab-cli/src/runner.rs:117-153`

Old code called `execute_new_run` which ran `validate_global_config`. New code calls `execute_plan` directly without config validation. This may be intentional (replay shouldn't fail because you changed your config), but it means a corrupted `config.toml` with zero concurrency/attempts (rejected by `validate_global_config` at `core/config.rs:142`) would not be caught. `validate_run_spec` at line 137 does catch these in the replayed spec (since the spec inherits from the source), so impact is mitigated.

---

### MEDIUM

**7. Test `doc_004` may have wrong expected exit code**
`crates/harnesslab-cli/tests/doctor_contract.rs:45-56`

The test runs `doctor --json` on a freshly-initialized home and asserts `.code(3)` (error). Assuming `init` creates a valid `fake` agent with an existing command (`echo`), the agent checks are `"ok"`, and benchmark checks are mixed `"ok"`/`"warning"`. The `overall_status` would be `"warning"` → exit code 1, not 3. Either:
- `init` creates an agent with a missing command (producing `"error"`), in which case the test should document this dependency explicitly, or
- The test is incorrect and will fail when run.

---

**8. Thread panics in `execute_attempts` propagate only after chunk completion**
`crates/harnesslab-cli/src/runner.rs:201-227`

If a task panics inside `thread::spawn`, the panic is caught by `handle.join()` and wrapped in an error. However, the chunk loop then **returns immediately** (`?` on line 222), skipping any remaining spawned threads in the current chunk. Those threads will continue running (with their Docker containers) until they complete or the process exits. The containers are destroyed when the threads complete naturally, but this is a transient leak during the error path.

---

**9. `execute_plan` does not call `cleanup_orphans` for the current run_id before exit**
`crates/harnesslab-cli/src/runner.rs:155-199`

After `execute_plan` completes (success or failure), any containers created during the run should be guaranteed destroyed. `run_agent` in sandbox.rs calls destroy in its success/exec-error paths and the `sandbox_failure` path, but if `DockerCliProvider::create` succeeds and `DockerCliProvider::exec` succeeds, and then the thread is killed externally, the destroy is never reached. A `cleanup_orphans(spec.run_id)` call at the end of `execute_plan` (or in a `Drop`-based guard) would provide a safety net.

---

### LOW / RESIDUAL RISKS

**10. Triple-nested shell quoting** — `docker.rs:316-323`, `sandbox.rs:133-135`
Agent commands pass through host shell → docker exec → container shell. Correct as written, tested in `c_sbox_005`, but a single quoting mistake in any layer would cause silent failures (commands partially executed, truncated arguments). Worth a dedicated fuzz test.

**11. No end-to-end Docker forward-path integration test**
`docker_tests.rs` covers arg construction and fake-runner lifecycle. `external_smoke_contract.rs` covers the "Docker missing → sandbox_create_failed" path. But the full create-exec-destroy cycle with real Docker is never tested in CI. The doc acknowledges this as intentional ("真 Docker 正向 smoke 只应在 Docker CLI 和 daemon 可用时执行"), but it means the most complex code path has zero automated coverage.

**12. `InputMode::File` writes instruction to `attempt_dir`, not `workspace`** (overlaps with finding #2 for Docker, but also matters for host tasks if sandbox isolation is tightened later)

---

### What's solid

- The **partition+resume** design (`partition_attempts` → `planned_attempts` → `attempt_result_path`) is well-structured and the test `replay_002` correctly verifies that completed attempts are preserved while missing ones are re-scheduled.
- **Thread safety** in `execute_attempts` is correct — all data is cloned/owned before move into threads, and each thread writes to a disjoint filesystem path.
- **Sandbox failure encoding** (`sandbox_failure` function, `AgentExecution.sandbox_failure`) cleanly separates "Docker failed to create" from "agent process classified normally", with proper log file writing.
- **Replay determinism** via `benchmark.snapshot.json` with adapter-level fallback is the right architecture.
- **Test coverage** is comprehensive for the new modules — `sandbox.rs` has unit tests for failure recording, command rendering, and timeout overrides; `replay.rs` is exercised through runner tests; `docker.rs` has 9 contract tests; new integration tests cover doctor benchmark readiness and replay.
