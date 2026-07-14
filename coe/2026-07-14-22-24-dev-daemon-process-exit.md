# Problem P-001

## Symptoms
- `ornnlab dev start/restart` 后前端页面不可用或白屏。
- daemon 日志出现后端退出后重启失败。
- 既往日志包含 `spawn /opt/homebrew/Cellar/node@22/22.22.1_3/bin/node ENOENT`。
- 最近一次重启失败日志包含 `127.0.0.1:8765 address already in use`。

## Expected behavior
- `ornnlab dev start` 使用稳定 Node 启动 daemon、backend、frontend。
- `ornnlab dev stop/restart` 必须完整停止受管前后端服务及其子进程树。
- 重启不应留下占用 8765/5173 的孤儿进程。

## Actual behavior
- daemon 曾使用过期 Homebrew Cellar Node 路径重启自身，导致 `ENOENT`。
- 后端冷启动较慢，旧 30s timeout 容易误判。
- status 健康探测 500ms 过短，真实服务可用时可能显示 degraded。
- restart 时曾出现旧后端占用 8765，新后端无法绑定。

## Known facts
- `lib/dev-daemon.js` 已改为通过 `/usr/bin/env node` 启动 Node 脚本。
- backend 启动已优先使用 `.venv/bin/ornnlab`，避免 `uv run` 冷启动成本。
- Vite 已忽略 `storybook-static`、`dist`、`coverage`、`node_modules/.vite`。
- 当前检查时 8765 没有监听进程，说明端口占用是重启链路的历史残留状态。

# Hypothesis H-001

## Claim
daemon 原来直接使用 `process.execPath` 启动自身和 wrapper，Homebrew Node 路径变化后导致重启链路 `ENOENT`。

## Prediction
- daemon 日志中能看到旧 Cellar Node 路径。
- 改为 PATH-stable Node invocation 后测试应覆盖不再依赖 `process.execPath`。

## Evidence
- daemon 日志曾出现 `spawn /opt/homebrew/Cellar/node@22/22.22.1_3/bin/node ENOENT`。
- `tests/node/dev-daemon.test.js` 已增加 PATH-stable node launcher 断言。

# Hypothesis H-002

## Claim
后端和前端冷启动时间超过旧默认 30s，导致 daemon 误判服务启动失败。

## Prediction
- 直接启动 `.venv/bin/ornnlab web` 可以成功，但等待时间明显长于短 timeout。
- 默认 timeout 提高到 300s 后冷启动不应因时间不足失败。

## Evidence
- 直接后端 smoke 曾最终返回 health 200，但启动耗时较长。
- `lib/dev.js` 默认 startup timeout 已调整为 300000ms。

# Hypothesis H-003

## Claim
wrapper 只可靠处理直接子进程，遇到后端启动链路产生孙进程或独立监听进程时，stop/restart 可能留下端口占用。

## Prediction
- 代码中 wrapper 的 signal forwarding 使用直接 child pid，没有明确以 child process group 为边界清理 descendants。
- 增加“wrapper 停止时清理孙进程监听端口”的测试可复现/防回退。

## Evidence
- `lib/dev-child-wrapper.js` 当前 SIGTERM 分支在 POSIX 上调用 `process.kill(child.pid, signal)`。
- 重启失败日志显示新后端无法绑定 8765，符合旧监听进程残留的现象。

# Hypothesis H-004

## Claim
`/system/health` 在 async route 中直接执行同步体检逻辑，体检慢或浏览器持续请求时会阻塞后端事件循环，导致 daemon 使用的轻量探活接口也无法及时返回。

## Prediction
- 后端端口已监听，但 `curl /api/webui/v1/system/live` 超时。
- `webui.py` 中 `system_health` 未把 `_system(request).health()` 放入线程池。
- 将完整 health 改为 `asyncio.to_thread` 后，live/status 应能独立返回。

## Evidence
- 真实服务重启后端口监听存在，但 `/system/live` 和 `/system/health` 均超时，`dev status` 显示 backend/frontend healthy 为 false。
- 代码检查确认 `system_health` 原来直接调用同步 `_system(request).health()`。

# Hypothesis H-005

## Claim
本机 `http_proxy` / `https_proxy` 指向 127.0.0.1:7890，但缺少 `NO_PROXY`，导致 daemon/status/curl 对 127.0.0.1 的本地探活也走代理；代理连接或 keep-alive 行为让探活超时，服务被误判 degraded。

## Prediction
- 环境变量中存在 `http_proxy` 和 `https_proxy`，没有对应 localhost no_proxy。
- 显式设置 `NO_PROXY=127.0.0.1,localhost` 后 `dev status` 应恢复 running。
- 启动器自动补齐 no_proxy 后，不再依赖用户手动设置。

## Evidence
- `env | rg -i 'http.*proxy|no_proxy'` 显示 `http_proxy=http://127.0.0.1:7890`、`https_proxy=http://127.0.0.1:7890`。
- `curl -v` 显示请求 `127.0.0.1:8765` 时实际连接了代理 `127.0.0.1:7890`。
- Python `urllib.request.urlopen` 直连 `/system/live` 8ms 返回 200。
- `NO_PROXY=127.0.0.1,localhost no_proxy=127.0.0.1,localhost node bin/ornnlab.js dev status --json` 返回 `status: running`。
