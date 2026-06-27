# OrnnLab Development Operations

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-15 | Recorded operational lessons for the Harbor WebUI rewrite. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Linked operations guidance to document version governance. |
| 1.2 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-27 | Recorded Colima startup check before real Harbor Docker smoke. |

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
