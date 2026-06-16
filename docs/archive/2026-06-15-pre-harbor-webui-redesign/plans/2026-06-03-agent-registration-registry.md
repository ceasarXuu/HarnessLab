# Agent Registration Registry Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a first-class, low-friction Agent Registry so users or helper agents can register CLI harnesses through a readable, validated, reproducible profile instead of hand-writing sandbox shell.

**Architecture:** Add explicit profile schema for setup, skills, tools, and hooks in `harnesslab-core`; add a CLI-side Agent Registry materializer that expands semantic profile fields into runtime setup, capability policy, summaries, and snapshots; wire the materialized runtime into sandbox, Terminal-Bench bridge, doctor, init templates, and report output.

**Tech Stack:** Rust workspace (`harnesslab-core`, `harnesslab-cli`, `harnesslab-report`), TOML/Serde config, clap CLI, JSON doctor output, Askama HTML report, Python Terminal-Bench bridge tests, `cargo test`/`nextest`, existing `scripts/test-after-change.sh`.

---

## Scope

This plan implements the product contract from:

- `docs/archive/stubs/prd.md` section `11. Agent Profile 配置体验`
- `docs/archive/stubs/architecture.md` section `5.3 Agent Registry`
- `docs/archive/stubs/mvp-development-spec.md` section `6.2 Agent Profile Schema`
- `docs/playbooks/terminal-bench-claude-ds.md` section `2. 注册 claude-ds agent`

In scope:

- Readable agent registration schema.
- `setup`, `skills`, `tools`, and `hooks` fields in `AgentProfile`.
- Per-kind validation and materialization boundary.
- Doctor checks with field paths, allowed values, and suggested fixes.
- Commented `init` profile templates and registry reference.
- Runtime snapshots and HTML report summaries.
- Terminal-Bench bridge support for materialized setup.
- Tests and traceability updates.

Out of scope:

- GUI profile editor.
- Remote registry marketplace.
- Per-agent vendor-perfect tools/hooks integration for every CLI on day one.
- Ranking changes or benchmark scoring changes.

## Design Decisions

1. `sandbox_setup_command` remains only as legacy compatibility under `labels`; new profiles should use `[setup]`.
2. The schema accepts `skills/tools/hooks` policies for all `kind`s, but doctor blocks run when a non-default policy cannot be materialized for that `kind`.
3. `allow` and `deny` conflict is a hard error. No precedence guessing.
4. `init` should create human-readable TOML with comments, not only machine-serialized TOML.
5. A helper agent should be able to run `harnesslab agent schema --json`, generate a profile, then run `harnesslab doctor --json` without starting a benchmark.
6. Run artifacts must preserve both original profile snapshot and effective materialized summary.

## Target User Flow

Minimal user path after implementation:

```bash
harnesslab init
harnesslab agent schema --json > /tmp/harnesslab-agent-schema.json
$EDITOR ~/.harnesslab/agents/claude-ds.toml
harnesslab doctor --json
harnesslab run --agent claude-ds --benchmark terminal-bench --split smoke --json
```

Target `claude-ds` profile shape:

```toml
schema_version = 1
name = "claude-ds"
kind = "claude-code"
display_name = "Claude Code via DeepSeek API"
command = "claude-ds -p --bare --output-format text"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 300

[auth]
inherit = true
inherit_env = [
  "ANTHROPIC_BASE_URL",
  "ANTHROPIC_AUTH_TOKEN",
  "ANTHROPIC_MODEL",
]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "builtin"
required_commands = ["claude", "claude-ds"]
run_as = "harnesslab"
commands = []

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
model = "deepseek"
terminal_bench_agent_import_path = "harnesslab_tb_agent:HarnessLabCommandAgent"
terminal_bench_agent_pythonpath = "/absolute/path/to/HarnessLab/integrations/terminal_bench"
```

## Task 1: Test Traceability For Agent Registry

**Files:**

- Modify: `tests/REQUIREMENTS.toml`
- Modify: `tests/TEST_REGISTRY.toml`

**Step 1: Add requirements before code**

Add requirements for:

- `AGT-REG-001`: profile schema supports setup/skills/tools/hooks.
- `AGT-REG-002`: doctor reports field-level validation errors with allowed values and suggested fixes.
- `AGT-REG-003`: init writes readable commented profile templates.
- `AGT-REG-004`: materialized runtime summary is saved and shown in report.
- `AGT-REG-005`: Terminal-Bench bridge receives materialized setup.
- `AGT-REG-006`: non-materializable non-default policies block run before benchmark execution.

**Step 2: Add placeholder registry entries**

Add matching `TEST_REGISTRY.toml` entries with intended test files:

- `crates/harnesslab-core/src/config_tests.rs`
- `crates/harnesslab-cli/tests/init_contract.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `crates/harnesslab-cli/tests/run_output_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_contract.rs`
- `integrations/terminal_bench/harnesslab_tb_agent_test.py`
- `crates/harnesslab-report/src/lib.rs`

**Step 3: Run registry validation**

Run:

```bash
scripts/verify-test-registry.sh
```

Expected:

- Either pass, or fail only because newly planned tests are not implemented yet. If it fails, keep the failure message as a checklist for later tasks.

**Step 4: Commit**

```bash
git add tests/REQUIREMENTS.toml tests/TEST_REGISTRY.toml
git commit -m "test: register agent registry requirements"
```

## Task 2: Core Schema For Setup And Capability Policies

**Files:**

- Create: `crates/harnesslab-core/src/agent_profile.rs`
- Modify: `crates/harnesslab-core/src/lib.rs`
- Modify: `crates/harnesslab-core/src/config.rs`
- Test: `crates/harnesslab-core/src/config_tests.rs`

**Step 1: Write failing schema tests**

Add tests:

```rust
#[test]
fn agt_reg_001_profile_deserializes_setup_skills_tools_hooks() {
    let profile: AgentProfile = toml::from_str(r#"
schema_version = 1
name = "claude-ds"
kind = "claude-code"
display_name = "Claude DS"
command = "claude-ds -p"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 300

[auth]
inherit = true
inherit_env = ["ANTHROPIC_AUTH_TOKEN"]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "builtin"
required_commands = ["claude", "claude-ds"]
run_as = "harnesslab"
commands = []

[skills]
inherit = true
allow = ["skill-a"]
deny = ["skill-b"]
include_paths = ["~/.claude/skills"]

[tools]
inherit = true
allow = []
deny = ["web_search"]

[hooks]
inherit = false
allow = []
deny = []

[usage]
parser = "none"
"#).unwrap();

    assert_eq!(profile.setup.preset, SetupPreset::Builtin);
    assert_eq!(profile.setup.run_as, RunAs::Harnesslab);
    assert_eq!(profile.skills.allow, vec!["skill-a"]);
    assert_eq!(profile.tools.deny, vec!["web_search"]);
    assert!(!profile.hooks.inherit);
}
```

Add validation tests:

- `setup.commands` with `preset = "builtin"` returns an error.
- `setup.required_commands = ["claude | sh"]` returns an error.
- same item in `skills.allow` and `skills.deny` returns an error.
- same item in `tools.allow` and `tools.deny` returns an error.
- same item in `hooks.allow` and `hooks.deny` returns an error.
- old profiles without new sections deserialize with defaults.

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p harnesslab-core agt_reg_001 -- --nocapture
```

Expected:

- Fails because `AgentProfile` has no `setup`, `skills`, `tools`, or `hooks` fields.

**Step 3: Implement schema types**

Create `crates/harnesslab-core/src/agent_profile.rs`:

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupConfig {
    #[serde(default)]
    pub preset: SetupPreset,
    #[serde(default)]
    pub required_commands: Vec<String>,
    #[serde(default)]
    pub run_as: RunAs,
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SetupPreset {
    None,
    #[default]
    Builtin,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RunAs {
    Root,
    #[default]
    Harnesslab,
    Current,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    #[serde(default = "default_true")]
    pub inherit: bool,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub include_paths: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for SetupConfig {
    fn default() -> Self {
        Self {
            preset: SetupPreset::Builtin,
            required_commands: Vec::new(),
            run_as: RunAs::Harnesslab,
            commands: Vec::new(),
        }
    }
}

impl Default for CapabilityPolicy {
    fn default() -> Self {
        Self {
            inherit: true,
            allow: Vec::new(),
            deny: Vec::new(),
            include_paths: Vec::new(),
        }
    }
}
```

Use separate structs for tools/hooks if `include_paths` should not appear in tools/hooks serialization. Recommended simple route for MVP:

- Reuse `CapabilityPolicy`.
- Document that `include_paths` is only honored for skills.
- Doctor warns if tools/hooks include paths are non-empty.

**Step 4: Extend `AgentProfile`**

In `crates/harnesslab-core/src/config.rs`, add fields with serde defaults:

```rust
#[serde(default)]
pub setup: SetupConfig,
#[serde(default)]
pub skills: CapabilityPolicy,
#[serde(default)]
pub tools: CapabilityPolicy,
#[serde(default)]
pub hooks: CapabilityPolicy,
```

Update `default_agent_profile` to populate:

- `setup.required_commands` based on `kind`.
- `setup.preset = Builtin` for built-ins.
- `setup.preset = None` for `Fake`.
- empty capability policies with `inherit = true`.

**Step 5: Extend validation**

Add `ConfigError` variants with field path and accepted values:

```rust
#[error("invalid {field}: {message}; accepted={accepted}")]
InvalidField {
    field: &'static str,
    message: String,
    accepted: &'static str,
}
```

Validation rules:

- `setup.commands` non-empty is valid only for `SetupPreset::Custom`.
- `required_commands` entries must match `[A-Za-z0-9._+-]+`.
- `allow` and `deny` must not overlap for each policy.
- capability names must be non-empty and must not contain path separators unless they are under `skills.include_paths`.

**Step 6: Run targeted core tests**

Run:

```bash
cargo test -p harnesslab-core agt_reg_001 -- --nocapture
cargo test -p harnesslab-core cfg_ -- --nocapture
```

Expected:

- New schema tests pass.
- Existing config tests pass.

**Step 7: Commit**

```bash
git add crates/harnesslab-core/src/agent_profile.rs crates/harnesslab-core/src/lib.rs crates/harnesslab-core/src/config.rs crates/harnesslab-core/src/config_tests.rs
git commit -m "feat: add semantic agent profile schema"
```

## Task 3: Agent Profile Reference Output

**Files:**

- Create: `crates/harnesslab-core/src/agent_profile_reference.rs`
- Modify: `crates/harnesslab-core/src/lib.rs`
- Modify: `crates/harnesslab-cli/src/lib.rs`
- Modify: `crates/harnesslab-cli/src/app.rs`
- Modify: `crates/harnesslab-cli/src/output.rs`
- Test: `crates/harnesslab-cli/tests/cli_contract.rs`

**Step 1: Write failing CLI test**

Add a test:

```rust
#[test]
fn int_agent_schema_json_exposes_profile_field_ranges() {
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["agent", "schema", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "agent schema");
    assert!(json["fields"].as_array().unwrap().iter().any(|field| {
        field["path"] == "setup.preset"
            && field["allowed_values"].as_array().unwrap().contains(&serde_json::json!("builtin"))
    }));
    assert!(json["fields"].as_array().unwrap().iter().any(|field| {
        field["path"] == "skills.allow"
            && field["example"] == serde_json::json!(["skill-a"])
    }));
}
```

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p harnesslab-cli int_agent_schema_json_exposes_profile_field_ranges -- --nocapture
```

Expected:

- Fails because `agent schema` does not exist.

**Step 3: Implement reference model**

Create `agent_profile_reference.rs` with:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct AgentProfileFieldReference {
    pub path: &'static str,
    pub required: bool,
    pub allowed_values: &'static [&'static str],
    pub example: serde_json::Value,
    pub description: &'static str,
}

pub fn agent_profile_field_reference() -> Vec<AgentProfileFieldReference> {
    vec![
        field("schema_version", true, &["1"], json!(1), "Profile schema version."),
        field("kind", true, &["codex", "claude-code", "opencode", "pi-coding-agent", "custom", "fake"], json!("claude-code"), "Agent harness kind."),
        field("setup.preset", false, &["none", "builtin", "custom"], json!("builtin"), "Sandbox setup mode."),
        field("skills.allow", false, &[], json!(["skill-a"]), "Skill whitelist."),
        field("skills.deny", false, &[], json!(["skill-b"]), "Skill blacklist."),
        field("tools.allow", false, &[], json!(["edit", "bash"]), "Tool whitelist."),
        field("hooks.deny", false, &[], json!(["pre_tool_use"]), "Hook blacklist."),
    ]
}
```

Keep the list complete enough that a helper agent can generate valid TOML without reading docs.

**Step 4: Add CLI command**

In `crates/harnesslab-cli/src/lib.rs`, extend:

```rust
pub(crate) enum AgentCommand {
    List { #[arg(long)] json: bool },
    Schema { #[arg(long)] json: bool },
}
```

In `app.rs`, dispatch `AgentCommand::Schema`.

Text output should be compact:

```text
Agent profile fields:
  - setup.preset: none | builtin | custom
  - skills.allow: string[]
  - skills.deny: string[]
```

JSON output should be stable.

**Step 5: Run CLI test**

Run:

```bash
cargo test -p harnesslab-cli int_agent_schema_json_exposes_profile_field_ranges -- --nocapture
```

Expected:

- Pass.

**Step 6: Commit**

```bash
git add crates/harnesslab-core/src/agent_profile_reference.rs crates/harnesslab-core/src/lib.rs crates/harnesslab-cli/src/lib.rs crates/harnesslab-cli/src/app.rs crates/harnesslab-cli/src/output.rs crates/harnesslab-cli/tests/cli_contract.rs
git commit -m "feat: expose agent profile schema reference"
```

## Task 4: Commented Init Templates And Agents README

**Files:**

- Create: `crates/harnesslab-cli/src/agent_registry/templates.rs`
- Create: `crates/harnesslab-cli/src/agent_registry/mod.rs`
- Modify: `crates/harnesslab-cli/src/app.rs`
- Test: `crates/harnesslab-cli/tests/init_contract.rs`

**Step 1: Write failing init test**

Extend `int_001_init_empty_home_creates_config_and_profiles`:

```rust
let claude = fs::read_to_string(home.path().join("agents/claude-code-default.toml")).unwrap();
assert!(claude.contains("# HarnessLab agent profile"));
assert!(claude.contains("[setup]"));
assert!(claude.contains("[skills]"));
assert!(claude.contains("allow = []"));
assert!(home.path().join("agents/README.md").exists());
let readme = fs::read_to_string(home.path().join("agents/README.md")).unwrap();
assert!(readme.contains("harnesslab agent schema --json"));
```

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p harnesslab-cli int_001_init_empty_home_creates_config_and_profiles -- --nocapture
```

Expected:

- Fails because init currently writes uncommented serialized TOML and no README.

**Step 3: Implement templates**

Create `templates.rs` with functions:

```rust
pub(crate) fn profile_template(kind: AgentKind, name: &str, command: &str) -> String;
pub(crate) fn agents_readme() -> &'static str;
```

Template requirements:

- Include comments for every section.
- Include `[setup]`, `[skills]`, `[tools]`, `[hooks]`.
- Do not include `labels.sandbox_setup_command` in default templates.
- Include `terminal_bench_agent_import_path` comments only as optional benchmark adapter label.
- Keep `pi-coding-agent-default` command and version command compatible with existing tests.

**Step 4: Switch init**

In `app.rs`, replace `toml::to_string_pretty(&profile)` with template output for built-in profiles.

Important:

- Existing user profile files must not be overwritten.
- `agents/README.md` should be written if missing.
- `agent list` must still load templates through Serde.

**Step 5: Run tests**

Run:

```bash
cargo test -p harnesslab-cli init_contract -- --nocapture
cargo test -p harnesslab-cli cli_contract -- --nocapture
```

Expected:

- Init tests pass.
- Agent list and load still pass.

**Step 6: Commit**

```bash
git add crates/harnesslab-cli/src/agent_registry crates/harnesslab-cli/src/app.rs crates/harnesslab-cli/tests/init_contract.rs
git commit -m "feat: generate readable agent profile templates"
```

## Task 5: Doctor Field-Level Validation And Suggestions

**Files:**

- Modify: `crates/harnesslab-core/src/config.rs`
- Modify: `crates/harnesslab-core/src/agent_profile.rs`
- Modify: `crates/harnesslab-cli/src/doctor.rs`
- Test: `crates/harnesslab-cli/tests/doctor_contract.rs`

**Step 1: Write failing doctor tests**

Add tests:

```rust
#[test]
fn doc_agent_registry_reports_setup_and_policy_field_paths() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    fs::write(home.path().join("agents/bad-registry.toml"), r#"
schema_version = 1
name = "bad-registry"
kind = "custom"
display_name = "Bad Registry"
command = "sh"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 1

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "builtin"
required_commands = ["sh | cat"]
run_as = "harnesslab"
commands = ["echo not allowed"]

[skills]
inherit = true
allow = ["a"]
deny = ["a"]
include_paths = []

[tools]
inherit = true
allow = ["bash"]
deny = ["bash"]

[hooks]
inherit = true
allow = []
deny = []

[usage]
parser = "none"
"#).unwrap();

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .args(["--home", home.path().to_str().unwrap(), "doctor", "--json"])
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let check = find_check(&json, "agent.bad-registry.validation");
    assert_eq!(check["status"], "error");
    assert!(check["details"]["errors"].as_array().unwrap().iter().any(|error| {
        error["field"] == "setup.commands"
            && error["accepted_values"].as_array().unwrap().contains(&serde_json::json!("custom"))
    }));
    assert!(check["details"]["errors"].as_array().unwrap().iter().any(|error| {
        error["field"] == "skills.allow"
            && error["suggested_fix"].as_str().unwrap().contains("remove duplicate")
    }));
}
```

Add test for advanced setup warning:

- `setup.preset = "custom"` and non-empty commands produces warning `agent.<name>.setup.advanced`.

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p harnesslab-cli doc_agent_registry -- --nocapture
```

Expected:

- Fails because current doctor details only contain a single string error.

**Step 3: Implement validation details**

Add a structured validation error type:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileValidationError {
    pub field: &'static str,
    pub message: String,
    pub accepted_values: Vec<&'static str>,
    pub suggested_fix: String,
}
```

Keep the old `profile.validate()` return shape only if needed for compatibility. Preferred:

- Add `profile.validation_report()` returning errors + warnings.
- Make `profile.validate()` call it and preserve existing tests.

**Step 4: Update doctor output**

In `append_profile_checks`:

- If errors exist: status `error`, severity `error`, details `{ "errors": [...] }`.
- If warnings only: status `warning`, details `{ "warnings": [...] }`.
- Add `agent.<name>.setup.advanced` warning for `setup.preset = custom`.
- Add `agent.<name>.capabilities.materialization` error for unsupported non-default policies.

**Step 5: Run doctor tests**

Run:

```bash
cargo test -p harnesslab-cli doctor_contract -- --nocapture
```

Expected:

- Existing doctor tests still pass.
- New field-level validation tests pass.

**Step 6: Commit**

```bash
git add crates/harnesslab-core/src/config.rs crates/harnesslab-core/src/agent_profile.rs crates/harnesslab-cli/src/doctor.rs crates/harnesslab-cli/tests/doctor_contract.rs
git commit -m "feat: add field-level agent profile diagnostics"
```

## Task 6: Agent Registry Materializer

**Files:**

- Create: `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- Modify: `crates/harnesslab-cli/src/agent_registry/mod.rs`
- Modify: `crates/harnesslab-cli/src/lib.rs` or `crates/harnesslab-cli/src/app.rs` module declarations
- Test: `crates/harnesslab-cli/src/runner_tests.rs`
- Test: `crates/harnesslab-cli/tests/doctor_contract.rs`

**Step 1: Write failing materializer tests**

Add unit tests for:

- Codex builtin setup command generated from `[setup]`, not `labels.sandbox_setup_command`.
- Claude Code builtin setup creates `claude-ds` wrapper when `required_commands` contains `claude-ds`.
- Fake profile with non-default tools deny fails materialization.
- Custom profile with default capabilities and `setup.preset = none` materializes.
- Custom profile with `setup.preset = custom` returns setup commands joined with `&&` or explicit newline-safe script.

Example:

```rust
#[test]
fn agt_reg_004_materializer_blocks_non_default_tools_for_custom() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
    profile.tools.deny = vec!["bash".to_string()];

    let error = materialize_profile(&profile).unwrap_err();

    assert_eq!(error.field, "tools");
    assert!(error.message.contains("not supported for kind custom"));
}
```

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p harnesslab-cli agt_reg_004 -- --nocapture
```

Expected:

- Fails because materializer does not exist.

**Step 3: Implement materializer types**

Create:

```rust
pub(crate) struct MaterializedAgentProfile {
    pub setup_script: Option<String>,
    pub setup_summary: String,
    pub skills_summary: String,
    pub tools_summary: String,
    pub hooks_summary: String,
    pub warnings: Vec<String>,
}

pub(crate) fn materialize_profile(profile: &AgentProfile) -> Result<MaterializedAgentProfile, MaterializationError>;
```

Rules:

- `setup.preset = none`: no setup script.
- `setup.preset = builtin`: generate kind-specific installer/wrapper script.
- `setup.preset = custom`: use `setup.commands`, but mark warning `advanced_custom_setup`.
- Legacy `labels.sandbox_setup_command`: accepted only if no `[setup]` custom commands; mark warning `legacy_sandbox_setup_command`.
- `skills/tools/hooks` default policy materializes to summary only.
- Non-default policy materialization support matrix:
  - `claude-code`: initially allow skills include/deny summary and required env/path checks; tools/hooks non-default block until adapter-specific implementation exists.
  - `codex`: initially block non-default skills/tools/hooks unless adapter implemented.
  - `opencode`: initially block non-default skills/tools/hooks unless adapter implemented.
  - `pi-coding-agent`: initially block non-default skills/tools/hooks unless adapter implemented.
  - `custom`: block non-default skills/tools/hooks.

This is strict by design: unsupported capability policy should not silently produce invalid comparisons.

**Step 4: Integrate with doctor**

Doctor should call `materialize_profile(profile)` after syntactic validation:

- materialization error -> `agent.<name>.capabilities.materialization` error.
- materialization warning -> warning check.
- setup summary -> details.

**Step 5: Run tests**

Run:

```bash
cargo test -p harnesslab-cli agt_reg_004 -- --nocapture
cargo test -p harnesslab-cli doctor_contract -- --nocapture
```

Expected:

- Materializer tests pass.
- Doctor reports unsupported policies before run.

**Step 6: Commit**

```bash
git add crates/harnesslab-cli/src/agent_registry crates/harnesslab-cli/src/runner_tests.rs crates/harnesslab-cli/tests/doctor_contract.rs
git commit -m "feat: materialize agent registry profiles"
```

## Task 7: Runtime Snapshot And Report Summary

**Files:**

- Modify: `crates/harnesslab-cli/src/runner/store.rs`
- Modify: `crates/harnesslab-cli/src/runner.rs`
- Modify: `crates/harnesslab-report/src/lib.rs`
- Test: `crates/harnesslab-cli/tests/run_output_contract.rs`
- Test: `crates/harnesslab-report/src/lib.rs`

**Step 1: Write failing tests**

Run output contract should assert:

- `agent-runtime.materialized.json` exists.
- `command.txt` references materialized snapshot.
- `agent-profile.snapshot.json` still exists and is redacted.

Report test should assert HTML contains:

- `Setup: builtin`
- `Skills: inherit=true`
- `Tools: inherit=true`
- `Hooks: inherit=true`

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p harnesslab-cli run_output_contract -- --nocapture
cargo test -p harnesslab-report rpt_001 -- --nocapture
```

Expected:

- Fails because materialized snapshot and report fields do not exist.

**Step 3: Persist materialized runtime**

Add constant:

```rust
pub(super) const MATERIALIZED_PROFILE_SNAPSHOT: &str = "agent-runtime.materialized.json";
```

Write it in `write_run_inputs`.

Security:

- Redact secrets from setup script and command.
- `agent-profile.runtime.json` remains `0600`.
- Materialized public snapshot should not include raw secret values.

**Step 4: Extend report context**

Add to `ReportContext`:

```rust
pub setup_summary: String,
pub skills_summary: String,
pub tools_summary: String,
pub hooks_summary: String,
```

Render these in HTML near Agent config.

**Step 5: Update agent config summary**

`store::agent_config_summary` should include:

- kind
- input_mode
- setup summary
- skills summary
- tools summary
- hooks summary
- command template

Keep it concise.

**Step 6: Run tests**

Run:

```bash
cargo test -p harnesslab-cli run_output_contract -- --nocapture
cargo test -p harnesslab-report -- --nocapture
```

Expected:

- Pass.

**Step 7: Commit**

```bash
git add crates/harnesslab-cli/src/runner/store.rs crates/harnesslab-cli/src/runner.rs crates/harnesslab-report/src/lib.rs crates/harnesslab-cli/tests/run_output_contract.rs
git commit -m "feat: persist materialized agent runtime"
```

## Task 8: Sandbox Runner Uses Materialized Setup

**Files:**

- Modify: `crates/harnesslab-cli/src/runner/sandbox.rs`
- Modify: `crates/harnesslab-cli/src/runner.rs`
- Test: `crates/harnesslab-cli/src/runner_tests.rs`
- Test: `crates/harnesslab-cli/tests/external_smoke_contract.rs` if needed

**Step 1: Write failing sandbox tests**

Add tests:

- Builtin setup comes from materializer.
- Legacy `labels.sandbox_setup_command` still works but produces warning.
- Empty setup means no prefix.

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p harnesslab-cli sandbox_setup -- --nocapture
```

Expected:

- Existing code still reads only labels, so tests fail.

**Step 3: Pass materialized setup through execution**

Refactor `execute_plan` and `run_agent` signatures to receive `MaterializedAgentProfile`.

Do not recompute materialization per task. Compute once after profile load and validation.

**Step 4: Replace `docker_setup_command`**

In `sandbox.rs`:

- Remove direct `profile.labels.get("sandbox_setup_command")` lookup.
- Use `materialized.setup_script`.
- Keep a tiny helper for prefixing command.

**Step 5: Log setup**

When setup is non-empty:

- Write setup script to `attempt/agent/setup.sh`.
- Log start/end events in `events.jsonl`.
- If setup fails, classify as `execution/external_runner_setup_failed`.

**Step 6: Run tests**

Run:

```bash
cargo test -p harnesslab-cli runner_tests -- --nocapture
cargo test -p harnesslab-cli external_smoke_contract -- --nocapture
```

Expected:

- Existing fake-terminal and smoke tests pass.

**Step 7: Commit**

```bash
git add crates/harnesslab-cli/src/runner.rs crates/harnesslab-cli/src/runner/sandbox.rs crates/harnesslab-cli/src/runner_tests.rs crates/harnesslab-cli/tests/external_smoke_contract.rs
git commit -m "feat: run sandbox agents from materialized setup"
```

## Task 9: Terminal-Bench Bridge Uses Materialized Setup

**Files:**

- Modify: `crates/harnesslab-cli/src/runner/external/terminal_bench_env.rs`
- Modify: `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- Modify: `integrations/terminal_bench/harnesslab_tb_agent.py`
- Test: `crates/harnesslab-cli/tests/terminal_bench_contract.rs`
- Test: `integrations/terminal_bench/harnesslab_tb_agent_test.py`

**Step 1: Write failing Rust env test**

Extend existing Terminal-Bench env tests:

```rust
assert!(terminal_bench_agent_env(&profile, 3600, &materialized)
    .contains("HARNESSLAB_AGENT_SETUP_COMMAND"));
```

**Step 2: Write failing Python bridge test**

Add test:

```python
def test_runs_setup_command_before_agent(tmp_path, monkeypatch):
    marker = tmp_path / "setup-marker"
    monkeypatch.setenv("HARNESSLAB_AGENT_SETUP_COMMAND", f"touch {shlex.quote(str(marker))}")
    monkeypatch.setenv("HARNESSLAB_AGENT_COMMAND", "python -c 'print(\"echo ok\")'")
    monkeypatch.setenv("HARNESSLAB_AGENT_INPUT_MODE", "stdin")
    monkeypatch.setenv("HARNESSLAB_AGENT_TIMEOUT_SEC", "5")

    output = run_registered_agent_from_test_helper()

    assert marker.exists()
```

Exact helper can follow existing `harnesslab_tb_agent_test.py` style.

**Step 3: Run tests to verify failure**

Run:

```bash
cargo test -p harnesslab-cli terminal_bench_agent_env -- --nocapture
python -m pytest integrations/terminal_bench/harnesslab_tb_agent_test.py -q
```

Expected:

- Fails because setup env and bridge execution do not exist.

**Step 4: Export setup env**

Update `terminal_bench_agent_env` to include:

- `HARNESSLAB_AGENT_SETUP_COMMAND`
- `HARNESSLAB_AGENT_SETUP_SUMMARY`
- `HARNESSLAB_AGENT_SKILLS_SUMMARY`
- `HARNESSLAB_AGENT_TOOLS_SUMMARY`
- `HARNESSLAB_AGENT_HOOKS_SUMMARY`

Use shell quoting and redaction rules.

**Step 5: Run setup in Python bridge**

In `harnesslab_tb_agent.py`:

- Before `run_registered_agent`, run setup command once per `perform_task`.
- Log `agent_setup_stdout.log`, `agent_setup_stderr.log`, `agent_setup_error.log`.
- If setup exits non-zero, return `FailureMode.UNKNOWN_AGENT_ERROR` with explicit log.
- Add log text that HarnessLab result parser can map to `external_runner_setup_failed`.

**Step 6: Map setup failure**

In `terminal_bench_result` or log scanner:

- Detect bridge setup failure marker.
- Override official `unknown_agent_error` to `execution/external_runner_setup_failed`.

**Step 7: Run tests**

Run:

```bash
cargo test -p harnesslab-cli terminal_bench_contract -- --nocapture
cargo test -p harnesslab-cli terminal_bench_failure_contract -- --nocapture
python -m pytest integrations/terminal_bench/harnesslab_tb_agent_test.py -q
```

Expected:

- All pass.

**Step 8: Commit**

```bash
git add crates/harnesslab-cli/src/runner/external/terminal_bench_env.rs crates/harnesslab-cli/src/runner/external/terminal_bench.rs integrations/terminal_bench/harnesslab_tb_agent.py crates/harnesslab-cli/tests/terminal_bench_contract.rs integrations/terminal_bench/harnesslab_tb_agent_test.py
git commit -m "feat: pass materialized setup to terminal-bench bridge"
```

## Task 10: Run Precheck Blocks Unsupported Registration Policies

**Files:**

- Modify: `crates/harnesslab-cli/src/runner.rs`
- Modify: `crates/harnesslab-cli/src/doctor.rs`
- Test: `crates/harnesslab-cli/tests/benchmark_contract.rs`
- Test: `crates/harnesslab-cli/tests/doctor_contract.rs`

**Step 1: Write failing run test**

Create profile:

- `kind = "custom"`
- `tools.deny = ["bash"]`

Run fake benchmark:

```bash
harnesslab --home <home> run --agent bad-tools --benchmark fake-terminal --split success --json
```

Expected:

- Exit code `3`.
- No run directory created, or run directory contains precheck failure only.
- Error contains `tools` and `not supported for kind custom`.

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p harnesslab-cli non_default_policy_blocks_run -- --nocapture
```

Expected:

- Fails because current runner does not know capability materialization.

**Step 3: Implement pre-run materialization**

In `execute_new_run`, after loading and syntactic validation:

- `let materialized = materialize_profile(&profile)?;`
- If materialization returns blocking error, bail before creating run dir.
- If warnings, add them to run inputs or doctor only; for run, log warnings into `events.jsonl` after run dir exists.

In `resume`/`replay`:

- Load materialized snapshot if present.
- If old run lacks snapshot, materialize from runtime profile and write a `profile_materialized_from_legacy` event.

**Step 4: Run tests**

Run:

```bash
cargo test -p harnesslab-cli benchmark_contract -- --nocapture
cargo test -p harnesslab-cli replay_contract -- --nocapture
cargo test -p harnesslab-cli resume_contract -- --nocapture
```

Expected:

- New precheck test passes.
- Replay/resume compatibility preserved.

**Step 5: Commit**

```bash
git add crates/harnesslab-cli/src/runner.rs crates/harnesslab-cli/src/doctor.rs crates/harnesslab-cli/tests/benchmark_contract.rs crates/harnesslab-cli/tests/replay_contract.rs crates/harnesslab-cli/tests/resume_contract.rs
git commit -m "feat: block unsupported agent registry policies before run"
```

## Task 11: Usage In Playbook And Docs

**Files:**

- Modify: `docs/playbooks/terminal-bench-claude-ds.md`
- Create or modify: `docs/archive/stubs/agent-profile-reference.md`
- Modify: `docs/playbooks/development-operations.md`
- Modify: `docs/archive/stubs/prd.md` only if implementation changes product contract

**Step 1: Update claude-ds playbook**

Replace the main example profile to use:

- `[setup]`
- `[skills]`
- `[tools]`
- `[hooks]`

Move `labels.sandbox_setup_command` to a clearly marked legacy appendix, or remove it if materialized setup fully replaces it.

**Step 2: Add field reference doc**

`docs/archive/stubs/agent-profile-reference.md` should include:

- Full TOML schema.
- Field table with required flag, allowed values, default, example.
- Capability policy truth table.
- Per-kind materialization support matrix.
- Examples:
  - default codex
  - `claude-ds`
  - Claude Code with skill whitelist
  - custom agent with no inherited tools/hooks

**Step 3: Validate docs**

Run:

```bash
git diff --check -- docs/playbooks/terminal-bench-claude-ds.md docs/archive/stubs/agent-profile-reference.md docs/playbooks/development-operations.md
```

Expected:

- No whitespace errors.

**Step 4: Commit**

```bash
git add docs/playbooks/terminal-bench-claude-ds.md docs/archive/stubs/agent-profile-reference.md docs/playbooks/development-operations.md docs/archive/stubs/prd.md
git commit -m "docs: document semantic agent registration"
```

## Task 12: Focused Regression And Coverage

**Files:**

- No new files unless tests reveal a missing helper.

**Step 1: Run focused Rust tests**

Run:

```bash
cargo test -p harnesslab-core -- --nocapture
cargo test -p harnesslab-cli doctor_contract -- --nocapture
cargo test -p harnesslab-cli init_contract -- --nocapture
cargo test -p harnesslab-cli run_output_contract -- --nocapture
cargo test -p harnesslab-report -- --nocapture
```

Expected:

- All pass.

**Step 2: Run Python bridge tests**

Run:

```bash
python -m pytest integrations/terminal_bench -q
```

Expected:

- All pass.

**Step 3: Run project gate**

Run:

```bash
scripts/test-after-change.sh
```

Expected:

- All tests pass.
- Coverage gate remains `>= 95%` line coverage and configured branch/critical thresholds.

**Step 4: Run traceability**

Run:

```bash
scripts/verify-test-registry.sh
scripts/generate-test-traceability.sh
```

Expected:

- Registry valid.
- Traceability generated without missing requirements.

**Step 5: Commit any test/doc fixes**

```bash
git add .
git commit -m "test: validate agent registration registry"
```

Only commit if this task produced additional changes.

## Task 13: Real MVP Validation With claude-ds

**Files:**

- No code files.
- Possible doc update: `docs/playbooks/development-operations.md` if new operational lesson is found.

**Step 1: Create or update a real `claude-ds` profile**

Use the new semantic schema under `.benchmarks/_harnesslab-home-terminal-real/agents/claude-ds.toml` or a fresh temp home outside git.

**Step 2: Doctor**

Run:

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks \
  target/debug/harnesslab --home .benchmarks/_harnesslab-home-terminal-real doctor --json
```

Expected:

- No profile validation error.
- No unsupported capabilities materialization error.
- Docker and benchmark data checks reflect current machine state.

**Step 3: Terminal-Bench smoke**

Run:

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks \
  target/debug/harnesslab --home .benchmarks/_harnesslab-home-terminal-real run \
  --agent claude-ds \
  --benchmark terminal-bench \
  --split smoke \
  --concurrency 1 \
  --timeout-sec 1800 \
  --json
```

Expected:

- Run completes through HarnessLab CLI.
- `agent-runtime.materialized.json` exists.
- `report.html` shows setup/skills/tools/hooks summary.
- No `execution_failure`.

**Step 4: Optional targeted full subset**

If smoke passes, run the existing two-QEMU subset or a small subset root:

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks/_terminal-bench-two-qemu-root-20260602T164020Z \
  target/debug/harnesslab --home .benchmarks/_harnesslab-home-terminal-real run \
  --agent claude-ds \
  --benchmark terminal-bench \
  --split full \
  --concurrency 1 \
  --timeout-sec 1800 \
  --json
```

Expected:

- Execution failures remain zero.
- Benchmark failures, if any, are benchmark verdicts, not registry/setup failures.

**Step 5: Commit operational notes only if needed**

```bash
git add docs/playbooks/development-operations.md
git commit -m "docs: record agent registry validation notes"
```

Only commit if new reusable lessons were found.

## Task 14: Adversarial Review And Final Gate

**Files:**

- Create: `vs_review/YYYY-MM-DD-agent-registration-registry-review.md`

**Step 1: Run subagent-vs-review**

Because this plan changes code behavior, use `subagent-vs-review` after implementation. The review packet should focus on:

- Whether profile schema is readable and stable.
- Whether unsupported policies are blocked before benchmark execution.
- Whether secrets are redacted from snapshots/reports.
- Whether Terminal-Bench bridge setup is correctly classified.
- Whether legacy `sandbox_setup_command` can still replay old runs without becoming the recommended path.

**Step 2: Fix accepted blockers**

For every accepted blocker:

- Add failing test.
- Fix implementation.
- Run targeted tests.
- Update review artifact.
- Commit.

**Step 3: Run full gate again**

Run:

```bash
scripts/test-after-change.sh
```

Expected:

- Pass.

**Step 4: Push**

Run:

```bash
git status --short
git push
```

Expected:

- Only intentional files are committed.
- `main` pushed.

## Acceptance Criteria

Implementation is complete only when all of these are true:

- `harnesslab init` creates commented, readable built-in profile templates and `agents/README.md`.
- `harnesslab agent schema --json` returns machine-readable field metadata with allowed values and examples.
- `AgentProfile` supports `[setup]`, `[skills]`, `[tools]`, and `[hooks]` with backwards-compatible defaults.
- `doctor --json` reports field-level errors with exact field path, accepted values, and suggested fix.
- Unsupported non-default skills/tools/hooks policy blocks run before benchmark execution.
- `setup.preset = custom` works as an advanced escape hatch and is visibly warned.
- `labels.sandbox_setup_command` remains legacy-compatible but is no longer used by new default templates.
- Sandbox tasks and Terminal-Bench external tasks both receive materialized setup.
- Run artifacts include `agent-profile.runtime.json`, `agent-profile.snapshot.json`, and `agent-runtime.materialized.json`.
- HTML report shows effective setup, skills, tools, and hooks summaries.
- Replay/resume remain compatible with old runs and new materialized snapshots.
- Targeted tests and `scripts/test-after-change.sh` pass with coverage thresholds intact.
- Real `claude-ds` Terminal-Bench smoke run succeeds through HarnessLab official CLI without mock or temporary scripts.

## Risk List

- **Agent-specific capability semantics are uneven.** Mitigation: strict materializer support matrix; block unsupported non-default policies instead of silently ignoring.
- **Terminal-Bench bridge setup failure could be misclassified as benchmark failure.** Mitigation: explicit setup failure markers and log scanner mapping to `execution/external_runner_setup_failed`.
- **Schema growth could push `config.rs` over 500 lines.** Mitigation: keep setup/capability structs in `agent_profile.rs` and only reference them from `config.rs`.
- **Commented templates could drift from schema.** Mitigation: parse templates in init tests and compare against `agent schema` reference fields.
- **Secrets could leak through materialized setup.** Mitigation: reuse redaction path and add tests for env secret redaction in `command.txt`, report, and materialized snapshot.
- **Backward compatibility with old runs.** Mitigation: keep default serde values, support legacy labels, and add replay tests for old snapshots missing `agent-runtime.materialized.json`.

## Suggested Commit Sequence

1. `test: register agent registry requirements`
2. `feat: add semantic agent profile schema`
3. `feat: expose agent profile schema reference`
4. `feat: generate readable agent profile templates`
5. `feat: add field-level agent profile diagnostics`
6. `feat: materialize agent registry profiles`
7. `feat: persist materialized agent runtime`
8. `feat: run sandbox agents from materialized setup`
9. `feat: pass materialized setup to terminal-bench bridge`
10. `feat: block unsupported agent registry policies before run`
11. `docs: document semantic agent registration`
12. `test: validate agent registration registry`

## Notes For Implementer

- Do not implement vendor-specific tools/hooks behavior by guessing. If the adapter cannot prove it can enforce a policy, return a blocking materialization error.
- Keep profile registration useful for helper agents: schema JSON, exact doctor field paths, and suggested fixes are product features, not developer niceties.
- Keep setup separate from benchmark adapters. Benchmark adapters consume already-materialized runtime configuration.
- Do not remove legacy `labels.sandbox_setup_command` until replay compatibility and migration docs are in place.
- Run `subagent-vs-review` after code implementation, not for this plan-only document.
