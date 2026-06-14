# Agent Profile Reference

Agent Profile 是 HarnessLab 注册和评测一个完整 CLI harness 的单文件注册表。每个 profile 表达一个可比较配置：agent kind、启动命令、认证继承、setup、skills、tools、hooks、usage parser 和报告标签。

如果你要从零完成注册流程，先看 [Agent Registration Guide](agent-registration-guide.md)。本文只作为字段参考手册使用。

先用机器可读 schema 辅助生成或校验：

```bash
harnesslab agent schema --json
harnesslab doctor --json
```

## Minimal Shape

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
```

## Fields

| Field | Required | Values | Example | Runtime status | Meaning |
| --- | --- | --- | --- | --- | --- |
| `schema_version` | yes | `1` | `1` | active; enforced by config loader | Profile schema version. Schema: Profile schema version. |
| `name` | yes | `[a-zA-Z0-9][a-zA-Z0-9._-]*` | `claude-ds` | active; enforced by config loader and `--agent` lookup | Value passed to `--agent`. Schema: Profile name used by --agent. |
| `kind` | yes | `codex`, `claude-code`, `opencode`, `pi-coding-agent`, `custom`, `fake` | `claude-code` | active; selects setup defaults, capability catalog, and materialization support | Harness kind. `fake` is for tests, not normal user registration. Schema: CLI harness kind. |
| `display_name` | yes | `string` | `Claude DS` | active; stored in snapshots/reports | Human-facing report name. Schema: Human-readable report name. |
| `command` | yes | `shell command` | `claude-ds -p --bare --output-format text` | active; executed by runner after setup; public artifacts are redacted | Agent command template. Do not put secret values here. Schema: Agent command template. |
| `input_mode` | yes | `stdin`, `argument`, `file`, `tty` | `stdin` | active; enforced by command renderer | How HarnessLab passes the task instruction. `argument` requires `{{instruction}}`; `file` requires `{{instruction_file}}` or `{{instruction}}`. Schema: How the task instruction is passed to the agent. |
| `working_dir` | yes | `workspace`, `run_dir` | `workspace` | active; enforced by process runner | Agent process working directory. Schema: Agent working directory. |
| `timeout_sec` | yes | `positive integer` seconds | `300` | active; used as per-task process timeout unless benchmark/run config is stricter | Default per-task agent timeout. Schema: Default per-task agent timeout. |
| `version_command` | no | `shell command` or omitted | `claude --version` | active; probed by doctor/run/replay with redacted bounded output | Optional version probe; mismatch on replay is a warning/event, not a benchmark score. Schema: Optional bounded CLI version probe. |
| `auth.inherit` | yes | `true`, `false` | `true` | active; enforced for host and Docker auth inheritance | Enables declared env/path inheritance. If `false`, `inherit_env` and `include_paths` are not inherited. Schema: Enable declared auth env/path inheritance. |
| `auth.inherit_env` | yes | `string[]` environment variable names | `["ANTHROPIC_AUTH_TOKEN"]` | active; only used when `auth.inherit = true`; host execution receives an explicit env map | Names to pass through; values are read from the host environment. Schema: Environment variable names to pass through. |
| `auth.include_paths` | yes | `path[]` or `host:container:mode[]` | `["~/.claude:/root/.claude:ro"]` | active; only used when `auth.inherit = true`; Docker mount dry-run checks readability/mountability | Auth/config paths mounted into sandbox. Schema: Auth/config paths to mount. |
| `auth.exclude_paths` | yes | `path[]` | `["~/.claude/logs"]` | active; used when resolving inherited auth paths | Removes inherited paths. Schema: Inherited auth paths to exclude. |
| `auth.mount_ssh_socket` | yes | `true`, `false` | `false` | active; used when resolving auth mounts; doctor checks availability | Mounts SSH agent socket. Schema: Mount SSH agent socket. |
| `auth.mount_docker_socket` | yes | `true`, `false` | `false` | active; doctor warns/errors because this is high privilege | Mounts Docker socket. Schema: Mount Docker socket; high privilege. |
| `setup.preset` | no | `none`, `builtin`, `custom` | `builtin` | active; materialized before agent execution where supported | Setup strategy before running agent in sandbox/external bridge. Schema: Sandbox setup strategy. |
| `setup.required_commands` | no | `command name[]`; bare command names using letters, digits, `.`, `_`, `+`, `-` | `["claude", "claude-ds"]` | active; checked by doctor; builtin/custom setup state is explained | Commands expected after setup. No shell pipes, paths, or arguments. Schema: Commands that must exist after setup. |
| `setup.run_as` | no | `root`, `harnesslab`, `current` | `current` when omitted; built-in Docker-oriented templates may set `harnesslab` | active; enforceable in Docker sandbox; host paths block unless `current` | User used for agent execution. Use `current` for host-only tasks. Schema: User used to run the agent command. |
| `setup.commands` | no | `shell command[]` | `["npm install -g @anthropic-ai/claude-code"]` | active; executed only with `setup.preset = "custom"`; public artifacts are redacted | Advanced custom setup. Keep secrets out of commands. Schema: Advanced custom setup commands. |
| `skills.inherit` | no | `true`, `false` | `true` | active; resolved into `candidate_effective`; non-default policies block unless materializer is verified | Inherit default skills for this harness kind when `skills.allow` is empty. Schema: Inherit default skills. |
| `skills.allow` | no | `string[]` known skill names for the selected catalog | `["code-review"]` | active; explicit desired skill set; unknown entries are field errors | Skill whitelist. If non-empty, it is the source set even when `inherit = false`. Must not overlap `skills.deny`. Schema: Skill whitelist. |
| `skills.deny` | no | `string[]` known skill names for the selected catalog | `["test-runner"]` | active; subtracts from inherited defaults or explicit allow; unknown entries are field errors | Skill blacklist. Schema: Skill blacklist. |
| `skills.include_paths` | no | `path[]` | `["~/.claude/skills"]` | active; accepted only for `skills`; rejected for `tools`/`hooks` | Extra skill locations. Schema: Extra skill paths. |
| `tools.inherit` | no | `true`, `false` | `true` | active; resolved into `candidate_effective`; non-default policies block unless materializer is verified | Inherit default tools when `tools.allow` is empty. Schema: Inherit default tools. |
| `tools.allow` | no | `string[]` known tool names for the selected catalog | `["bash"]` | active; explicit desired tool set; unknown entries are field errors | Tool whitelist. If non-empty, it is the source set even when `inherit = false`. Must not overlap `tools.deny`. Schema: Tool whitelist. |
| `tools.deny` | no | `string[]` known tool names for the selected catalog | `["web_search"]` | active; subtracts from inherited defaults or explicit allow; unknown entries are field errors | Tool blacklist. Schema: Tool blacklist. |
| `hooks.inherit` | no | `true`, `false` | `true` | active; resolved into `candidate_effective`; non-default policies block unless materializer is verified | Inherit default hooks when `hooks.allow` is empty. Schema: Inherit default hooks. |
| `hooks.allow` | no | `string[]` known hook names for the selected catalog | `["pre_tool_use"]` | active; explicit desired hook set; unknown entries are field errors | Hook whitelist. If non-empty, it is the source set even when `inherit = false`. Must not overlap `hooks.deny`. Schema: Hook whitelist. |
| `hooks.deny` | no | `string[]` known hook names for the selected catalog | `["post_tool_use"]` | active; subtracts from inherited defaults or explicit allow; unknown entries are field errors | Hook blacklist. Schema: Hook blacklist. |
| `usage.parser` | yes | `none`, `regex`, `json_path` | `none` | active; enforced by usage collector | Token/cost parser. `none` records usage as unknown. Schema: Token/cost usage parser. |
| `usage.source` | no | `agent_stdout`, `agent_stderr`, `agent_logs`, `file:<safe-relative-path>` | `agent_logs` | active; safe paths only; no run-dir escape | Input source for usage parsing. Schema: Input source for usage parsing. |
| `usage.input_tokens_key` | no | `string` | `input_tokens` | active; used by structured/JSON usage parsing | Field/key name for input tokens. Schema: Field/key name for input tokens. |
| `usage.output_tokens_key` | no | `string` | `output_tokens` | active; used by structured/JSON usage parsing | Field/key name for output tokens. Schema: Field/key name for output tokens. |
| `usage.total_tokens_key` | no | `string` | `total_tokens` | active; used by structured/JSON usage parsing | Field/key name for total tokens. Schema: Field/key name for total tokens. |
| `usage.cost_usd_key` | no | `string` | `cost_usd` | active; used by structured/JSON usage parsing | Field/key name for USD cost. Schema: Field/key name for USD cost. |
| `labels` | no | `key/value` string map | `{"model":"deepseek"}` | active; stored in profile snapshots/reports; adapter-specific known labels are consumed by adapters | Report labels and benchmark adapter hints. Schema: Open report labels and benchmark adapter hints map. |
| `labels.model` | no | `string` | `deepseek` | active; report/common model label | Human-readable model label. Schema: Common report model/config label. |
| `labels.terminal_bench_agent` | no | `terminal-bench agent name` | `codex` | active; consumed by Terminal-Bench adapter | Uses a Terminal-Bench built-in agent. Schema: Terminal-Bench built-in agent name. |
| `labels.terminal_bench_agent_import_path` | no | `python import path` | `harnesslab_tb_agent:HarnessLabCommandAgent` | active; consumed by Terminal-Bench adapter; host-agent run_as precheck applies | Uses HarnessLab's Terminal-Bench bridge agent. Schema: Terminal-Bench import-path bridge agent. |
| `labels.terminal_bench_agent_pythonpath` | no | `absolute path`, `python path string` | `/repo/integrations/terminal_bench` | active; consumed by Terminal-Bench adapter | Python path for the Terminal-Bench bridge. Schema: Python path prepended before loading the Terminal-Bench bridge. |
| `labels.terminal_bench_model` | no | `string` | `deepseek` | active; consumed by Terminal-Bench adapter | Model label for Terminal-Bench. Schema: Model label consumed by Terminal-Bench built-in agents. |
| `labels.sandbox_setup_command` | no | `shell command` | `npm install -g @anthropic-ai/claude-code` | legacy; legacy compatibility only; prefer `[setup]` | Legacy setup field for old profiles and old snapshots. Schema: Legacy setup escape hatch; new profiles should use [setup]. |

## Materialization Rules

HarnessLab does not silently ignore capability policies. If a non-default `skills`, `tools`, or `hooks` policy cannot be enforced by the selected `kind`, `doctor` and `run` return a blocking error before benchmark execution.

Current support:

| Kind | Builtin setup | `skills/tools/hooks` non-default policies |
| --- | --- | --- |
| `codex` | installs/validates `codex` | blocked until a verified Codex materializer exists |
| `claude-code` | installs/validates `claude`; can generate `claude-ds` wrapper | blocked until a verified Claude materializer exists |
| `opencode` | installs/validates `opencode` | blocked until a verified OpenCode materializer exists |
| `pi-coding-agent` | validates command availability | blocked until a verified Pi Coding Agent materializer exists |
| `custom` | no builtin setup | blocked unless a future custom runtime contract is added |
| `fake` | no builtin setup | reserved for contract tests; normal users should not register it |

Capability algebra:

```text
if allow is non-empty:
  target = allow
else if inherit = true:
  target = default_enabled(kind, domain)
else:
  target = {}

candidate_effective = target - deny
effective = candidate_effective only when the domain materializer is verified and no errors exist
```

Important consequences:

- `inherit = false` does not disable `allow`. If `allow = ["bash"]`, the requested source set is `["bash"]`.
- `allow` and `deny` are validated against `available(kind, domain)`. A typo in either list is a blocking field error, not a no-op.
- `deny` subtracts from either inherited defaults or explicit `allow`.
- `skills.include_paths` is valid only for `skills`; `tools.include_paths` and `hooks.include_paths` are rejected.
- `doctor --json` reports `available`, `default_enabled`, `candidate_effective`, `effective`, `unsupported_reason`, `field_path`, and `suggested_fix`.

`labels.sandbox_setup_command` remains a legacy compatibility path for old runs and old profiles. New profiles should use `[setup]`.

## Runtime Artifacts

New runs write public, redacted artifacts that show what was actually materialized:

- `agent-profile.snapshot.json`: public profile snapshot with command-like fields redacted.
- `agent-runtime.materialized.json`: setup summary, optional setup script, run user, capability summaries, and structured resolved capability policies.
- `agent-version.snapshot.json`: present when `version_command` exists; includes redacted command/output/status.
- `report.html`: links the profile/runtime/version snapshots and displays effective capability summaries plus version probe status.
- `command.txt` and `tasks/**/agent/command.txt`: redacted command snapshots for replay/debugging.

Replay uses the source run's runtime/report profile snapshots to keep known secret values redacted even if the current shell no longer has the original environment variables. `agent-profile.runtime.json` is private runtime state; public artifact checks intentionally exclude only that file.

## Validation Checklist

- Run `harnesslab doctor --json` after editing a profile.
- Confirm the report and `agent-runtime.materialized.json` show the intended setup and capability summaries.
- Keep API keys in environment variables, never in `command`, `labels`, or `setup.commands`.
- Use separate profile names for model/skill/tool/hook variants you want to compare.
