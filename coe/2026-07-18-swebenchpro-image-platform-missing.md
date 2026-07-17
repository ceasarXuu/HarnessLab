# Problem P-001

- 状态：fixed
- 症状：本地 Dataset `swebenchpro@1.0` 的 Task 运行环境显示 Docker 镜像为 `-`，无法展示镜像架构。
- 期望：解析 Task 实际声明的容器镜像来源，并展示 Registry 返回的平台信息。
- 影响：用户无法从 Dataset 详情识别该运行环境的基础镜像及 `linux/amd64` 架构。
- 已知事实：该 Dataset 的 731 个 Task 均通过 `environment/Dockerfile` 声明一个 `FROM jefzda/sweap-images:...`，`task.toml` 未设置 `environment.docker_image`。
- 修复标准：结构化区分 Harbor 预构建环境镜像与 Dockerfile 基础镜像；真实样本可返回镜像引用及 `linux/amd64`；API、前端和 Storybook 使用统一契约。
- 结论：Task 环境改用结构化 `containerImages`，Dockerfile 外部 `FROM` 已进入现有 OCI 平台解析链路。

# Hypothesis H-001

- 状态：confirmed
- 声明：Task 摘要解析器只读取 `EnvironmentConfig.docker_image`，没有解析 Dockerfile `FROM`，导致平台解析器收不到 swebenchpro 的镜像引用。
- 预测：真实 Task API 返回 `definitions=[dockerfile]`、`dockerImage=null`；直接查询其 `FROM` 镜像 Registry 能返回平台。
- 诊断证据计划：对比真实 Task API、Dockerfile 与 Registry manifest；用失败测试固定 Dockerfile 镜像来源契约。

# Evidence E-001

- 类型：runtime-state
- 对应：H-001
- 观察：真实 Task API 返回 `dockerImage=null`、`imagePlatforms=[]`，同时 Dockerfile 明确包含 `FROM jefzda/sweap-images:...`。
- 结论：支持 H-001。

# Evidence E-002

- 类型：registry-probe
- 对应：H-001
- 观察：使用现有 OCI Registry 解析器查询该 `FROM` 镜像，返回 `linux/amd64`。
- 结论：排除 Registry 不可访问或平台解析失败，根因位于镜像引用提取阶段。

# Evidence E-003

- 类型：failing-test
- 对应：H-001
- 观察：修复前新增的 Dockerfile 基础镜像提取测试和容器镜像列表平台回填测试稳定失败。
- 结论：测试直接捕获缺失的提取机制，而非仅验证 UI 文案。

# Evidence E-004

- 类型：fix-validation
- 对应：P-001
- 观察：对用户提供目录中的真实 Task 执行摘要解析和 Registry 查询，返回 `jefzda/sweap-images:...`、`source=dockerfile-base` 和 `platforms=[linux/amd64]`。
- 结论：原始 Dataset 症状已在真实数据上消失。

# Evidence E-005

- 类型：regression
- 对应：P-001
- 观察：Python 全量测试 117 passed、3 skipped；前端 107 tests passed；Ruff、类型检查、lint、生产构建、Storybook build 与 smoke test 全部通过。
- 结论：容器镜像契约重构未破坏已覆盖的其他资源和交互。

# Evidence E-006

- 类型：browser-validation
- 对应：P-001
- 观察：重启开发服务后，在 Codex Web Preview 展开 `swebenchpro@1.0` 的真实 Task，界面显示“Dockerfile 基础镜像”和 `linux/amd64`，控制台无 error。
- 结论：API 结果已正确进入正式前端并完成可见展示。
