# OrnnLab Development Operations

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-15 | Recorded operational lessons for the Harbor WebUI rewrite. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Linked operations guidance to document version governance. |
| 1.2 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-27 | Recorded Colima startup check before real Harbor Docker smoke. |
| 1.3 | Python app `0.2.0`; Harbor `0.13.x` | 2026-07-19 | 记录 macOS 到 Ubuntu 的数据恢复与验证经验。 |
| 1.4 | npm launcher `0.1.3`; Vite `8.x` | 2026-07-19 | 记录 Ubuntu inotify 限额导致前端启动失败的诊断和部署方案。 |

This file records current operational lessons for the Harbor WebUI rewrite.
Legacy Rust CLI operations were archived on 2026-06-15.

- Archived copy: `../archive/2026-06-15-pre-harbor-webui-redesign/development-operations.md`
- Current plan: `plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

## 2026-06-15 Documentation Archive Pass

When moving old documents, keep the historical content under
`docs/archive/2026-06-15-pre-harbor-webui-redesign/` and leave only short
supersession stubs at old paths that are still referenced by tests, reports, or
onboarding links.

Legacy tests that validate old document semantics should read the archived copy
directly. Current stubs must stay short and must not contain stale
implementation instructions.

## 2026-06-27 Real Harbor Docker Smoke

本机有 Docker CLI 不代表 Docker daemon 已可用。真实 Harbor Docker smoke
前先确认当前 context 和 Colima 状态：

```bash
docker context ls
colima status
docker info
```

如果 `docker info` 报 `/Users/xuzhang/.colima/default/docker.sock` 不存在，
说明当前 `colima` context 指向的 daemon 没启动。先执行：

```bash
colima start
docker info
ORNNLAB_REAL_HARBOR=1 uv run pytest -m docker tests/python/test_harbor_real_smoke.py tests/python/test_real_harbor_cancel_recovery.py -vv
```

2026-06-27 验证结果：`colima start` 后 Docker ServerVersion 为 `29.2.1`，
真实 Harbor Docker smoke `3 passed`，耗时约 3 分 58 秒。

## 2026-07-19 Ubuntu 数据恢复

恢复包的外层 `.sha256` 使用相对文件名，必须在归档所在目录执行校验：

```bash
cd /path/to/backup
sha256sum -c ornnlab-ubuntu-backup-20260719.tar.gz.sha256
```

恢复前还要在解包目录执行 `sha256sum -c SHA256SUMS`，并通过
`git bundle verify payload/HarnessLab.bundle` 校验源码历史。如果当前仓库 HEAD
与 manifest 的 `gitCommit` 一致且工作区干净，可保留当前仓库，仅恢复
`~/.ornnlab/data` 与 `~/ornnlab-data`，避免恢复脚本因源码目录非空而停止。

macOS 创建的归档可能包含 `LIBARCHIVE.xattr.*` 扩展头和 `._*` AppleDouble
文件。扩展头警告不代表内容损坏；解压外置数据时应过滤 AppleDouble 文件：

```bash
tar -xzf payload/external-data.tar.gz \
  -C ~/ornnlab-data \
  --exclude='._*' \
  --exclude='*/._*'
```

完成主数据导入和 `rebase_paths.py` 路径重映射后，至少执行以下验证：

1. 对 `~/.ornnlab/data/*.sqlite` 执行 `PRAGMA integrity_check`。
2. 搜索 `/Users/` 与旧 `/Volumes/` 数据路径，确认没有可识别文本残留。
3. 确认 `terminal-bench@2.0`、`swebenchpro@1.0` 和 Job 目录存在。
4. 执行 `ORNNLAB_HOME=~/.ornnlab/data uv run ornnlab doctor`，确认 Docker、Harbor、数据库 schema 与孤儿容器检查均正常。

## 2026-07-19 Ubuntu Vite watcher 限额

如果 `ornnlab dev start` 报 `frontend exited before becoming ready`，且
`~/.ornnlab/dev-service/logs/frontend.log` 包含以下错误：

```text
ENOSPC: System limit for number of file watchers reached
```

先检查 inotify 限额和当前占用，不要把该错误误判成磁盘空间不足：

```bash
sysctl fs.inotify.max_user_watches fs.inotify.max_user_instances
```

有 sudo 权限时，推荐提高开发机的 watch 限额并持久化：

```bash
sudo tee /etc/sysctl.d/99-ornnlab-inotify.conf >/dev/null <<'EOF'
fs.inotify.max_user_watches=524288
fs.inotify.max_user_instances=512
EOF
sudo sysctl --system
```

暂时不能修改内核参数时，只对 OrnnLab launcher 启用 polling，避免影响其他
Node 项目：

```bash
CHOKIDAR_USEPOLLING=true CHOKIDAR_INTERVAL=500 ornnlab dev start
```

部署到固定源码目录时，可用用户级 wrapper 固化 `ORNNLAB_SOURCE`、
`ORNNLAB_HOME` 和上述 polling 环境。wrapper 应放在用户自己的 `~/.local/bin`
下，不提交硬编码的本机路径到仓库。完成后必须执行 `dev stop`、`dev start`、
`dev status --json` 生命周期回归，并验证后端与前端代理的 `/api/webui/v1/system/live`。
