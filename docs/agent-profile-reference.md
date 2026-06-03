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

| Field | Required | Values | Example | Meaning |
| --- | --- | --- | --- | --- |
| `schema_version` | yes | `1` | `1` | Profile schema version. |
| `name` | yes | `[a-zA-Z0-9][a-zA-Z0-9._-]*` | `claude-ds` | Value passed to `--agent`. |
| `kind` | yes | `codex`, `claude-code`, `opencode`, `pi-coding-agent`, `custom`, `fake` | `claude-code` | Selects default auth/setup/materialization rules. |
| `display_name` | yes | string | `Claude DS` | Human-facing report name. |
| `command` | yes | shell command | `claude-ds -p` | Agent command template. Do not put secret values here. |
| `input_mode` | yes | `stdin`, `argument`, `file`, `tty` | `stdin` | How HarnessLab passes the task instruction. |
| `working_dir` | yes | `workspace`, `run_dir` | `workspace` | Agent process working directory. |
| `timeout_sec` | yes | positive integer | `300` | Default per-task agent timeout. |
| `version_command` | no | shell command | `claude --version` | Optional version probe. |
| `auth.inherit` | yes | `true`, `false` | `true` | Enables declared env/path inheritance. |
| `auth.inherit_env` | yes | env var names | `["ANTHROPIC_AUTH_TOKEN"]` | Names to pass through; values are read from the host environment. |
| `auth.include_paths` | yes | path or `host:container:mode` array | `["~/.claude:/root/.claude:ro"]` | Auth/config paths mounted into sandbox. |
| `auth.exclude_paths` | yes | path array | `[]` | Removes inherited paths. |
| `auth.mount_ssh_socket` | yes | `true`, `false` | `false` | Mounts SSH agent socket. |
| `auth.mount_docker_socket` | yes | `true`, `false` | `false` | Mounts Docker socket; high privilege and doctor warns/errors. |
| `setup.preset` | no | `none`, `builtin`, `custom` | `builtin` | Setup strategy before running agent in sandbox/external bridge. |
| `setup.required_commands` | no | bare command names | `["claude", "claude-ds"]` | Commands expected after setup. No shell pipes or paths. |
| `setup.run_as` | no | `root`, `harnesslab`, `current` | `harnesslab` | User used for agent execution inside sandbox. |
| `setup.commands` | no | shell command array | `[]` | Advanced custom setup. Valid only with `preset = "custom"`. |
| `skills.inherit` | no | `true`, `false` | `true` | Inherit default skills for this harness kind. |
| `skills.allow` | no | string array | `["skill-a"]` | Skill whitelist. Must not overlap `skills.deny`. |
| `skills.deny` | no | string array | `["skill-b"]` | Skill blacklist. |
| `skills.include_paths` | no | path array | `["~/.claude/skills"]` | Extra skill locations. |
| `tools.inherit` | no | `true`, `false` | `true` | Inherit default tools. |
| `tools.allow` | no | string array | `["bash"]` | Tool whitelist. Must not overlap `tools.deny`. |
| `tools.deny` | no | string array | `["web_search"]` | Tool blacklist. |
| `hooks.inherit` | no | `true`, `false` | `true` | Inherit default hooks. |
| `hooks.allow` | no | string array | `["pre_tool_use"]` | Hook whitelist. Must not overlap `hooks.deny`. |
| `hooks.deny` | no | string array | `["post_tool_use"]` | Hook blacklist. |
| `usage.parser` | yes | `none`, `regex`, `json_path` | `none` | Token/cost parser. |
| `labels.*` | no | string key/value | `model = "deepseek"` | Report labels and benchmark adapter hints. |

## Materialization Rules

HarnessLab does not silently ignore capability policies. If a non-default `skills`, `tools`, or `hooks` policy cannot be enforced by the selected `kind`, `doctor` and `run` return a blocking error before benchmark execution.

Current MVP support:

| Kind | Builtin setup | `skills/tools/hooks` non-default policies |
| --- | --- | --- |
| `codex` | installs/validates `codex` | blocked until adapter materializer exists |
| `claude-code` | installs/validates `claude`; can generate `claude-ds` wrapper | blocked until adapter materializer exists |
| `opencode` | installs/validates `opencode` | blocked until adapter materializer exists |
| `pi-coding-agent` | validates command availability | blocked until adapter materializer exists |
| `custom` | no builtin setup | blocked |
| `fake` | no builtin setup | blocked |

`labels.sandbox_setup_command` remains a legacy compatibility path for old runs and old profiles. New profiles should use `[setup]`.

## Validation Checklist

- Run `harnesslab doctor --json` after editing a profile.
- Confirm the report and `agent-runtime.materialized.json` show the intended setup and capability summaries.
- Keep API keys in environment variables, never in `command`, `labels`, or `setup.commands`.
- Use separate profile names for model/skill/tool/hook variants you want to compare.
