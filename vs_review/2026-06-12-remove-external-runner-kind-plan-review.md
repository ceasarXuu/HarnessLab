# 对抗性审查：移除 ExternalRunnerKind 清理计划

- Review Target: docs/plans/2026-06-12-remove-external-runner-kind-plan.md
- Review Type: Architecture plan review (pre-implementation)
- Date: 2026-06-12
- Status: Closure Pending (all blocking findings accepted, plan revision in progress)

## Review Input Packet

### Objective
从 harnesslab adapter 层彻底移除 `ExternalRunnerKind` enum 及其作为 adapter 分发机制的所有遗留使用，实现纯 protocol-id 驱动的 adapter 架构。

### Review Target
架构实施计划文档：`docs/plans/2026-06-12-remove-external-runner-kind-plan.md`

相关代码位置（供 reviewer 直接读取验证）：
- `crates/harnesslab-core/src/benchmark.rs` — ExternalRunnerKind enum 定义
- `crates/harnesslab-core/src/runtime.rs` — RuntimePreflightReport.runner_kind
- `crates/harnesslab-core/src/adapter_protocol.rs` — AdapterProtocolAuthority.legacy_runner_kind
- `crates/harnesslab-adapters/src/protocol_registry.rs` — binding legacy_runner_kind
- `crates/harnesslab-adapters/src/registry.rs` — adapter_for_with_root
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs` — runtime_adapter_for, kind()
- `crates/harnesslab-cli/src/runner/external/runtime_authority.rs` — runtime_snapshot_source_ref(kind)
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs` — 35 处引用
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_test_support.rs` — external_task(kind)
- `xtask/src/no_branch_guard.rs` — FORBIDDEN_TOKENS

### Change Introduction
计划分 5 阶段从代码库中删除 ExternalRunnerKind：
1. Runtime adapter 分发层去 Kind 化
2. Core types 去 Kind 化
3. Adapter registry 去 Kind 化
4. 测试层全面更新
5. 最终删除 enum 与 guard 更新

### Risk Focus
- 计划是否遗漏了隐含的 ExternalRunnerKind 引用路径
- 测试辅助函数 `external_task(kind)` 的签名变更对测试生态的冲击是否被低估
- `runtime_adapter_for_task()` 移除 fallback 后，未设置 `runtime_binding` 的测试路径是否会大面积失败
- `ExternalRunnerSpec` 的非 kind 字段（dataset_path, source_path, agent_timeout_sec）与 `TaskRuntimeBinding` 的关系是否被正确梳理
- 序列化格式变更对快照测试的影响
- 5 个阶段的依赖关系是否真的如计划所述是线性的，是否存在可以并行或需要重排的部分

### Assumptions to Attack
- "无生产环境数据需要兼容" — 是否有集成测试、CI 缓存、或其他非生产但持久化的数据依赖旧格式？
- "Protocol binding 已成为唯一权威" — 当前代码中是否所有 external task 创建路径都已设置 runtime_binding？
- "ExternalRunnerSpec 的非 kind 字段仍有价值" — 这些字段是否完全可以被 TaskRuntimeBinding 替代，从而进一步简化？
- "每阶段作为一个独立 commit" — 阶段 1 移除 trait 方法后，阶段 2 之前代码是否能编译？

### Adversarial Lenses
- **Architecture**: 删除 ExternalRunnerKind 后是否引入了新的隐式依赖（如字符串比较替代 enum 匹配）？
- **Maintenance**: 未来新增 adapter 时，开发者是否还需要了解这段历史？
- **Testing**: 测试更新的工作量（15-20 个文件）是否被低估？是否存在测试逻辑需要重写而不仅是签名变更？
- **State/Data**: ExternalRunnerSpec 的序列化格式变更是否会影响 replay 测试？

### Verification Status
计划尚未实施，无验证数据。

### Reviewer Instructions
- Fresh session，不继承主 agent 上下文
- 直接读取计划文档和上述代码文件
- 不修改任何文件
- 对计划的假设、遗漏风险、执行可行性进行对抗性挑战
- 对 blocking findings 提供反例和证据路径

---

## Reviewer Launch Record

- **Reviewer role**: Architecture plan adversarial reviewer
- **Internal subagent mechanism**: general_purpose_task
- **Session freshness**: Fresh session, no main-agent context inherited
- **Input packet**: Full review navigation packet above
- **Timeout policy**: Complex (15-25 minutes)
- **Status**: Completed

---

## Reviewer Output

### Summary
该计划的方向（彻底协议化、删除 legacy runner kind）是正确的，但**严重低估了持久化 schema 变更与集成测试契约的连锁反应**。核心问题在于：`runner_kind` 不仅是一个内存中的枚举，它已被写入**外部运行时快照的指纹计算、replay 校验逻辑、集成测试断言、以及事件日志**。若直接删除，历史运行的 replay 将不可逆地断裂，且多个阶段之间存在**时序死锁**（阶段 1 的验证依赖阶段 4 的测试更新）。计划列出的 8–12 小时工作量在 fingerprint 兼容性、guard/claim 基础设施同步、以及集成测试重新录制面前过于乐观。

### Blocking Findings

#### 1. Runtime Snapshot Fingerprint 包含 `runner_kind`，移除将永久破坏历史 Replay
- **证据路径**：
  - `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs:398–408`（`runtime_fingerprint` 计算包含 `request.runner_kind`）
  - `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs:424–433`（`public_fingerprint` 同样包含）
  - `crates/harnesslab-cli/tests/support/runtime_snapshot.rs:233–268`（测试辅助函数重写指纹时同样包含 `runner_kind`）
  - `crates/harnesslab-cli/src/runner/replay.rs:282–284`（replay 对比 `private["runner_kind"]` 与当前计算的 `runner_kind`）
- **失败场景**：旧运行目录中的 `external-runtime.private.json` 与 `external-runtime.public.json` 已按包含 `runner_kind` 的指纹归档。新代码删除该字段后，重新计算指纹将与旧值不匹配，导致**所有历史外部任务 replay 被阻断**。
- **建议修复**：在移除前决定兼容策略——要么用 `adapter_id` 替代 `runner_kind` 参与指纹计算（保持哈希不变性），要么升级快照 `schema_version` 并显式区分新旧 replay 路径。

#### 2. 阶段 1 移除 Fallback 与测试辅助函数未更新存在时序死锁
- **证据路径**：
  - `crates/harnesslab-cli/src/runner/external/runtime_adapter_test_support.rs:74`：`external_task(kind)` 的 `runtime_binding` 硬编码为 `None`
  - `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`：全文调用 `external_task("...", ExternalRunnerKind::...)` 约 20 余处
- **失败场景**：阶段 1 计划移除 `runtime_adapter_for_task()` 的 fallback（当 `runtime_binding` 缺失时回退到 `external_runner.kind`），并声称验证通过 `cargo test -p harnesslab-cli runner::external::runtime_adapter_tests`。但 `runtime_adapter_tests` 的测试数据全部由 `external_task()` 生成，**均未设置 `runtime_binding`**。移除 fallback 后这些测试会在阶段 1 立即 panic，导致阶段 1 根本无法验证通过。
- **建议修复**：将 `external_task()` 的改造（添加 `runtime_binding`）作为阶段 1 的**前置条件**或阶段 1 的第一步，而不是放到阶段 4。

#### 3. `runtime_snapshot_source_ref` 存在按 `kind` 分支的不可替代业务逻辑
- **证据路径**：
  - `crates/harnesslab-cli/src/runner/external/runtime_authority.rs:51–62`
- **失败场景**：`TerminalBench` 直接读取 `source_path`，而 `SweBenchPro` 调用 `runtime_source_ref()`（内部会校验 protocol binding 与 legacy path 的一致性）。计划说“合并逻辑到 `runtime_source_ref(task)`”，但**未说明如何保留两者的行为差异**。若直接删除 `kind` 参数，TerminalBench 可能意外引入 protocol binding 校验，或 SweBenchPro 失去校验。
- **建议修复**：明确替代分支策略，例如按 `task.runtime_binding.as_ref().map(|b| b.authority.adapter_id.as_str())` 分支，或确认 TerminalBench 也可以统一走 protocol binding 校验。

#### 4. 集成测试契约硬编码 `runner_kind` 字符串
- **证据路径**：
  - `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs:380–381`：断言 `public["runner_kind"] == "swe_bench_pro"`
  - `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs:42–43`：断言 `public["runner_kind"] == "terminal_bench"`
  - `crates/harnesslab-cli/tests/external_smoke_contract.rs:391`：断言事件日志包含 `runner_kind=SweBenchPro`
- **失败场景**：这些是**持久化 JSON schema 的端到端契约测试**。删除 `runner_kind` 字段后，这些测试不是编译错误，而是运行时断言失败。需要系统性重写契约预期。
- **建议修复**：将 snapshot schema 变更（`runner_kind` → `adapter_id` 或删除）纳入计划，并同步更新所有集成测试的断言。

#### 5. Guard 与 Claims 基础设施需要同步更新，否则阶段 5 验证失败
- **证据路径**：
  - `xtask/src/no_branch_guard.rs:54–68`：`LEGACY_SHIM_FILES` 列表包含将被清理的文件（如 `runtime_adapter.rs`, `runtime_snapshot.rs` 等）
  - `xtask/src/no_branch_guard.rs:6–40`：`FORBIDDEN_TOKENS` 包含 `"ExternalRunnerKind"`, `"TerminalBench"`, `"SweBenchPro"` 等
  - `xtask/src/adapter_claims.rs:249–411`：`active_route_spec` 中的 `file_patterns` 指向将被移除或修改的文件
  - `xtask/src/no_branch_guard.rs:393–450`：`adapt_protocol_008_allowlist_inventory_is_review_locked` 测试硬编码了上述列表
- **失败场景**：如果阶段 5 只删除 enum 而不更新 guard 的允许列表，`cargo run -p xtask -- verify-no-branch-guard` 和 `adapter_claims` 测试会因 `LEGACY_SHIM_FILES` / `FORBIDDEN_TOKENS` 与代码实际状态不符而失败。
- **建议修复**：将 `no_branch_guard.rs` 和 `adapter_claims.rs` 的列表更新纳入计划，并视为阶段 5 的必需步骤。

#### 6. Adapter Registry 测试在阶段 3 就会断裂（阶段 4 才修复）
- **证据路径**：
  - `crates/harnesslab-adapters/src/protocol_contract_tests.rs:345–365`：`adapt_protocol_010` 测试断言 `tb_descriptor.binding.legacy_runner_kind.is_some()`
  - `crates/harnesslab-adapters/src/protocol_registry.rs:446–449`：测试断言 `mismatched_legacy` 会触发 `legacy_runner_kind_mismatch`
- **失败场景**：阶段 3 的验证命令包含 `cargo test -p harnesslab-adapters`，但测试更新被安排在阶段 4。阶段 3 执行时上述测试会立即失败。
- **建议修复**：将 `protocol_contract_tests.rs` 和 `protocol_registry.rs` 中的相关测试更新并入阶段 3，或调整验证命令为仅编译检查（`cargo check`）直到阶段 4。

### Non-blocking Risks

1. **旧快照反序列化兼容性**：未发现 `#[serde(deny_unknown_fields)]`，因此删除 `runner_kind` 后旧 JSON 反序列化不会直接报错。但 replay 逻辑在 `replay.rs:282–284` 会比较旧快照中的 `runner_kind` 字段，若当前代码不再提供该值， mismatch 检测会失败。需在 replay 层显式处理新旧格式。
2. **`ExternalRunnerSpec` 非 `kind` 字段的后续命运**：`dataset_path`、`source_path`、`agent_timeout_sec` 仍被多处读取（如 `terminal_bench_adapter.rs:369` 读取 `agent_timeout_sec`）。计划正确地保留了这些字段，但应在开放问题 1 中明确它们最终是否迁移到 `TaskRuntimeBinding`。
3. **事件名称保留 `external_runner` 前缀**：计划正确地决定本次不触及事件名称（开放问题 3）。这是合理的范围控制。

### Underestimated Work

| 领域 | 计划估计 | 实际评估 | 理由 |
|------|---------|---------|------|
| 受影响文件数 | 30–45 | 35–50+ | 仅 `.rs` 文件就有 22+ 处直接引用，加上 `xtask/`  guard/claims、测试 support 文件、集成测试 |
| 测试文件更新 | 15–20 文件 / 3–4 小时 | 25+ 文件 / 6–8 小时 | `runtime_adapter_tests`（35 处引用）、3 个集成测试契约、`protocol_contract_tests`、`data_contract_tests`、`swe_bench_pro_tests`、`cleanup_tests`、`runtime_anchor` 测试等 |
| 阶段 1 工作量 | 2–3 小时 | 4–6 小时 | 不仅要改 trait，还要同步重写 `runtime_snapshot_source_ref` 分支、更新事件日志输出、处理 fingerprint 兼容性决策 |
| 阶段 5 工作量 | 1 小时 | 2–3 小时 | 需同步更新 `no_branch_guard.rs`、`adapter_claims.rs`、以及 `frozen_execution_files.rs`（若存在）等元数据文件 |

### Recommendations

1. **增加阶段 0：Fingerprint 与 Schema 兼容性决策**
2. **将 `external_task()` 辅助函数更新前置到阶段 1**
3. **明确 `runtime_snapshot_source_ref` 的替代分支策略**
4. **将 snapshot schema 升级纳入阶段 2**
5. **将 Guard/Claims 更新设为阶段 5 的正式步骤**
6. **将 `protocol_contract_tests` 和 `protocol_registry` 测试更新并入阶段 3**
7. **事件日志输出中的 `runner_kind` 应在阶段 1 同步替换为 `adapter_id`**

---

## Main Agent Triage

### Blocking Findings Response

| # | Finding | Triage | Action |
|---|---------|--------|--------|
| 1 | Runtime Snapshot Fingerprint 包含 `runner_kind` | **accept** | 增加阶段 0 进行 fingerprint/schema 兼容性决策；阶段 2 中用 `adapter_id` 替换 `runner_kind` 参与 fingerprint 计算，保持旧 replay 可用；replay 校验逻辑同步更新 |
| 2 | 阶段 1 移除 Fallback 与测试辅助函数未更新存在时序死锁 | **accept** | 将 `external_task()` 和 `protocol_bound_terminal_task()` 的更新作为阶段 1 步骤 0（前置条件）；所有测试辅助函数必须先构造带 `runtime_binding` 的 TaskPlan |
| 3 | `runtime_snapshot_source_ref` 存在按 `kind` 分支的不可替代业务逻辑 | **accept** | 在阶段 1 中明确：删除 `kind` 参数后，统一按 `task.runtime_binding` 是否存在走 protocol 路径；若不存在则直接读取 `external_runner.source_path`（不校验 binding）；TerminalBench 和 SweBenchPro 的行为差异通过 adapter 自身的 `execute()` 实现，不在 source_ref 层分支 |
| 4 | 集成测试契约硬编码 `runner_kind` 字符串 | **accept** | 将集成测试契约更新纳入阶段 4；`runner_kind` 字段在 snapshot JSON 中替换为 `adapter_id`；事件日志中的 `runner_kind` 替换为 `adapter_id` |
| 5 | Guard 与 Claims 基础设施需要同步更新 | **accept** | 阶段 5 明确列出：`xtask/src/no_branch_guard.rs`（移除 `ExternalRunnerKind`/`TerminalBench`/`SweBenchPro` 相关 token，更新 `LEGACY_SHIM_FILES`）、`xtask/src/adapter_claims.rs`（更新 `file_patterns`）、`tests/FROZEN_SELECTOR_MANIFEST.toml`（如有受影响 selector） |
| 6 | Adapter Registry 测试在阶段 3 就会断裂 | **accept** | 将 `protocol_contract_tests.rs` 和 `protocol_registry.rs` 中相关测试更新并入阶段 3；阶段 3 验证命令保持不变 |

### Non-blocking Risks Response

| # | Risk | Triage | Action |
|---|------|--------|--------|
| 1 | 旧快照反序列化兼容性 | **accept** | 已纳入阶段 0 决策；replay 层在阶段 2 同步处理新旧格式 |
| 2 | `ExternalRunnerSpec` 非 `kind` 字段的后续命运 | **defer** | 标记为 ADAPT-DATA-000 后续工作，本次清理不涉及 |
| 3 | 事件名称保留 `external_runner` 前缀 | **accept** | 维持原计划开放问题 3 的决定，本次不触及 |

---

## Closure Status

- **Blocking findings**: 6 accepted, 0 rejected, 0 deferred
- **Plan revision**: In progress — 主 agent 正在根据所有 accepted findings 修订计划文档
- **Next step**: 修订后的计划将重新提交用户确认；用户确认后进入实施阶段
