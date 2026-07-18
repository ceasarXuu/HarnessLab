# Problem P-001: Ubuntu 开发守护进程启动时前端提前退出
- Status: fixed
- Created: 2026-07-19 04:22
- Updated: 2026-07-19 04:27
- Objective: 让恢复后的 Ubuntu 本机开发环境通过应用级守护进程稳定启动前后端。
- Symptoms:
  - 执行仓库内 Node launcher 的 `dev start` 返回 `frontend exited before becoming ready`。
- Expected behavior:
  - daemon、后端和前端均进入 Running，5173/8765 健康端点可访问。
- Actual behavior:
  - launcher 在前端就绪阶段退出，服务未部署成功。
- Impact:
  - Ubuntu 本机 WebUI 无法访问，恢复后的项目数据无法通过开发界面使用。
- Reproduction:
  - `ORNNLAB_HOME=/home/zhangxu/.ornnlab/data ORNNLAB_SOURCE=/home/zhangxu/HarnessLab node bin/ornnlab.js dev start`
- Environment:
  - Ubuntu x86_64；Node 25.9.0；npm 11.12.1；main@ba8e96b；Docker 29.6.1。
- Known facts:
  - 启动前 5173/8765 均未监听。
  - 前后端依赖已按锁文件安装。
  - 当前用户使用 65,402 / 65,536 个 inotify watches，其中 VS Code 使用 60,021 个。
  - 用户级 launcher wrapper 仅为 OrnnLab 注入 polling 环境，并固定当前源码和数据路径。
- Ruled out:
  - 启动前已有进程占用默认端口。
- Fix criteria:
  - 原始启动命令成功；status 为 Running；直接和代理健康端点均成功；日志无未处理启动错误。
- Current conclusion: inotify watch 限额耗尽导致 Vite 确定性退出；用户级隔离 polling 部署已通过完整生命周期和健康检查验证。
- Related hypotheses:
  - H-001
  - H-002
- Resolution basis:
  - H-001；E-003、E-004、E-006、E-007、E-008
- Close reason:
  - 原始开发环境启动目标已验证；永久提升 sysctl 作为可选的主机优化记录在运维文档。

## Hypothesis H-001: 前端运行时或依赖导致 Vite 进程退出
- Status: confirmed
- Parent: P-001
- Claim: daemon 启动的 Vite 命令在本机 Node/npm 或已安装依赖下产生确定性错误并退出。
- Layer: environment
- Factor relation: single
- Depends on:
  - none
- Rationale:
  - launcher 明确报告 frontend 在 ready 前退出，且当前 Node 25 高于项目文档要求的 Node 22。
- Falsifiable predictions:
  - If true: frontend.log 或同环境直接启动会出现相同的非零退出错误。
  - If false: 直接启动 Vite 可持续运行且日志没有运行时/依赖错误。
- Diagnostic evidence plan:
  - Prediction or clause under test: frontend.log 或直接启动应捕获同一确定性错误。
  - Signal: frontend.log 的异常与前台 Vite 退出码。
  - Capture method: 读取启动日志，并在相同源码和环境下短时前台启动。
  - Event name or marker:
    - frontend process stderr
  - Correlation keys:
    - 2026-07-19 04:21 startup attempt
  - Differentiates from:
    - H-002 launcher 源码或环境解析错误
  - Supports if:
    - 日志与直接启动都显示同一运行时或依赖错误。
  - Refutes if:
    - 直接启动健康且 daemon 日志显示使用了错误源码/环境。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-001
  - E-003
  - E-004
  - E-006
  - E-007
- Conclusion: 当前用户的 inotify watches 几乎耗尽，Vite 无法为源码创建 watcher 并以 ENOSPC 退出。
- Repair design readiness: ready；部署先显式使用 polling，永久修复为提升内核 watch 限额。
- Next step: closed
- Blocker:
  - none
- Close reason:
  - not closed

## Hypothesis H-002: launcher 使用了非预期源码或环境
- Status: refuted
- Parent: P-001
- Claim: daemon 未继承预期的 ORNNLAB_SOURCE/ORNNLAB_HOME，因而从错误目录或错误数据环境启动前端。
- Layer: interaction
- Factor relation: single
- Depends on:
  - none
- Rationale:
  - Node launcher 与 Python CLI 是不同入口，daemon 又会脱离当前终端派生子进程。
- Falsifiable predictions:
  - If true: state/log 中记录的 source、cwd 或环境路径不是 `/home/zhangxu/HarnessLab` 与恢复数据目录。
  - If false: state/log/code 路径证明 daemon 继承并使用了预期路径。
- Diagnostic evidence plan:
  - Prediction or clause under test: daemon 记录的源码、cwd 和数据路径应与命令传入值比较。
  - Signal: state.json、daemon/backend/frontend 日志和 spawn 代码路径。
  - Capture method: 读取运行时状态与 launcher spawn 实现。
  - Event name or marker:
    - dev_service.start_requested
  - Correlation keys:
    - 2026-07-19 04:21 startup attempt
  - Differentiates from:
    - H-001 前端运行时或依赖失败
  - Supports if:
    - 运行时证据显示 source/cwd/home 与传入值不一致。
  - Refutes if:
    - 运行时和代码证据均显示预期路径被完整继承。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-002
  - E-005
- Conclusion: daemon 日志和 spawn 实现均指向当前仓库，且后端从相同环境成功启动并通过健康检查。
- Repair design readiness: blocked until Status is confirmed and Evidence gate is satisfied
- Next step: closed
- Blocker:
  - none
- Close reason:
  - not closed

## Evidence E-001: 初始启动症状
- Related hypotheses:
  - H-001
- Direction: neutral
- Type: reproduction
- Source: Node launcher `dev start`
- Prediction or plan link:
  - H-001 前端在 ready 前非零退出的预测
- Matched signal:
  - frontend exited before becoming ready
- Correlation keys:
  - 2026-07-19 04:21 startup attempt
- Raw content:
  ```text
  frontend exited before becoming ready
  Rerun `ornnlab install` after fixing the issue; bootstrap will retry incomplete stages.
  ```
- Interpretation: 证实症状发生，但尚不能区分运行时失败与 launcher 路径错误。
- Time: 2026-07-19 04:21

## Evidence E-002: 启动前端口未占用
- Related hypotheses:
  - H-002
- Direction: neutral
- Type: environment
- Source: `ss -ltnp | rg ':5173|:8765'`
- Prediction or plan link:
  - P-001 已知事实与替代原因排除
- Matched signal:
  - no listeners
- Correlation keys:
  - 2026-07-19 pre-start
- Raw content:
  ```text
  （无输出）
  ```
- Interpretation: 默认端口冲突不是本次启动失败的前置原因。
- Time: 2026-07-19 04:20

## Evidence E-003: frontend 日志命中 inotify ENOSPC
- Related hypotheses:
  - H-001
- Direction: supports
- Type: diagnostic-log
- Source: `~/.ornnlab/dev-service/logs/frontend.log`
- Prediction or plan link:
  - H-001 日志应捕获确定性运行时错误
- Matched signal:
  - ENOSPC: System limit for number of file watchers reached
- Correlation keys:
  - 2026-07-19 04:21 startup attempt
- Raw content:
  ```text
  Error: ENOSPC: System limit for number of file watchers reached, watch '/home/zhangxu/HarnessLab/frontend/src/api/dataMode.ts'
  ```
- Interpretation: 前端在 ready 前退出的直接机制是无法创建新的 inotify watch。
- Time: 2026-07-19 04:23

## Evidence E-006: polling 环境被 daemon 和前端继承
- Related hypotheses:
  - H-001
- Direction: supports
- Type: probe
- Source: `/proc/<daemonPid>/environ` 与 `/proc/<frontendPid>/environ` 的定向字段检查
- Prediction or plan link:
  - 修复验证要求 polling 配置必须进入实际守护进程和前端进程
- Matched signal:
  - daemon 与 frontend 均含 CHOKIDAR_USEPOLLING=true、CHOKIDAR_INTERVAL=500 和预期源码/数据路径
- Correlation keys:
  - 2026-07-19 04:25 running service
- Raw content:
  ```text
  daemon {CHOKIDAR_INTERVAL: 500, ORNNLAB_SOURCE: /home/zhangxu/HarnessLab, CHOKIDAR_USEPOLLING: true, ORNNLAB_HOME: /home/zhangxu/.ornnlab/data}
  frontend {CHOKIDAR_INTERVAL: 500, ORNNLAB_SOURCE: /home/zhangxu/HarnessLab, CHOKIDAR_USEPOLLING: true, ORNNLAB_HOME: /home/zhangxu/.ornnlab/data}
  ```
- Interpretation: workaround 只作用于 OrnnLab 进程，并已进入实际运行链路。
- Time: 2026-07-19 04:25

## Evidence E-007: 用户级入口生命周期与健康检查通过
- Related hypotheses:
  - H-001
- Direction: supports
- Type: fix-validation
- Source: `~/.local/bin/ornnlab`；从 `/tmp` 执行 path、stop、start、status
- Prediction or plan link:
  - P-001 fix criteria：原始启动成功且全部进程和健康检查为 Running/true
- Matched signal:
  - path 指向当前仓库；stop/start 成功；daemon/backend/frontend alive 与 healthy 全为 true
- Correlation keys:
  - daemonPid 3644144；2026-07-19 04:26 lifecycle run
- Raw content:
  ```text
  /home/zhangxu/HarnessLab
  OrnnLab dev service is stopped.
  OrnnLab dev service is running.
  status: running
  daemonAlive: true
  backendAlive: true
  frontendAlive: true
  backendHealthy: true
  frontendHealthy: true
  ```
- Interpretation: 原始症状在持久用户入口下不再发生，部署验收条件满足。
- Time: 2026-07-19 04:26

## Evidence E-008: 构建与 launcher 全量回归通过
- Related hypotheses:
  - H-001
- Direction: supports
- Type: fix-validation
- Source: `npm --prefix frontend run build`、`CHOKIDAR_USEPOLLING=true npm run test:launcher`、实际服务 status/live 断言
- Prediction or plan link:
  - P-001 fix criteria：polling 环境下开发启动链路和已部署服务均稳定
- Matched signal:
  - Vite 构建成功；launcher 27/27 通过；实际服务保持 running 且代理 live 为 ok
- Correlation keys:
  - 2026-07-19 04:30 final validation
- Raw content:
  ```text
  ✓ built in 114ms
  tests 27
  pass 27
  fail 0
  deployed service remains healthy: ok
  frontend proxy live assertion: ok
  ```
- Interpretation: polling 配置覆盖真实部署和 launcher API-mode 回归，原始 ENOSPC 启动失败不再出现。
- Time: 2026-07-19 04:30

## Evidence E-004: inotify 限额与独立复现
- Related hypotheses:
  - H-001
- Direction: supports
- Type: reproduction
- Source: `sysctl fs.inotify.*`、`/proc/*/fdinfo` 和直接执行 `npm run dev`
- Prediction or plan link:
  - H-001 独立启动应出现相同错误，且系统 watch 占用应接近上限
- Matched signal:
  - 65,402 / 65,536 watches；直接启动 exit 1 并产生相同 ENOSPC
- Correlation keys:
  - 2026-07-19 04:23 diagnostic run
- Raw content:
  ```text
  fs.inotify.max_user_watches = 65536
  uid=1000 instances=58 watches=65402
  watches=60021 pid=15458 instances=3 comm=code
  exit=1
  Error: ENOSPC: System limit for number of file watchers reached
  ```
- Interpretation: 限额耗尽和同错误独立复现共同确认 H-001，并把 Node 版本不兼容降级为非根因。
- Time: 2026-07-19 04:23

## Evidence E-005: launcher 路径正确且后端健康
- Related hypotheses:
  - H-002
- Direction: refutes
- Type: code-location
- Source: `lib/dev-daemon.js`、backend.log、state.json
- Prediction or plan link:
  - H-002 若为真，运行时应使用非预期 cwd/source/home
- Matched signal:
  - frontend cwd 为 sourceDir/frontend；错误路径位于当前仓库；后端 live 返回 200
- Correlation keys:
  - 2026-07-19 04:21 startup attempt
- Raw content:
  ```text
  cwd: path.join(sourceDir, "frontend")
  INFO: 127.0.0.1 - "GET /api/webui/v1/system/live HTTP/1.1" 200 OK
  ```
- Interpretation: daemon 使用了预期源码与环境，路径偏差不能解释前端 ENOSPC。
- Time: 2026-07-19 04:23
