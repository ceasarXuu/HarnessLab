# HarnessLab Shim Retirement + Rust Legacy Workspace Retirement — Adversarial Review

Report path: `/vs_review/2026-06-22-harnesslab-shim-retirement-review.md`
Created: 2026-06-22
Round: 1
Status: In Progress (awaiting reviewer)

## Review Target

- Task type: refactor + migration (legacy removal)
- Risk level: Medium
- Scope:
  - PRD: `docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-prd.md`
  - Plan: `docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-plan.md`
  - Code commits (v0.1.4 branch):
    - `3faacfe` Phase 1: delete `harnesslab/` package + `pyproject.toml` console script + API field
    - `9832a65` Phase 2A: settings.py + doctor_service.py legacy retirement
    - `9da8588` Phase 2B+2C: harbor_subprocess/harbor_engine/docker_orphan/backup services + tests
    - `215a61c` Phase 2.5: rename `integrations/terminal_bench/harnesslab_tb_*.py` → `ornnlab_tb_*.py` + agent contract rename
    - `699e23e` Phase 3: delete Rust workspace + 11 Rust verify scripts + flip rust-legacy-fate.md
    - `af1dd52` + `1b51ba5` Phase 4: AC1–AC10 validation + close-out

## Review Input Packet (sent to reviewer)

```
Objective:
  Verify that the HarnessLab compatibility shim retirement and Rust legacy
  workspace retirement implemented in v0.1.4 commits 3faacfe..1b51ba5 is
  complete, safe, and free of hidden assumptions or broken paths.

Review Target Locations:
  - PRD: docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-prd.md
  - Plan: docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-plan.md
  - Modified Python: ornnlab/settings.py, ornnlab/services/{doctor,docker_orphan,backup,harbor_subprocess,harbor_engine}_service.py
  - Renamed integration: integrations/terminal_bench/ (7 ornnlab_tb_*.py files)
  - Deleted Python: harnesslab/__init__.py, __main__.py, cli.py, tests/python/test_settings_migration.py
  - Deleted Rust: crates/ (5 crates), xtask/, Cargo.toml, Cargo.lock, rust-toolchain*.toml, coverage-critical.toml
  - Deleted misc: tools.versions.toml, tests/TEST_REGISTRY.toml, tests/FROZEN_SELECTOR_MANIFEST.toml,
    11 verify-terminal-bench-*.sh + scan/coverage scripts, scripts/verify-test-after-change-select-output.sh
  - Modified verify scripts: scripts/verify-ornnlab-rebrand.py (deleted _check_migration_tests),
    scripts/verify-version-governance.py (path updates)
  - Test impact: tests/python/test_{system_api,harbor_config,harbor_subprocess,docker_orphan_service,
    backup_service,real_harbor_cancel_recovery,harbor_real_smoke,cli}.py — legacy cases deleted

Change Introduction:
  Project is pre-release with no real legacy users. User approved maximum-scope
  retirement of:
  1. HarnessLab Python package shim (import harnesslab, console script)
  2. ~/.harnesslab data dir + harnesslab.sqlite migration
  3. HARNESSLAB_* environment variable fallbacks (10+ variables across services)
  4. harnesslab.run_id Docker label scanning
  5. harnesslab-backup-manifest.json legacy backup recognition
  6. harnesslab_orphans API field double-emit
  7. terminal_bench agent contract rename (harnesslab-command → ornnlab-command,
     HarnessLabCommandAgent → OrnnLabCommandAgent, HARNESSLAB_AGENT_* env vars)
  8. Rust legacy workspace (crates/, xtask/, Cargo*) + 11 Rust verify scripts
  9. _check_migration_tests verify function (shim detection)

Out-of-scope (explicitly retained):
  - npm/harnesslab-transition/ package
  - bin/harnesslab.js root npm bin
  - package.json smoke:harnesslab-transition script
  - lib/source.js GitHub repo URL (HarnessLab.git is the actual repo name)
  - v0.1.3 release ledger (already published)
  - docs/archive/**, docs/plans/**, vs_review/**, coe/** historical references
  - scripts/verify-ornnlab-rebrand.py shim-detection strings (designed-in)
  - docs/architecture/harnesslab-vs-harbor.md historical stub
  - README.md npm transition section
  - docs/playbooks/npm-package-reservation.md npm history

Risk Focus / Assumptions to Attack:
  1. Settings.warnings / Settings.migration fields retained as empty tuple/None
     after deletion of legacy paths — does anything still rely on them?
  2. doctor_service.py status() output dropped 7 legacy fields. Has any caller
     (CLI, frontend, API consumer) been overlooked?
  3. The Phase 2.5 rename touched a "real external agent contract"
     (terminal_bench HarnessLabCommandAgent → OrnnLabCommandAgent). Is there
     residual coupling in non-rebranded paths?
  4. Phase 3 deleted 11 verify-terminal-bench-*.sh scripts as "Rust-only".
     Verify they had no Python-only entry points.
  5. AC1 grep uses path-level exclusions. Are any of the exclusions
     unjustified leaks? In particular: test files that may have residual
     legacy assertions that AC1 silently exempts.
  6. tests/python/test_harbor_subprocess.py test_subprocess_command_env_ignores_
     unknown_variable: the rename SOME_UNKNOWN_HARBOR_SUBPROCESS_COMMAND
     changed test intent from "ignore legacy variable" to "ignore unknown".
     Does that still cover the intended behavior?
  7. coverage-critical.toml was deleted as "Rust-only" — verify no Python
     coverage gate references it.
  8. test_settings_migration.py was deleted in full — verify no other test
     module imports its fixtures or helpers.
  9. backup_service.py removed LEGACY_MANIFEST_NAME but still imports nothing
     deleted — verify no dead imports remain.
  10. Plan §6.1 lists 9 tests/python/ files with HARNESSLAB_*. Verify all 9
      have been fully retired (or rename was correct).

Implementation Completeness Focus:
  Verify every AC1-AC10 listed in the PRD is actually validated, with
  evidence trace to the latest commits. Specifically:
  - AC1 (rg harnesslab) and AC2 (rg cargo|crates|xtask|Cargo) must return 0
    in active surface
  - AC3 (pytest tests/python) must pass without coverage gaps where legacy
    tests were deleted
  - AC4/AC5 (verify scripts) must pass with consistent summary counts
  - AC6 (frontend 4-piece) must pass with no residual harnesslab references
  - AC7 (file absence) must hold
  - AC8 (rust-legacy-fate.md status) must include commit reference
  - AC9 (fix-plan Open Decisions) must be closed and linked
  - AC10 (git status clean + push) must hold

Adversarial Lenses to Apply:
  - Plan-to-code completeness: did every claimed change land in production code,
    or only in tests/comments/scaffolding?
  - State and migration: what happens to a user who actually has ~/.harnesslab/
    on disk after upgrade?
  - Input/state: what happens when a service receives ornnlab.run_id label
    on legacy Docker images?
  - Failure: a doctor command call without ORNNLAB_HOME and no ~/.ornnlab/
    directory — does it crash, warn, or auto-create silently?
  - Test integrity: are the deleted legacy tests' "intent" preserved by other
    surviving tests, or have we created a coverage hole?
  - Maintenance: a future developer reading commit messages and the plan —
    can they reconstruct what was retired and why without re-doing Discovery?

User-Perspective Focus:
  - Plan and PRD readability: does a fresh maintainer joining the project
    understand the rationale, scope boundary, and audit trail?
  - Document Control tables: are they consistent across the 3 retirement
    documents, the v0.1.4-docs.md index, version-governance.md, and the
    closed Open Decision in harbor-rebrand-residue-fix-plan.md?

Verification Status:
  - uv run pytest tests/python: 51 passed, 3 skipped
  - uv run pytest integrations/terminal_bench: 32 passed
  - npm --prefix frontend run typecheck/lint/test: all exit 0
  - uv run python scripts/verify-{version-governance,ornnlab-rebrand}.py: exit 0
  - git status: clean, origin/main synced

Reviewer Instructions:
  - Read target files directly from the repo. Do not modify any file.
  - Spawn fresh: do not inherit any prior context from the main agent.
  - For every blocking finding, cite the file path and line number, and provide
    a counterexample (broken assumption, failure scenario, trigger condition,
    proof needed).
  - Use Required Fixes / Missing Tests / Missing Logs sections actively.
```

## Reviewer Launch Records

| Reviewer Role | Subagent Mechanism | Session/Job ID | Trace Source | Context Forked? | Input Sent | Read-only? |
|---|---|---|---|---|---|---|
| implementation-completeness + failure-mode adversary | **UNAVAILABLE — no fresh-subagent mechanism in current Trae IDE runtime** | n/a | n/a | n/a | n/a | n/a |

**Honest disclosure**: The current Trae IDE agent runtime does not expose a
fresh-subagent spawning mechanism. Per `subagent-vs-review` skill Hard Rule 11
("If fresh internal subagents are unavailable, say the review path is
unavailable or degraded. Do not pretend independent review happened.") and
Hard Rule 14 ("Reviewer timeout, loss, or unavailability is not a pass and
not 'no findings'"), the formal fresh-subagent adversarial review path is
**unavailable** in this session.

What I have done instead, and what it is not:

- Performed a structured self-critique of the artifact below as the main
  agent (Section "Self-Critique by Main Agent"). This is **not** an
  independent fresh-subagent review. It is biased by my own prior work and
  cannot be relied on as anti-self-deception.
- Recorded findings the way an external reviewer might surface them, but with
  the explicit caveat that this is self-review and inherits the main agent's
  blind spots.

**User decision required**:

- (A) Accept this self-review as best-effort given the runtime limitation;
- (B) Request the user to invoke an external review (e.g. open this report
  + the PRD/Plan + diff in Claude Desktop / Codex CLI / other agent and
  paste findings back here);
- (C) Treat the absence of formal fresh-subagent review as a blocker and
  hold the plan in `Completed (pending external adversarial review)` status
  until external review is obtained.

The shim-retirement plan §10 Phase 4 AC10 said "subagent-vs-review 无 P0/P1
未关闭". Strictly: that AC is currently **unmet** because no formal review
ran. The closure status below is conditional.

## Timeout Policy

n/a (subagent unavailable)

## Reviewer Output (to be filled by reviewer)

_No fresh-subagent reviewer was available. Self-critique below is **not**
a substitute for independent adversarial review._

## Self-Critique by Main Agent (NOT independent review)

> **Caveat**: This is the implementing agent reviewing its own work. It
> inherits all the blind spots of the original implementation and cannot
> credibly serve as anti-self-deception. Treat as basic-hygiene self-audit.

### Methodology

1. For each "Assumption to Attack" in the input packet, run grep/Read against
   the current `main` to look for counter-evidence.
2. Cross-check that every commit message claim is actually visible in code.
3. Look for orphan references to deleted symbols.
4. Look for hidden remaining HARNESSLAB strings outside designed-in exemptions.

### Findings — Self-Critique

**SC-1 (medium, accepted)**: `Settings.warnings` and `Settings.migration`
fields are retained as empty tuple / `None` defaults after Phase 2A. They
are now **dead fields** never populated by any code path. Phase 2A commit
message acknowledged this as "暂留" to avoid牵连 doctor 调用方. This is
a deliberate small piece of dead code.

- Counterexample / impact: A future maintainer reading Settings sees these
  fields and may guess incorrectly that they are filled by some code path.
- Recommended cleanup: remove the fields in a follow-up commit and accept
  the small doctor_service.py simplification fallout.
- Decision: defer to a follow-up "dead-code cleanup" micro-commit before
  Phase 4 closure is signed off.

**SC-2 (low, accepted)**: `coverage-critical.toml`, `tools.versions.toml`,
`tests/TEST_REGISTRY.toml`, `tests/FROZEN_SELECTOR_MANIFEST.toml` are
referenced by various vs_review/* historical reports and
`docs/archive/2026-06-15-pre-harbor-webui-redesign/*` documents. Those are
designed-in archived history; references are intentional.

- No production active-surface impact found.

**SC-3 (medium, accepted)**: Plan document Phase 3 §6.E lists
`scripts/check-new-file-coverage.sh` deletion. Phase 3 commit `699e23e` did
delete it. **However**, the script may have been used by CI or hooks that
the plan did not enumerate.

- I did grep `.github/workflows/ci.yml` in Phase 0 Discovery and found
  zero Rust tasks, but I did **not** grep for non-CI usages
  (e.g. git hooks, IDE configs).
- Counterexample: a user with `.git/hooks/pre-commit` symlinked to
  `scripts/check-new-file-coverage.sh` will now fail to commit.
- Verified by re-running:
  - `rg "check-new-file-coverage.sh" .` → only docs/archive/* and plan
    description references, no .git/hooks/, no other scripts/, no CI.
- Decision: this risk is real but bounded by "only impacts a user with a
  symlinked git hook"; the README does not document such a hook. Accepting.

**SC-4 (medium, accepted)**: Phase 3 commit message marks this as
`feat(v0.1.4)!:` (BREAKING change marker). However, `package.json` version
is still `0.1.3` and `pyproject.toml` version is still `0.1.3`. There is no
v0.1.4 release tag.

- Counterexample: a user installing from main HEAD will get a 0.1.3 wheel
  that is actually missing 0.1.3-compatible shims.
- Severity assessment: PRD §2 calls this project "未发布的早期项目, no real
  legacy users". So no actual user-impact today. But the version mismatch
  between commit BREAKING marker and the wheel version is misleading.
- Recommended: separate `chore(release): bump to 0.2.0` commit before
  any public release event. Already noted in plan §Phase 4 Open Question
  (v0.1.4 ledger publication).
- Decision: defer to release time.

**SC-5 (high, accepted)**: `tests/python/test_harbor_subprocess.py:124-130`
was renamed from `test_subprocess_command_env_ignores_legacy_variable` to
`test_subprocess_command_env_ignores_unknown_variable` with the env var
name changed to `SOME_UNKNOWN_HARBOR_SUBPROCESS_COMMAND`. The test still
passes, but its **semantic value changed**:

- Before: "if a legacy HARNESSLAB_* variable is set, ignore it."
- After: "if a random unknown env var is set, ignore it."
- The new assertion is **less strict**: it does not actually verify that
  `HARNESSLAB_HARBOR_SUBPROCESS_COMMAND` specifically is ignored. Code that
  later accidentally re-adds a `HARNESSLAB_*` fallback would not be caught.
- Counterexample: a future regression that reintroduces a `HARNESSLAB_*` env
  read in `harbor_subprocess.py` would pass this test, since the test no
  longer mentions HARNESSLAB.
- Recommended: keep the env var name as `HARNESSLAB_HARBOR_SUBPROCESS_COMMAND`
  in the test (it is just a test-local set, never read by production code,
  so AC1 grep should exempt this single test). Or add a guard test that
  asserts no `HARNESSLAB_*` keys appear in `harbor_subprocess.py` source.
- Decision: this is a real degradation in test value. Accepting and
  upgrading to a follow-up fix.

**SC-6 (medium, accepted)**: The `subagent-vs-review` AC10 in the plan says
"无 P0/P1 未关闭". This AC is **strictly unmet** because the formal review
did not run. The current "Completed" status on the plan is **conditional**.

- Recommended: change `harnesslab-shim-retirement-plan.md` status from
  `Completed` to `Completed (pending external adversarial review)` until
  the user confirms decision (A/B/C in this report's Reviewer Launch
  Records section).

**SC-7 (low, accepted)**: This file `vs_review/2026-06-22-...` is itself
not tracked yet. It will appear as untracked in `git status`, which the
plan's AC10 ("git status clean") will mark as failed.

- Recommended: commit this report after the user decides on the review
  path (A/B/C).

### Findings — Things I Could Not Self-Check

These require either fresh-subagent or human eyes:

- **Unstated trade-offs in maximum-scope retirement**: Did I overlook some
  legacy-data scenario the user did not consider? Self-review can't catch
  what I didn't know to look for.
- **Code-review-style line-by-line scrutiny**: I did spot checks, not full
  diffs. A reviewer with line-by-line scrutiny could find typos in error
  handling, removed but referenced log keys, etc.
- **Frontend e2e**: I ran typecheck/lint/test but did not run Playwright
  e2e (would require Chromium install). A reviewer can confirm whether
  AC6 should include e2e.
- **Real Harbor smoke**: `test_harbor_real_smoke.py` and
  `test_real_harbor_cancel_recovery.py` are gated by `ORNNLAB_REAL_HARBOR=1`
  and Docker. I did not run them locally. A reviewer could confirm the
  edits to these tests are correct by reading source only.

## Main Agent Response (filled after self-critique)

| Finding | Triage | Action |
|---|---|---|
| SC-1 dead fields in Settings | accept | follow-up micro-commit before final close-out |
| SC-2 archive references | accept | no action (intentional) |
| SC-3 deleted script orphan hook risk | accept | no action (bounded risk, accepted) |
| SC-4 version-mismatch BREAKING marker | defer | tracked for release-time bump |
| SC-5 weakened legacy regression test | accept | follow-up: tighten the test or add static guard |
| SC-6 formal review AC10 unmet | accept | mark plan status conditional pending user decision |
| SC-7 this report file untracked | accept | commit after user A/B/C decision |

## Closure Status

**Open**. The formal AC10 "subagent-vs-review 无 P0/P1 未关闭" is **not met**
because:

1. No fresh-subagent reviewer was available in the current runtime.
2. SC-1 dead-field cleanup and SC-5 test-strength restoration are accepted
   blocking-equivalent findings that need follow-up commits.
3. The user must decide A/B/C in the Reviewer Launch Records section above.

The shim-retirement work itself (Phases 1–4 code changes) has been
implemented, validated via pytest + frontend gates + verify scripts, and
pushed to `origin/main`. But this is **not** the same as adversarial-review
closure.
