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
| `schema_version` | yes | `1` | `1` | enforced by config loader | Profile schema version. |
| `name` | yes | `[a-zA-Z0-9][a-zA-Z0-9._-]*` | `claude-ds` | enforced by config loader and `--agent` lookup | Value passed to `--agent`. |
| `kind` | yes | `codex`, `claude-code`, `opencode`, `pi-coding-agent`, `custom`, `fake` | `claude-code` | selects setup defaults, capability catalog, and materialization support | Harness kind. `fake` is for tests, not normal user registration. |
| `display_name` | yes | string | `Claude DS` | stored in snapshots/reports | Human-facing report name. |
| `command` | yes | shell command | `claude-ds -p` | executed by runner after setup; public artifacts are redacted | Agent command template. Do not put secret values here. |
| `input_mode` | yes | `stdin`, `argument`, `file`, `tty` | `stdin` | enforced by command renderer | How HarnessLab passes the task instruction. `argument` requires `{{instruction}}`; `file` requires `{{instruction_file}}` or `{{instruction}}`. |
| `working_dir` | yes | `workspace`, `run_dir` | `workspace` | enforced by process runner | Agent process working directory. |
| `timeout_sec` | yes | positive integer seconds | `300` | used as per-task process timeout unless benchmark/run config is stricter | Default per-task agent timeout. |
| `version_command` | no | shell command or omitted | `claude --version` | probed by doctor/run/replay with redacted bounded output | Optional version probe; mismatch on replay is a warning/event, not a benchmark score. |
| `auth.inherit` | yes | `true`, `false` | `true` | enforced for host and Docker auth inheritance | Enables declared env/path inheritance. If `false`, `inherit_env` and `include_paths` are not inherited. |
| `auth.inherit_env` | yes | environment variable name array | `["ANTHROPIC_AUTH_TOKEN"]` | only used when `auth.inherit = true`; host execution receives an explicit env map | Names to pass through; values are read from the host environment. |
| `auth.include_paths` | yes | path or `host:container:mode` array | `["~/.claude:/root/.claude:ro"]` | only used when `auth.inherit = true`; Docker mount dry-run checks readability/mountability | Auth/config paths mounted into sandbox. |
| `auth.exclude_paths` | yes | path array | `[]` | used when resolving inherited auth paths | Removes inherited paths. |
| `auth.mount_ssh_socket` | yes | `true`, `false` | `false` | used when resolving auth mounts; doctor checks availability | Mounts SSH agent socket. |
| `auth.mount_docker_socket` | yes | `true`, `false` | `false` | doctor warns/errors because this is high privilege | Mounts Docker socket. |
| `setup.preset` | no | `none`, `builtin`, `custom` | `builtin` | materialized before agent execution where supported | Setup strategy before running agent in sandbox/external bridge. |
| `setup.required_commands` | no | bare command names using letters, digits, `.`, `_`, `+`, `-` | `["claude", "claude-ds"]` | checked by doctor; builtin/custom setup state is explained | Commands expected after setup. No shell pipes, paths, or arguments. |
| `setup.run_as` | no | `root`, `harnesslab`, `current` | `harnesslab` | enforceable in Docker sandbox; host paths block unless `current` | User used for agent execution. Use `current` for host-only tasks. |
| `setup.commands` | no | shell command array | `[]` | executed only with `setup.preset = "custom"`; public artifacts are redacted | Advanced custom setup. Keep secrets out of commands. |
| `skills.inherit` | no | `true`, `false` | `true` | resolved into `candidate_effective`; non-default policies block unless materializer is verified | Inherit default skills for this harness kind when `skills.allow` is empty. |
| `skills.allow` | no | known skill names for the selected catalog | `["code-review"]` | explicit desired skill set; unknown entries are field errors | Skill whitelist. If non-empty, it is the source set even when `inherit = false`. Must not overlap `skills.deny`. |
| `skills.deny` | no | known skill names for the selected catalog | `["test-runner"]` | subtracts from inherited defaults or explicit allow; unknown entries are field errors | Skill blacklist. |
| `skills.include_paths` | no | path array | `["~/.claude/skills"]` | accepted only for `skills`; rejected for `tools`/`hooks` | Extra skill locations. |
| `tools.inherit` | no | `true`, `false` | `true` | resolved into `candidate_effective`; non-default policies block unless materializer is verified | Inherit default tools when `tools.allow` is empty. |
| `tools.allow` | no | known tool names for the selected catalog | `["bash"]` | explicit desired tool set; unknown entries are field errors | Tool whitelist. If non-empty, it is the source set even when `inherit = false`. Must not overlap `tools.deny`. |
| `tools.deny` | no | known tool names for the selected catalog | `["web_search"]` | subtracts from inherited defaults or explicit allow; unknown entries are field errors | Tool blacklist. |
| `hooks.inherit` | no | `true`, `false` | `true` | resolved into `candidate_effective`; non-default policies block unless materializer is verified | Inherit default hooks when `hooks.allow` is empty. |
| `hooks.allow` | no | known hook names for the selected catalog | `["pre_tool_use"]` | explicit desired hook set; unknown entries are field errors | Hook whitelist. If non-empty, it is the source set even when `inherit = false`. Must not overlap `hooks.deny`. |
| `hooks.deny` | no | known hook names for the selected catalog | `["post_tool_use"]` | subtracts from inherited defaults or explicit allow; unknown entries are field errors | Hook blacklist. |
| `usage.parser` | yes | `none`, `regex`, `json_path` | `none` | enforced by usage collector | Token/cost parser. `none` records usage as unknown. |
| `usage.source` | no | `agent_stdout`, `agent_stderr`, `agent_logs`, `file:<safe-relative-path>` | `agent_logs` | safe paths only; no run-dir escape | Input source for usage parsing. |
| `usage.input_tokens_key` | no | string | `input_tokens` | used by structured/JSON usage parsing | Field/key name for input tokens. |
| `usage.output_tokens_key` | no | string | `output_tokens` | used by structured/JSON usage parsing | Field/key name for output tokens. |
| `usage.total_tokens_key` | no | string | `total_tokens` | used by structured/JSON usage parsing | Field/key name for total tokens. |
| `usage.cost_usd_key` | no | string | `cost_usd` | used by structured/JSON usage parsing | Field/key name for USD cost. |
| `labels.*` | no | string key/value | `model = "deepseek"` | stored in profile snapshots/reports; adapter-specific known labels are consumed by adapters | Report labels and benchmark adapter hints. |
| `labels.model` | no | string | `deepseek` | report/common model label | Human-readable model label. |
| `labels.terminal_bench_agent` | no | Terminal-Bench agent name | `codex` | consumed by Terminal-Bench adapter | Uses a Terminal-Bench built-in agent. |
| `labels.terminal_bench_agent_import_path` | no | Python import path | `harnesslab_tb_agent:HarnessLabCommandAgent` | consumed by Terminal-Bench adapter; host-agent run_as precheck applies | Uses HarnessLab's Terminal-Bench bridge agent. |
| `labels.terminal_bench_agent_pythonpath` | no | absolute path or Python path string | `/repo/integrations/terminal_bench` | consumed by Terminal-Bench adapter | Python path for the Terminal-Bench bridge. |
| `labels.terminal_bench_model` | no | string | `deepseek` | consumed by Terminal-Bench adapter | Model label for Terminal-Bench. |
| `labels.swe_bench_pro_agent` | no | `gold` or adapter-supported value | `gold` | consumed by SWE-bench Pro adapter; host-agent run_as precheck applies for `gold` | Selects special SWE-bench Pro agent behavior. |
| `labels.sandbox_setup_command` | no | shell command | `npm install -g @anthropic-ai/claude-code` | legacy compatibility only; prefer `[setup]` | Legacy setup field for old profiles and old snapshots. |

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
