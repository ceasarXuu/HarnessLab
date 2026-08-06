# v1.0.5 第一轮走查问题台账

- 走查期间：2026-08-07 起
- 规则：每次反馈新增一条记录；每条唯一 ID（W1-01、W1-02…）；状态变更时更新状态与日期；
  收尾时在[README](README.md)汇总并与工程计划同步。

## 问题列表

### W1-01 System 页 CPU 占用数字不更新
- 日期：2026-08-07
- 模块/页面：System 页 → 主机资源（CPU）
- 严重度：P2
- 现象：停留在 System 页时 CPU 占用百分比长时间不变，仅在页面重新加载或触发操作刷新后可能变化。
- 复现步骤：1. 打开 System 页；2. 观察 CPU 数值数秒。
- 期望行为：CPU 占用应定期刷新，反映实时负载。
- 实际行为：数值是页面挂载时的单次快照；`useSystemHealth` 只在 mount 时加载一次，System 页未使用 `usePollingRefresh`。
- 结论：closed（accepted，已修复 2026-08-07）
- 备注：根因在 `frontend/src/api/hooks.ts:225` 的 `useSystemHealth` 单次加载。修复：`App.tsx` 增加
  `usePollingRefresh(systemResource.refresh, route.page === 'system', 2_000)`，System 页停留时每 2s
  刷新健康数据。回归测试 `App.test.tsx` 新增定时刷新断言；真实 API 验证 CPU 值随轮询变化
  （11.1% → 15.0% → 20.3%）。

### W1-02 System 页 GPU 占用模块提示异常
- 日期：2026-08-07
- 模块/页面：System 页 → 主机资源（GPU）
- 严重度：P2（展示）；底层为主机环境问题
- 现象：GPU 卡片显示异常状态，但本机 GPU（RTX 4070 Ti SUPER）硬件正常可用。
- 复现步骤：1. 打开 System 页；2. 观察 GPU 卡片状态与 `deviceCount=0`。
- 期望行为：能展示 GPU 真实状态，或至少给出可定位的错误原因。
- 实际行为：后端执行 `nvidia-smi --query-gpu=utilization.gpu` 退出码 18，stderr 为 `Failed to initialize NVML: Driver/library version mismatch`（NVML 库 580.173 与内核驱动模块版本不一致）。探测如实返回 `error`，但 UI 只显示「异常」，未透传原因。
- 结论：closed（主机侧 2026-08-07 重启后 nvidia-smi 恢复，GPU 显示 normal / 1 device；产品侧 P3 改进已实现）
- 备注：`system_health_probe.py:221` `_gpu_component` 捕获 `CalledProcessError` 后仅置 `state=error`，未带上 stderr。修复：新增 `_probe_error_detail`，失败时把 nvidia-smi 错误原因写入组件 `error` 字段（兜底用退出码），UI 在 GPU 卡片内展示该原因；前端 contract/domain/mock 同步新增 `error` 字段。回归测试覆盖 error 详情、stderr 为空回退退出码、not-detected 无 error、正常读取。重启后验证 `/api/webui/v1/system/health` 返回 `resource-gpu: normal / 5.0% / 1 device / error=None`。

### W1-03 质量门偶发失败（环境代理抖动，非产品问题）
- 日期：2026-08-07
- 模块/页面：测试基础设施（`test_webui_external_dataset_storage_routes_preserve_files`）
- 严重度：P3（观察项）
- 现象：完整门禁 `scripts/test-after-change-web.sh` 中 pytest 偶发 1 个失败：DELETE external dataset 请求阶段 `httpx.RemoteProtocolError: Server disconnected`，断言层实际是响应未收到。
- 复现步骤：全套件运行时偶发（本次 1/10 左右）；单测隔离 5/5 通过，改动后 9/9、基线 4/4 全量通过。
- 期望行为：门禁稳定通过。
- 实际行为：traceback 走 `httpcore/_async/http_proxy.py`，本机 shell 存在 Clash 代理变量
  `http_proxy/http(s)_proxy=http://127.0.0.1:7890`，与 TestClient httpx 连接在代理抖动时中断，
  属于环境性、时序性偶发，与 W1-01/W1-02 改动路径无关。
- 结论：triaged（观察项，未修代码；后续如再复现，优先排查本机代理对 TestClient 的影响，必要时在测试环境显式剥离代理变量）
- 备注：见 `docs/releases/v1.0.5/engineering-plan.md` 运行经验条目。

<!-- 新问题追加到此处，编号递增 -->
