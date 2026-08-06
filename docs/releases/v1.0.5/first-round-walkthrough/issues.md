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
- 结论：triaged（accepted，待修复）
- 备注：后端每次 `/api/webui/v1/system/health` 返回的 CPU 值不同（实测 11.1 → 13.1 → 12.2），探测本身正常；根因在 `frontend/src/api/hooks.ts:225` 的 `useSystemHealth` 单次加载。

### W1-02 System 页 GPU 占用模块提示异常
- 日期：2026-08-07
- 模块/页面：System 页 → 主机资源（GPU）
- 严重度：P2（展示）；底层为主机环境问题
- 现象：GPU 卡片显示异常状态，但本机 GPU（RTX 4070 Ti SUPER）硬件正常可用。
- 复现步骤：1. 打开 System 页；2. 观察 GPU 卡片状态与 `deviceCount=0`。
- 期望行为：能展示 GPU 真实状态，或至少给出可定位的错误原因。
- 实际行为：后端执行 `nvidia-smi --query-gpu=utilization.gpu` 退出码 18，stderr 为 `Failed to initialize NVML: Driver/library version mismatch`（NVML 库 580.173 与内核驱动模块版本不一致）。探测如实返回 `error`，但 UI 只显示「异常」，未透传原因。
- 结论：triaged（OrnnLab 探测行为符合设计；主机存在 NVIDIA 驱动用户态/内核模块版本不匹配）
- 备注：`system_health_probe.py:221` `_gpu_component` 捕获 `CalledProcessError` 后仅置 `state=error`，未带上 stderr；`lspci` 确认 GPU 设备存在。环境侧建议重启主机或重载 NVIDIA 内核模块以消除驱动版本不匹配；产品侧建议后续把 nvidia-smi 错误信息透传到 UI（P3）。

<!-- 新问题追加到此处，编号递增 -->
