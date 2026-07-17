# Problem P-001

- 状态：fixed
- 症状：Dataset 下载期间列表刷新后，下载中状态与进度消失。
- 期望：刷新列表或页面后，进行中的下载仍显示下载状态、当前进度和取消入口。
- 影响：用户无法判断下载是否仍在执行，也无法可靠取消。
- 已知事实：前端使用 `useOperation` 保存当前页面最近一次 operation；后端将 operation 持久化到 `webui_operations`。
- 修复标准：列表刷新与页面重新挂载后均可恢复进行中的 Dataset 下载 operation；完成后自动切换为已下载状态；同类异步状态不依赖瞬时组件内存。
- 结论：Dataset 读模型现从持久化 operation 合并 `downloading` 和进度；前端按资源状态轮询，组件重新挂载后可恢复。
- Resolution basis：H-001、H-002；E-001、E-002、E-003、E-004、E-005、E-006。
- 补充边界：服务启动时将遗留的活动 operation 对账为中断失败，避免异常退出后永久显示下载中。

# Hypothesis H-001

- 状态：confirmed
- 声明：Dataset 下载状态只存在于 `DatasetsPage` 的 `useOperation` 本地状态，页面刷新或重挂载后没有按资源恢复后端进行中的 operation，因此列表 DTO 的 `not-downloaded` 覆盖了下载状态。
- 预测：`useOperation` 初始化为空；后端 operation 表仍有 queued/running 记录；现有 API 只能按 operation ID 查询，前端刷新后已丢失该 ID。
- 诊断证据计划：检查 hook 初始化和 API contract；构造组件重挂载测试，验证同一后端 operation 在重新挂载后 UI 不显示 downloading。

# Hypothesis H-002

- 状态：confirmed
- 声明：后端 Dataset 列表没有合并 `webui_dataset_downloads` 或进行中 operation，导致每次列表刷新都将进行中 Dataset 返回为 `not-downloaded`。
- 预测：下载开始后，`list_datasets()` 对该 ref 仍返回 `not-downloaded`，尽管 operation 与 pending download 记录存在。
- 诊断证据计划：检查 `_remote_dto`、pending download 表和下载 API；增加服务/API 级复现测试捕获刷新期间 DTO。

# Evidence E-001

- 类型：code-path
- 对应：H-001 预测
- 观察：`useOperation` 的 operation 初始值固定为 `null`，只在当前组件提交 mutation 后保存 ID；API 仅支持按 operation ID 查询，没有重挂载后的恢复入口。
- 结论：支持 H-001。

# Evidence E-002

- 类型：code-path
- 对应：H-002 预测
- 观察：后端 `_remote_dto` 固定返回 `download.status = not-downloaded`，`list_datasets()` 与 `get_dataset()` 均未合并 `webui_operations` 中的 queued/running 下载记录。
- 结论：支持 H-002，并解释刷新响应为何覆盖临时前端状态。

# Evidence E-003

- 类型：failing-test
- 对应：H-001、H-002 诊断证据计划
- 观察：API 测试在 operation 为 running、progress 为 37 时实际返回 `not-downloaded`；前端 mock 在提交下载后也返回 `not-downloaded`。两个测试均在修复前稳定失败。
- 结论：确认 H-001 与 H-002 的联合机制，而非单纯渲染问题。

# Evidence E-004

- 类型：fix-validation
- 对应：P-001 修复标准
- 观察：新增 App 重挂载测试在开始下载并卸载整个应用后，以同一 client 重新挂载，Dataset 行恢复显示 `50%` 和“取消下载”。
- 结论：原始刷新/重挂载症状已消失。

# Evidence E-005

- 类型：regression
- 对应：P-001 修复标准
- 观察：Python 全量测试 116 passed、3 skipped；前端 107 tests passed；类型检查、lint、生产构建、Storybook build 与 smoke test 全部通过。
- 结论：资源状态契约扩展未破坏已覆盖的其他流程。

# Evidence E-006

- 类型：restart-recovery
- 对应：P-001 修复标准
- 观察：服务启动对账会把失去进程内任务的 `queued/running` WebUI operation 转为 `OPERATION_INTERRUPTED`。
- 结论：持久化活动状态不会在异常重启后变成永久伪状态。
