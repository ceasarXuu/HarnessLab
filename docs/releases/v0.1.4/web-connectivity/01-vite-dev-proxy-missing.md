# BUG-WEB-01: Vite dev server 缺少 `/api` proxy

- Created: 2026-06-23
- Updated: 2026-06-24
- Version: 1.1
- Status: Implemented
- Owner / Responsible: project maintainer
- Related Systems: frontend (Vite), ornnlab FastAPI
- Related Links: [README](README.md), [frontend/vite.config.ts](../../../../frontend/vite.config.ts), [ornnlab/app.py](../../../../ornnlab/app.py), [ornnlab/cli.py](../../../../ornnlab/cli.py), [ornnlab/settings.py](../../../../ornnlab/settings.py)
- Risk Level: Low
- Plan Type: Standard
- Phase: 1（通路打通）
- Revision Notes: v1.1 修正默认端口 8000→8765（F1），新增端口布局表（R6），新增后端启动命令 AC 项（R8）。来源：vs_review/2026-06-23-web-connectivity-plan-review.md

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

[frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 默认使用 `basePath = '/api'`，即前端运行时通过相对路径访问 API。但 [frontend/vite.config.ts](../../../../frontend/vite.config.ts) 既没有配置 `server.proxy` 也没有 `preview.proxy`，dev 与 preview 模式下 `/api/*` 请求只会落到 Vite 自身，返回 404 或被 SPA fallback 重写为 `index.html`，导致前端**无法在本地真正调通后端**。

## 端口布局（事实源）

| 用途 | 端口 | 来源 |
|---|---|---|
| Vite dev server | 4173 | [frontend/vite.config.ts](../../../../frontend/vite.config.ts) `server.port` |
| Vite preview server（默认） | 4173 | [frontend/vite.config.ts](../../../../frontend/vite.config.ts) `preview.port` |
| Playwright e2e preview server | 4174 | [frontend/package.json](../../../../frontend/package.json) `e2e` 脚本 `--port 4174 --strictPort` |
| FastAPI 后端 | **8765** | [ornnlab/cli.py#L24](../../../../ornnlab/cli.py) `web.add_argument("--port", type=int, default=8765)` 与 [ornnlab/settings.py](../../../../ornnlab/settings.py) `port: int = 8765` |

> 注：dev/preview 默认共用 4173；e2e 通过 `--strictPort 4174` 显式错开，避免与 dev 同时运行时冲突。后端固定 8765，**不要**误用 8000。

## 证据

[frontend/vite.config.ts](../../../../frontend/vite.config.ts) 当前内容：

```ts
server: {
  host: '127.0.0.1',
  port: 4173,
},
preview: {
  host: '127.0.0.1',
  port: 4173,
},
```

[frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 入口：

```ts
export const apiClient = createApiClient('/api')
```

后端默认监听 `127.0.0.1:8765`（[ornnlab/cli.py#L24](../../../../ornnlab/cli.py) `--port` 默认值；[ornnlab/settings.py](../../../../ornnlab/settings.py) `port: int = 8765`），与前端不同源。

## 修复方案

1. 在 `vite.config.ts` 中新增 `server.proxy` 与 `preview.proxy`，把 `/api` 转发到本地 FastAPI；目标地址从环境变量 `ORNNLAB_API_TARGET` 读取，默认 `http://127.0.0.1:8765`（与 [ornnlab/cli.py#L24](../../../../ornnlab/cli.py) 一致）。
2. 不在生产构建产物中硬编码 API 地址；仍保留运行时相对路径 `/api`，生产部署形态在 v1.0.5 PRD 决定。
3. 在 `frontend/README` 或 `docs/playbooks/development-operations.md`（不在本 PR 范围）追加一句"启动顺序：先 FastAPI，再 `npm run dev`"的提示，**本立项只交付 vite 配置变更**。

参考最小变更示例（实施时按上面方案执行）：

```ts
const apiTarget = process.env.ORNNLAB_API_TARGET ?? 'http://127.0.0.1:8765'

export default defineConfig({
  // ...
  server: {
    host: '127.0.0.1',
    port: 4173,
    proxy: { '/api': { target: apiTarget, changeOrigin: true } },
  },
  preview: {
    host: '127.0.0.1',
    port: 4173,
    proxy: { '/api': { target: apiTarget, changeOrigin: true } },
  },
})
```

## Acceptance Criteria

- [x] `npm --prefix frontend run dev` 后，浏览器请求 `/api/system/status` 返回 FastAPI 真实响应（非 Vite SPA fallback）。
- [x] `npm --prefix frontend run preview` 同样可代理 `/api`。
- [x] `ORNNLAB_API_TARGET` 环境变量可覆盖默认目标，已在本文件或 playbook 中说明。
- [x] dev/preview 启动日志中无 proxy 配置告警。
- [x] **后端启动命令可一行执行**：`python -m ornnlab web`（默认 `127.0.0.1:8765`，见 [ornnlab/cli.py#L22-L24](../../../../ornnlab/cli.py)），并在本文件或 playbook 中显式记录"先起后端、再起前端"的顺序。

## Implementation

落地于 commit `2fd7541`：
- [frontend/vite.config.ts](../../../../frontend/vite.config.ts) 新增 `server.proxy` + `preview.proxy`，目标默认 `http://127.0.0.1:8765`，可通过 `ORNNLAB_API_TARGET` 环境变量覆盖。

## 风险与回滚

- 配置变更仅影响本地开发体验，不进入生产构建产物，回滚直接还原 `vite.config.ts` 即可。
