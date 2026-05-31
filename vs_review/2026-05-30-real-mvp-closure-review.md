# Real MVP Closure Review

## Round 1: Product Runtime Hardening Review

### Review Input

#### Objective
Verify that the current implementation moves HarnessLab toward a non-demo first runnable version: product CLI should expose real benchmarks by default, preserve test fixtures as internal engineering tools, honor configured run/cache paths, produce official SWE-bench Pro JSONL predictions, and stream process logs without buffering entire logs in memory.

#### Review Target
Recent changes in benchmark registry visibility, CLI command surface, run directory resolution, benchmark cache resolution, SWE-bench Pro prediction path, and process execution logging.

#### Target Locations
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-cli/src/app.rs`
- `crates/harnesslab-cli/src/lib.rs`
- `crates/harnesslab-cli/src/benchmark_data.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-cli/tests/cli_contract.rs`
- `docs/prd.md`
- `docs/mvp-development-spec.md`

#### Change Introduction
The product benchmark list now hides `fake-terminal` and `fake-patch` unless fixture descriptors are explicitly enabled, while direct adapter lookup still supports fixture tests. CLI `run` accepts `--concurrency` and `--attempts`. Default `runs_dir` and `benchmarks_dir` now resolve to the configured HarnessLab home instead of accidentally nesting `~/.harnesslab` under that home. SWE-bench Pro predictions now use `prediction.jsonl`. Host process stdout/stderr are streamed to files through reader threads.

#### Risk Focus
- Product still exposes fake/mock benchmark surfaces by default.
- Hiding fixture descriptors breaks internal fake benchmark runtime tests or replay.
- Configured `runs_dir` or `benchmarks_dir` behavior is still wrong for default or custom paths.
- Process streaming can hang, drop logs, or mishandle timeout.
- SWE-bench Pro external runner no longer matches official evaluator expectations.
- Tests prove only fake paths and do not protect the real product path.

#### Assumptions To Attack
- `benchmark list` without fixture env contains only real benchmarks.
- Direct `fake-terminal` and `fake-patch` runs still work for test engineering.
- `report open latest` follows configured `runs_dir`.
- `prediction.jsonl` is accepted by the current SWE-bench Pro runner path.
- Streamed log writers always complete on success and timeout.
- The new command overrides are actually reflected in `run.json`.

#### Adversarial Lenses
- requirements
- runtime behavior
- testing
- security and data leakage
- maintainability
- user-facing product semantics

#### Verification Status
Passed so far:
- `cargo check --workspace`
- `cargo test -p harnesslab-adapters c_bench_001`
- `cargo test -p harnesslab-cli --test cli_contract cli_003_m0_json_commands_have_stable_shape`
- `cargo test -p harnesslab-cli --test external_smoke_contract int_011_swe_bench_pro_smoke_runs_external_evaluator_contract`
- `cargo test -p harnesslab-cli default_benchmarks_dir_uses_harnesslab_home_not_nested_home`
- `cargo test -p harnesslab-cli default_runs_dir_uses_harnesslab_home_not_nested_home`
- `cargo test -p harnesslab-cli custom_runs_dir_can_be_relative_to_harnesslab_home`
- `cargo test -p harnesslab-infra c_sbox_002_host_exec_echo_captures_stdout`
- `cargo test -p harnesslab-infra c_sbox_003_host_exec_timeout_is_structured`

Not yet completed at review launch:
- full `scripts/test-after-change.sh`
- fresh real Terminal-Bench smoke after these edits
- fresh real SWE-bench Pro smoke after these edits

#### Reviewer Instructions
- Use a fresh internal subagent session.
- Do not inherit main-agent history.
- Read target files directly.
- Do not modify files.
- Focus on blocking correctness gaps and evidence gaps, not style.
- Cite evidence paths and line numbers when possible.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Multi-file product/runtime change needs direct code review | runtime behavior, product semantics, maintainability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| product-runtime-adversary | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e77f7-0bc3-76a3-ab2a-99cb1a769d5e` | spawn tool result | false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round1-product-runtime | product-runtime-adversary | 1 | `019e77f7-0bc3-76a3-ab2a-99cb1a769d5e` | within timeout | completed | reviewer returned REQUEST CHANGES | accept blocking fixes and re-review |

### Reviewer Outputs

#### round1-product-runtime

Summary: REQUEST CHANGES. The reviewer found that the product discovery change is directionally correct, but not yet safe as a non-demo runnable baseline.

Blocking findings:
- Configured `benchmarks_dir` was ignored when the configured path did not exist, allowing fallback to repo `.benchmarks` and false readiness.
- SWE-bench Pro prediction output had been changed to JSONL, but the currently invoked Scale Pro evaluator reads a JSON array via `json.load`.
- Host process timeout could still hang when descendants inherit stdout/stderr pipes because only the parent process was killed.

Non-blocking risk:
- `--concurrency` and `--attempts` were persisted in `run.json` but not preserved in the user-visible original command snapshot.

Required fixes:
- Make configured/env benchmark roots authoritative even when missing.
- Keep public `prediction.jsonl` while giving the Scale Pro evaluator the JSON array it actually consumes.
- Kill the process group on timeout and bound post-kill stream joins.
- Include non-default `--concurrency` and `--attempts` in `command.txt` / report original command.

Missing tests:
- Missing configured benchmark cache must not fall back to repo `.benchmarks`.
- Actual SWE-bench Pro smoke should prove the evaluator path still succeeds.
- Timeout with a background child holding inherited pipes.
- `report open latest` under custom `runs_dir`.

Missing logs / observability:
- Benchmark root selection is still not explicitly recorded.
- Timeout kill and stream join diagnostics are limited.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| Configured benchmark path fallback | accept | `resolve_benchmarks_dir` now returns env/config paths authoritatively, even when missing. Added unit and integration regression tests. |
| SWE-bench Pro prediction/evaluator mismatch | accept | Public artifact remains `prediction.jsonl` and now includes `instance_id`, `model_name_or_path`, and `model_patch`; the Scale Pro evaluator receives `prediction.eval.json` as the JSON array contract it actually consumes. Fresh real SWE-bench Pro smoke passed. |
| Timeout can hang on descendant pipe holders | accept | Host process execution now starts Unix children in a new session, kills the process group on timeout, and bounds post-kill stream joins. Added inherited-pipe timeout regression test. |
| Original command drops run overrides | accept | `original_run_command` now includes non-default `--concurrency` and `--attempts`; added unit coverage. |
| Benchmark root and timeout observability gaps | defer | Not blocking for this fix round, but should be handled in the next observability pass. Runtime artifacts and failure logs still capture execution output. |

### Historical Closure Status Before Round 2

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: not yet at this point in the review timeline
- Blocking re-review passed: not yet at this point in the review timeline
- Proceed decision: wait for Round 2

## Round 2: Blocking Fix Closure Review

### Review Input

#### Objective
Verify closure of Round 1 blocking findings after fixes. The target is still a non-demo first runnable HarnessLab MVP path, especially real benchmark cache selection, SWE-bench Pro evaluator compatibility, timeout behavior, and command reproducibility.

#### Target Locations
- `crates/harnesslab-cli/src/benchmark_data.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-cli/tests/benchmark_contract.rs`
- `crates/harnesslab-cli/tests/init_contract.rs`

#### Change Introduction
Accepted Round 1 findings were fixed. Configured/env benchmark roots are authoritative. SWE-bench Pro now writes a public JSONL artifact with official fields and gives the Scale Pro evaluator a JSON array sidecar. Host process execution uses a Unix session/process-group kill on timeout and bounded stream joins. Original command snapshots include non-default run overrides.

#### Verification Status
Passed:
- `cargo check --workspace`
- `cargo test -p harnesslab-cli --test benchmark_contract bench_005_configured_missing_benchmark_dir_does_not_use_repo_cache`
- `cargo test -p harnesslab-cli --test init_contract int_001_report_open_latest_uses_configured_runs_dir`
- `cargo test -p harnesslab-cli original_command_preserves_non_default_run_overrides`
- `cargo test -p harnesslab-cli configured_missing_benchmarks_dir_is_authoritative`
- `cargo test -p harnesslab-infra c_sbox_003_timeout_kills_background_pipe_holder`
- Fresh real Terminal-Bench smoke passed: `.benchmarks/_harnesslab-home-terminal-real/runs/tb-oracle-terminal-bench-smoke-20260530T082919766149Z`
- Fresh real SWE-bench Pro smoke passed: `.benchmarks/_harnesslab-home-terminal-real/runs/swe-gold-swe-bench-pro-smoke-20260530T082920748303Z`

#### Reviewer Instructions
- Fresh internal subagent session.
- Do not inherit main-agent history.
- Read target files directly.
- Do not modify files.
- Focus only on closure of accepted Round 1 blocking findings and any new high-impact regression introduced by the fixes.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| blocking-fix-closure | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e7803-114c-7fd1-8e28-7ffab53ffd86` | spawn tool result | false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round2-blocking-closure | blocking-fix-closure | 1 | `019e7803-114c-7fd1-8e28-7ffab53ffd86` | within timeout | completed | reviewer returned COMMENT with no blocking findings | address medium follow-up and run final gate |

### Reviewer Outputs

#### round2-blocking-closure

Summary: COMMENT. Scope reviewed 6 target files plus adjacent call sites. Round 1 blockers are closed and no new high-impact regressions were found.

Verified closures:
- Configured benchmark roots are authoritative even when missing.
- SWE-bench Pro public `prediction.jsonl` and evaluator `prediction.eval.json` match their respective contracts.
- Host timeout behavior kills descendant pipe holders and bounds output joins.
- Original command provenance preserves non-default `--concurrency` and `--attempts`.
- Configured `runs_dir` is respected for `report open latest`.

Non-blocking risk:
- Real SWE smoke passed, but evaluator stderr showed ambient `/opt/anaconda3` NumPy/pyarrow tracebacks. This weakens reproducibility. Recommended fix: sanitize Python-related env for the SWE evaluator invocation and add a smoke assertion that ambient Python env does not leak.

Missing tests:
- No CLI-level contract asserts `--concurrency` and `--attempts` are written into `command.txt`.
- No contract test directly inspects generated `prediction.jsonl` and `prediction.eval.json` contents.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| Round 1 blocker closure | accept | Round 2 found no blocking regressions. |
| Ambient Python environment leakage | accept | SWE evaluator invocation now unsets `PYTHONHOME`, `PYTHONPATH`, and `PYTHONUSERBASE`, exports `PYTHONNOUSERSITE=1`, and has a CLI integration regression test that fails if evaluator `uv` sees ambient Python env. |
| Missing CLI-level command snapshot contract | defer | Unit-level provenance coverage exists. Full CLI contract remains a follow-up unless final audit shows product behavior unproven. |
| Missing direct prediction artifact contract | defer | Real smoke verifies artifact existence and evaluator consumption. Direct file-shape contract remains a follow-up unless final audit shows product behavior unproven. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Allowed to proceed: yes

## Round 3: Environment Isolation Follow-up Review

### Review Input

#### Objective
Review the follow-up change made after Round 2: SWE-bench Pro evaluator execution should be reproducible and not inherit ambient host Python package paths.

#### Target Locations
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `crates/harnesslab-cli/tests/swe_env_contract.rs`
- `crates/harnesslab-cli/src/benchmark_data.rs`
- `crates/harnesslab-cli/src/runner/store.rs`

#### Change Introduction
The SWE-bench Pro evaluator command now unsets host Python env variables before invoking `uv run`, and a CLI-level integration test asserts that ambient `PYTHONPATH`, `PYTHONHOME`, and `PYTHONUSERBASE` do not leak into the evaluator `uv` process. Additional path expansion tests cover configured `~`, `~/...`, and absolute paths for benchmark and run roots.

#### Risk Focus
- Env sanitization breaks `uv` or Docker host setup.
- Sanitization is applied too broadly or too narrowly.
- The new integration test proves only the fake path and not the actual command behavior.
- Coverage-oriented tests add brittle host-specific assumptions.

#### Verification Status
Passed:
- `cargo test -p harnesslab-cli --test swe_env_contract int_012_swe_evaluator_ignores_ambient_python_environment`
- `cargo test -p harnesslab-cli custom_benchmarks_dir_expands_home_and_absolute_paths`
- `cargo test -p harnesslab-cli custom_runs_dir_expands_home_and_absolute_paths`

#### Reviewer Instructions
- Fresh internal subagent session.
- Do not inherit main-agent history.
- Read target files directly.
- Do not modify files.
- Focus on high-impact correctness, reproducibility, and testing gaps introduced by this follow-up.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| env-isolation-closure | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e7cb0-fe93-7112-8ae7-f16431e7fe15` | spawn tool result | false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round3-env-isolation | env-isolation-closure | 1 | `019e7cb0-fe93-7112-8ae7-f16431e7fe15` | within timeout | completed | reviewer returned COMMENT with no blocking findings | accept low-risk reproducibility follow-up |

### Reviewer Outputs

#### round3-env-isolation

Summary: COMMENT. No blocking regressions found. The env scrub is applied on the actual SWE evaluator shell command, and the new CLI contract drives `harnesslab run` end-to-end through that path. Path-expansion tests derive expectations from runtime `HOME` and tempdirs rather than hard-coded host paths.

Blocking findings:
- None.

Non-blocking risks:
- The env regression test did not positively assert `PYTHONNOUSERSITE=1`.
- SWE metadata extraction used a separate unsanitized `uv run`; evaluator isolation was fixed, but full SWE-path reproducibility was not uniform.
- Round 3 report was still pending before this update.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| No blocking findings | accept | Proceed after final targeted and full-gate verification. |
| Missing `PYTHONNOUSERSITE=1` assertion | accept | The CLI env contract now fails unless fake `uv` sees `PYTHONNOUSERSITE=1`. |
| Metadata extraction unsanitized | accept | SWE metadata extraction now uses the same Python env isolation before `uv run`; the contract failed before this production change and passed after it. |
| Round 3 report pending | accept | Report now records Round 3 reviewer output and main-agent response. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Allowed to proceed: yes

## Round 4: Final SWE Env Closure Review

### Review Input

#### Objective
Review the final follow-up after Round 3: the SWE-bench Pro metadata and evaluator `uv run` invocations should both be isolated from ambient Python env variables, and the CLI contract should prove the positive `PYTHONNOUSERSITE=1` guard.

#### Target Locations
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `crates/harnesslab-cli/tests/swe_env_contract.rs`
- `vs_review/2026-05-30-real-mvp-closure-review.md`

#### Change Introduction
The env contract was strengthened to fail if metadata extraction or evaluator execution sees `PYTHONPATH`, `PYTHONHOME`, `PYTHONUSERBASE`, or misses `PYTHONNOUSERSITE=1`. The test was confirmed red before the metadata production fix. Metadata extraction now unsets the same Python env variables and exports `PYTHONNOUSERSITE=1` before `uv run`.

#### Risk Focus
- Final test still does not exercise both `uv run` paths.
- Metadata env isolation changes command execution semantics incorrectly.
- File size or review artifact constraints are violated.
- Any high-impact regression introduced by the last follow-up.

#### Verification Status
Passed:
- Red before implementation: `cargo test -p harnesslab-cli --test swe_env_contract int_012_swe_evaluator_ignores_ambient_python_environment` failed on metadata env isolation.
- Green after implementation: `cargo test -p harnesslab-cli --test swe_env_contract int_012_swe_evaluator_ignores_ambient_python_environment`.

#### Reviewer Instructions
- Fresh internal subagent session.
- Do not inherit main-agent history.
- Read target files directly.
- Do not modify files.
- Focus only on this final env isolation closure and any new high-impact regression.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| final-env-closure | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e7cb6-6e94-7f40-8eef-a9c787d76775` | spawn tool result | false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round4-final-env-closure | final-env-closure | 1 | `019e7cb6-6e94-7f40-8eef-a9c787d76775` | within timeout | completed | reviewer returned COMMENT with no blocking findings | accept env-presence test precision fix |

### Reviewer Outputs

#### round4-final-env-closure

Summary: COMMENT. No blocking regression found. The production runner scrubs Python env on both real `uv run` call sites: metadata extraction and evaluator execution. The CLI contract drives `harnesslab run` end-to-end and success requires both fake `uv` branches to pass.

Blocking findings:
- None.

Non-blocking risks:
- The contract checked non-empty values rather than variable presence, so a future regression to `VAR=''` would not be caught.
- Round 4 review section was pending before this update.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| No blocking findings | accept | Proceed after final verification. |
| Env presence precision | accept | The CLI contract now checks `${VAR+x}` so empty-string environment variables still count as leaked. |
| Round 4 report pending | accept | Report now records Round 4 reviewer output and main-agent response. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Allowed to proceed: yes
