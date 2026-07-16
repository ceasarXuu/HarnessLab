# Problem P-001

- Symptom: 用户点击 `run-75721687f051` 的“恢复”后界面没有可见反馈。
- Expected: 只有 Harbor 可恢复的 Job 展示恢复入口；恢复失败时抽屉内显示明确结果并刷新 Job 状态。
- Environment: 本地 API 模式，OrnnLab dev service，Harbor 0.13.2。
- Fix criteria: 不可恢复 Job 不展示恢复入口；可恢复 Job 的 Operation 失败可见；终态后列表和详情刷新。
- Conclusion: 后端恢复前置条件未进入 Job DTO，前端仅按 `failed/interrupted` 状态展示按钮；前端同时遗漏 Operation terminal failure 的呈现与刷新。

# Hypothesis H-001

- Claim: 原 Job 在 Harbor 写出原生 `config.json` 前失败，因此 Harbor resume 无法启动。
- Prediction: resume Operation 已创建，但 Harbor CLI 返回非零；Job 目录不存在 `config.json`。
- Diagnostic evidence plan: 对照 Operation、后端日志和 Job 目录文件，三者必须指向同一个缺失前置条件。
- Status: confirmed。

# Evidence E-001

- Operation `op-36ae1181ee16` 类型为 `resume-job`，状态为 `failed`，错误为 `harbor job resume exited with 1`。
- Supports: H-001 的“请求已到后端且 Harbor CLI 失败”预测。

# Evidence E-002

- 后端日志记录 `ValueError: Config file not found: /Volumes/XU-1TB-NPM/projects/ornnlab_jobs/test/config.json`。
- Supports: H-001 的“缺失 Harbor 原生配置”预测。

# Evidence E-003

- Job 目录只有 `harbor.config.json`、`harbor.capability.json` 和 `job.log`；`job.log` 内容为 Docker daemon 未运行，没有原生 `config.json` 或 Trial 目录。
- Supports: H-001，并排除“路径解析选错已有原生子目录”的替代解释。

# Hypothesis H-002

- Claim: 前端未展示 Operation 自身的 failed 状态，因此用户观察到“没有反应”。
- Prediction: `useOperation` 能轮询到 failed Operation，但页面只读取请求/轮询错误，不读取 `operation.error`。
- Diagnostic evidence plan: 追踪 App 的 `runJobAction`、`useOperation` 和 Jobs 页面错误渲染链路。
- Status: confirmed。

# Evidence E-004

- `runJobAction` 正确提交 `client.resumeJob`；`ResourceStatus` 只读取 `jobOperation.error`，而 Operation 业务失败位于 `jobOperation.operation.error`。
- Supports: H-002，并解释请求成功后界面无失败反馈。

# Evidence E-005

- 当前刷新 effect 只在 Operation `completed` 时刷新 Jobs；`failed` 和 `cancelled` 终态不刷新。
- Supports: H-002 的状态不同步机制。

# Fix F-001

- 后端 Job DTO 新增 `canResume`，仅在 Job 为 `failed/interrupted` 且解析后的 Harbor Job 目录存在原生 `config.json` 时为真。
- 恢复接口复用同一前置条件；缺少原生配置时直接返回 `422 INVALID_REQUEST`，不再创建必然失败的 Operation。
- 前端只根据 `canResume` 展示恢复入口；Operation 进入任意终态后刷新 Jobs，恢复失败在对应 Job 抽屉内展示产品化错误。

# Verification V-001

- 后端回归覆盖无原生配置时 `canResume=false`、恢复请求被拒绝，以及配置出现后 `canResume=true`。
- 组件回归覆盖可恢复和不可恢复的失败 Job；App 回归覆盖 failed Operation 的抽屉反馈、底层错误隐藏和列表刷新。

# Prevention P-001

- 资源动作是否可用必须由后端 DTO 暴露能力事实，不得由前端依据状态枚举推断文件系统或 Harbor 前置条件。
- 异步 Operation 的 `completed/failed/cancelled` 都属于需要刷新资源的终态；业务失败与请求失败分开呈现。
