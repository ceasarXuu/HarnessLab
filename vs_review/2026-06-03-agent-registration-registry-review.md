# Subagent VS Review: agent registration registry

- Created: 2026-06-03T07:30:44+0800
- Updated: 2026-06-03T08:58:46+0800
- Report schema: adversarial-v1
- Task: complete `plans/2026-06-03-agent-registration-registry.md` and adversarially review/fix the implementation
- Report path: `vs_review/2026-06-03-agent-registration-registry-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: closed

## Review Objective

Make HarnessLab agent registration low-friction, readable, field-documented, materializable, and safe enough for the first MVP user path: register a CLI harness such as `claude-ds`, run real Terminal-Bench/SWE-bench style benchmarks through HarnessLab, capture reproducible run snapshots, and produce reports without mock shortcuts.

## Reviewed Scope

- Agent profile model and reference metadata: `crates/harnesslab-core/src/agent_profile.rs`, `crates/harnesslab-core/src/agent_profile_reference.rs`, `crates/harnesslab-core/src/config.rs`.
- CLI registration UX: `crates/harnesslab-cli/src/agent_registry/`, `crates/harnesslab-cli/src/app.rs`, `crates/harnesslab-cli/src/doctor.rs`.
- Runtime materialization, snapshots, reports, and external runner bridges: `crates/harnesslab-cli/src/runner/`, `crates/harnesslab-report/src/lib.rs`, `integrations/terminal_bench/`.
- Real-run scripts, traceability registry, and docs: `scripts/test-after-change.sh`, `scripts/verify-terminal-bench-registered-setup.sh`, `tests/REQUIREMENTS.toml`, `tests/TEST_REGISTRY.toml`, `docs/`.

## Round Records

| Round | Reviewer | Session / Job ID | Outcome |
|---|---|---|---|
| 1 | implementation-adversary | `019e8aad-8de3-7aa0-81ed-f9882fb34fda` | Tool failure: invalid `gpt-image-2` request surfaced as `image_generation_user_error`; no usable review output. |
| 1 | implementation-adversary replacement | `019e8ab4-6f69-7b73-8ae7-dd9560121df7` | Same tool failure; implementation lens degraded to later focused closure reviews. |
| 1 | test-validity-adversary | `019e8aad-beb8-7ad2-9d50-e6b6c57fe92e` | Blocking findings accepted. |
| 2 | materialized-setup closure | `019e8ac0-3c55-74f0-802e-a98801cc50bd` | Blocking finding accepted. |
| 3 | materialized-setup re-review | `019e8aca-7b5b-73d2-a5bd-705cd3583d4b` | No blocking findings for AGT-REG-005 closure. |
| 4 | security/failure critic | `019e8ac0-7381-77c3-997d-80896ad2d9a2` | Two blocking findings accepted. |
| 5 | redaction closure | `019e8ad9-8bfd-7cc2-91c0-453b7864f497` | One blocking redaction variant remained. |
| 6 | final redaction closure | `019e8ae4-cba3-7062-b09d-6d6720936457` | No blocking findings; accepted blockers closed. |

All usable review rounds were read-only, fresh internal sessions with main-agent history, reasoning, drafts, and conclusions excluded.

## Accepted Findings And Fixes

| Finding | Source Round | Disposition | Fix Evidence |
|---|---|---|---|
| AGT-REG-005 only tested the Python bridge directly and did not prove the real HarnessLab registration flow. | Round 1 test-validity | Accepted and fixed. | Added `scripts/verify-terminal-bench-registered-setup.sh`; wired `scripts/test-after-change.sh --select AGT-REG-005`; script runs `harnesslab init`, `doctor --json`, and `harnesslab run --agent registered-setup --benchmark terminal-bench --split smoke --json`. |
| Runtime proof lacked preserved artifacts showing setup executed before the agent. | Round 1 test-validity | Accepted and fixed. | Registered setup smoke checks marker ordering and official `agent_setup_stdout.log`, `agent_setup_stderr.log`, and `agent_setup_command.sha256`. |
| Real setup proof did not prove Terminal-Bench consumed the materialized runtime snapshot rather than a raw profile path. | Round 2 closure | Accepted and fixed. | Added Rust source-boundary contract `agt_reg_005_terminal_bench_env_uses_materialized_setup_not_raw_profile`; bridge logs `agent_setup_command.sha256`; smoke compares bridge hash to `agent-runtime.materialized.json.setup_script`. |
| Attempt-level Terminal-Bench `agent/command.txt` leaked setup secrets from rendered commands. | Round 4 critic | Accepted and fixed. | Changed external command snapshot writer to redact known raw env secrets and shell-escaped variants; added `terminal_bench_redaction_contract.rs`. |
| Terminal-Bench bridge setup failure carried stale official benchmark `test_failed` warning text. | Round 4 critic | Accepted and fixed. | Suppressed official benchmark warnings when an infra/setup failure is already classified; added `terminal_bench_setup_failure_contract.rs` for `execution/external_runner_setup_failed`. |
| Redaction still leaked single-quote shell-escaped secrets such as `pa'\''ss`. | Round 5 closure | Accepted and fixed. | Expanded redaction refs to include raw, inner shell-escaped, and fully single-quoted variants; updated redaction contract to use secret `pa'ss` and assert both raw and escaped forms are absent. |

## Deferred Non-Blocking Notes

- Round 3 noted that setup hash proof is intentionally scoped to materialized setup identity, not a full secrecy proof. The later redaction rounds covered public artifact leakage separately.
- Round 6 noted redacted shell tokens may appear as `'[REDACTED]'`. This is acceptable for artifact readability and does not affect runtime execution.
- Round 6 noted the external snapshot redaction helper is shared beyond Terminal-Bench. This is acceptable because it only changes public command snapshot artifacts, not the runtime command.

## Final Validation

Latest full local gate:

```bash
scripts/test-after-change.sh
```

Observed pass signals:

- `PASS terminal-bench registered setup`
- `bridge setup command hash matches materialized snapshot`
- `PASS terminal-bench import timeout cleanup`
- `PASS terminal-bench import success cleanup`
- `PASS terminal-bench docker activity watchdog`
- `PASS terminal-bench docker activity grace expiry`
- `registry ok: 21 requirements, 145 tests`
- `secret scan ok`
- `coverage ok: lines 95.47% (8028/8409), branches 83.22% (744/894), modules 2`
- `new-file coverage ok: 7 new production Rust files are present in coverage data`
- `PASS scripts/test-after-change.sh`

## Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Rejected findings backed by evidence: none
- Deferred findings documented: yes
- Blocked reason: none
- Allowed to proceed: yes

## Final Conclusion

The adversarial review is closed. All accepted blocking findings were fixed, re-reviewed by fresh internal subagents, and covered by the full local gate.
