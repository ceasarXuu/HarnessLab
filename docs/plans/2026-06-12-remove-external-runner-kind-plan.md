# 清理计划：移除 ExternalRunnerKind 与 Legacy Runner Kind 兼容性补丁

- Status: Completed
- Created: 2026-06-12
- Updated: 2026-06-12 (implemented)
- Owner: harnesslab adapter protocol team
- Source request: 确认无生产环境数据需要兼容后，移除 ExternalRunnerKind 作为 adapter 分发机制的遗留补丁
- Review: [vs_review/2026-06-12-remove-external-runner-kind-plan-review.md](../../vs_review/2026-06-12-remove-external-runner-kind-plan-review.md)

## 1. 背景与动机

当前 adapter 层实现了 protocol-first 的通用协议（ADAPT-PROTOCOL-001..012），但 terminal-bench 和 swe-bench-pro 两个 adapter 的 binding 和 runtime adapter 中仍保留了 `ExternalRunnerKind` 作为向后兼容的 shim：

- `AdapterBindingDescriptor.legacy_runner_kind: Option<ExternalRunnerKind>`
- `BenchmarkRuntimeAdapter.kind() -> ExternalRunnerKind`
- `runtime_adapter_for(kind)` 通过 match kind 分发到具体 adapter
- `TaskPlan.external_runner: Option<ExternalRunnerSpec>` 中 `ExternalRunnerSpec` 包含 `kind: ExternalRunnerKind`

**关键观察**：目前运行过的任务均为测试任务，不存在需要兼容的生产环境历史数据。因此 legacy runner kind 的保留已从无用的兼容性补丁降级为纯技术债务，阻碍架构的彻底通用化。

## 2. 目标

1. 从 adapter 分发路径中彻底移除 `ExternalRunnerKind` 依赖，实现纯 protocol-id 驱动
2. 从 core 类型中移除或弱化 `ExternalRunnerKind`，消除 benchmark-specific 分支的温床
3. 保证 terminal-bench 和 swe-bench-pro 两个现有 benchmark 在清理后仍能完整运行（数据生命周期 + 运行时生命周期 + replay + report）
4. 保持 deterministic-sample adapter 作为无 legacy 的参考实现

## 3. 前提假设

- **无生产数据兼容需求**：所有历史 task plan、snapshot、report 均为测试生成，可接受破坏性 schema 变更
- **Protocol binding 已成为唯一权威**：所有新任务计划都通过 `TaskRuntimeBinding.authority.adapter_id` 标识 adapter
- **ExternalRunnerSpec 的非 kind 字段仍有价值**：`dataset_path`、`source_path`、`agent_timeout_sec` 仍被 runtime 读取，不能随 `kind` 一并删除

## 4. 影响范围分析

### 4.1 Core Types（harnesslab-core）

| 文件 | 涉及内容 | 影响程度 |
|------|---------|---------|
| `src/benchmark.rs` | `ExternalRunnerKind` enum 定义；`ExternalRunnerSpec.kind` 字段 | 高 |
| `src/runtime.rs` | `RuntimePreflightReport.runner_kind` | 高 |
| `src/adapter_protocol.rs` | `AdapterProtocolAuthority.legacy_runner_kind`；`legacy_runner_kind_authority()` | 高 |
| `src/model_tests.rs` | 测试断言 `external_runner` | 中 |

### 4.2 Adapter 层（harnesslab-adapters）

| 文件 | 涉及内容 | 影响程度 |
|------|---------|---------|
| `src/protocol_registry.rs` | `AdapterBindingDescriptor.legacy_runner_kind`；`ensure_legacy_mapping()` | 高 |
| `src/registry.rs` | `adapter_for_with_root()` match 分发 | 中 |
| `src/terminal_bench.rs` | 构造 `ExternalRunnerSpec` 时设置 `kind` | 中 |
| `src/swe_bench_pro.rs` | 构造 `ExternalRunnerSpec` 时设置 `kind` | 中 |
| `src/protocol_contract_tests.rs` | 测试断言 legacy_runner_kind | 高 |
| `src/data_contract_tests.rs` | 测试断言 | 低 |
| `src/swe_bench_pro_tests.rs` | 测试断言 | 低 |

### 4.3 CLI Runtime 层（harnesslab-cli）

| 文件 | 涉及内容 | 影响程度 |
|------|---------|---------|
| `src/runner/external/runtime_adapter.rs` | `BenchmarkRuntimeAdapter.kind()`；`runtime_adapter_for()`；`RuntimeCleanupTarget.runner_kind` | 高 |
| `src/runner/external/terminal_bench_adapter.rs` | `kind()` 实现 | 中 |
| `src/runner/external/swe_bench_pro_adapter.rs` | `kind()` 实现 | 中 |
| `src/runner/external/runtime_authority.rs` | `runtime_snapshot_source_ref(kind)` 按 kind 分支 | 高 |
| `src/runner/external.rs` | `task.external_runner.is_some()` 判断 external task；事件日志输出 `runner_kind` | 高 |
| `src/runner/replay.rs` | 比较 `task.external_runner` 与 `expected.external_runner`；对比 `private["runner_kind"]` | 高 |
| `src/runner/external/runtime_snapshot.rs` | `runtime_fingerprint` / `public_fingerprint` 包含 `runner_kind` | 高 |
| `src/runner/external/runtime_adapter_tests.rs` | 35 处 `ExternalRunnerKind` 引用 | 高 |
| `src/runner/external/runtime_adapter_test_support.rs` | `external_task(kind)` 辅助函数 | 高 |
| `src/runner/cleanup_tests.rs` | 构造 `ExternalRunnerSpec` | 中 |
| `src/runner/external/runtime_anchor.rs` | 构造 `ExternalRunnerSpec` | 中 |
| `tests/external_runtime_snapshot_contract.rs` | 集成测试契约断言 `runner_kind` | 高 |
| `tests/swe_runtime_snapshot_contract.rs` | 集成测试契约断言 `runner_kind` | 高 |
| `tests/external_smoke_contract.rs` | 集成测试契约断言事件日志 `runner_kind` | 高 |
| `tests/support/runtime_snapshot.rs` | 测试辅助函数重写指纹时包含 `runner_kind` | 高 |

### 4.4 Guard 与验证（xtask）

| 文件 | 涉及内容 | 影响程度 |
|------|---------|---------|
| `src/no_branch_guard.rs` | `FORBIDDEN_TOKENS` 包含 `"ExternalRunnerKind"` 等；`LEGACY_SHIM_FILES` 列表 | 高 |
| `src/adapter_claims.rs` | `active_route_spec` 的 `file_patterns` 指向将被修改的文件 | 中 |
| `tests/FROZEN_SELECTOR_MANIFEST.toml` | 如有受影响的 selector 需更新 | 低 |

## 5. 分阶段实施计划

### 阶段 0：Fingerprint 与 Schema 兼容性决策（前置决策，无代码变更）

**目标**：在写代码前确定 snapshot fingerprint 和 replay 的兼容策略，避免历史 replay 被不可逆破坏。

**决策项**：

0.1 **Fingerprint 字段替换策略**
   - `runtime_fingerprint` 和 `public_fingerprint` 的计算 JSON 中，将 `"runner_kind"` 替换为 `"adapter_id"`
   - 使用 `request.protocol_authority.as_ref().map(|a| a.adapter_id.as_str()).unwrap_or("legacy")` 作为 adapter_id 值
   - 这样旧运行的 fingerprint 会变化，但新运行的 fingerprint 逻辑是确定的
   - **替代方案**：若要保持旧 replay 完全兼容，可保留 `"runner_kind"` 字段但值固定为 `"legacy"`，同时新增 `"adapter_id"`。经评估，用户确认无生产数据，**采用替换策略**（旧 replay 需重新生成测试数据）

0.2 **Replay 校验策略**
   - `replay.rs` 中对比 `private["runner_kind"]` 的逻辑改为对比 `private["adapter_id"]`
   - 对于旧 snapshot（含 `runner_kind` 但不含 `adapter_id`），replay 逻辑应显式处理：若 `adapter_id` 缺失但 `runner_kind` 存在，视为 legacy 格式，允许通过（或按具体测试策略决定）
   - **结论**：旧 snapshot 仅用于测试，测试数据可重新生成，replay 逻辑直接切换为 `adapter_id` 对比，不保留 legacy 兼容分支

0.3 **Snapshot schema_version**
   - 若决定保留旧格式兼容，schema_version 保持 1
   - 若决定破坏性变更，schema_version 升级到 2，并在文档中标注
   - **结论**：保持 schema_version 为 1（测试数据可重新生成），不升级

**验证**：
- 本阶段为纯决策，产出为本文档的确认标记

---

### 阶段 1：运行时 Adapter 分发层去 Kind 化（高风险，核心路径）

**目标**：让 `runtime_adapter_for_task()` 和 cleanup 路径完全通过 `adapter_id` 工作，不再引用 `ExternalRunnerKind`。

**前置条件（步骤 0）**：更新所有测试辅助函数构造带 `runtime_binding` 的 TaskPlan

1.0 **更新测试辅助函数**（`runtime_adapter_test_support.rs` 等）
   - 修改 `external_task(task_id, kind)` 签名：移除 `kind` 参数，增加 `adapter_id: &str` 参数（或按 benchmark 名称推断）
   - 构造 `TaskPlan` 时同时设置 `runtime_binding`（使用 `built_in_protocol_registry()` 中对应 adapter 的 authority）和 `external_runner`（保留 dataset_path 等字段，不设置 kind）
   - 修改 `protocol_bound_terminal_task()` 以符合新的辅助函数签名
   - **必须在步骤 1.1 之前完成**，否则阶段 1 的测试验证无法通过

1.1 **修改 `BenchmarkRuntimeAdapter` trait**（`runtime_adapter.rs`）
   - 移除 `fn kind(&self) -> ExternalRunnerKind`
   - `RuntimeCleanupTarget` 将 `runner_kind: ExternalRunnerKind` 替换为 `adapter_id: &'static str`

1.2 **重写 `runtime_adapter_for_task()`**（`runtime_adapter.rs`）
   - 删除 `runtime_adapter_for(kind: ExternalRunnerKind)` 函数
   - `runtime_adapter_for_task()` 只接受 `TaskPlan`，逻辑简化为：
     ```rust
     pub(super) fn runtime_adapter_for_task(task: &TaskPlan) -> Result<&'static dyn BenchmarkRuntimeAdapter> {
         let binding = task.runtime_binding.as_ref()
             .context("external task missing runtime binding")?;
         built_in_protocol_registry()
             .validate_authority(&binding.authority)
             .map_err(|e| anyhow!("invalid protocol runtime binding: {e}"))?;
         runtime_adapter_for_adapter_id(binding.authority.adapter_id.as_str())
     }
     ```
   - 移除 fallback 到 `task.external_runner.kind` 的逻辑

1.3 **更新 cleanup 路径**
   - `cleanup_runtime_target()` 改为通过 `adapter_id` 查找 adapter（调用 `runtime_adapter_for_adapter_id`）
   - `runtime_cleanup_targets()` 中 `seen.contains(&adapter.kind())` 改为 `seen.contains(adapter.adapter_id())`

1.4 **更新 `runtime_snapshot_source_ref()`**（`runtime_authority.rs`）
   - 删除 `runtime_snapshot_source_ref(task, kind)` 函数
   - 统一逻辑：若 `task.runtime_binding` 存在，走 `runtime_source_ref(task)`（校验 binding 与 external_runner 一致性）；若不存在，直接读取 `task.external_runner.source_path`
   - 实际上，由于 1.0 已确保所有测试任务都设置 `runtime_binding`，生产路径永远走 protocol 校验

1.5 **更新事件日志输出**（`external.rs`, `cleanup.rs` 等）
   - 将事件日志中的 `runner_kind={:?}` 替换为 `adapter_id={}`
   - 例如 `"external_runner_preflight"` 事件的数据中将 `runner_kind` 字段替换为 `adapter_id`

1.6 **更新 terminal_bench_adapter.rs 和 swe_bench_pro_adapter.rs**
   - 移除 `kind()` 实现
   - `cleanup_target_resources()` 中如果有引用 `target.runner_kind`，改为 `target.adapter_id`

**验证**：
- `cargo test -p harnesslab-cli runner::external::runtime_adapter_tests` 通过
- `cargo test -p harnesslab-cli runner::cleanup_tests` 通过

---

### 阶段 2：Core Types 与 Snapshot 去 Kind 化（中等风险，序列化变更）

**目标**：从核心数据模型和 snapshot fingerprint 中移除 `ExternalRunnerKind` 字段。

**具体步骤**：

2.1 **修改 `ExternalRunnerSpec`**（`benchmark.rs`）
   - 移除 `kind: ExternalRunnerKind` 字段
   - 保留 `dataset_path`、`source_path`、`agent_timeout_sec`

2.2 **修改 `RuntimePreflightReport`**（`runtime.rs`）
   - 移除 `runner_kind: ExternalRunnerKind` 字段
   - 已有 `adapter_id: String` 和 `protocol_adapter_id: Option<String>`，信息不丢失

2.3 **修改 `AdapterProtocolAuthority`**（`adapter_protocol.rs`）
   - 移除 `legacy_runner_kind: Option<ExternalRunnerKind>` 字段
   - 移除 `with_legacy_runner_kind()` 方法
   - 移除 `legacy_runner_kind_authority()` 函数
   - 更新序列化/反序列化：旧数据若含 `legacy_runner_kind` 字段，serde 默认行为会忽略未知字段（需确认 `#[serde(deny_unknown_fields)]` 不存在）

2.4 **更新 snapshot fingerprint 计算**（`runtime_snapshot.rs`）
   - `runtime_fingerprint` 和 `public_fingerprint` 的 JSON 中：
     - 移除 `"runner_kind": request.runner_kind`
     - 新增 `"adapter_id": request.protocol_authority.as_ref().map(|a| a.adapter_id.as_str()).unwrap_or("legacy")`
   - 更新 `ExternalRuntimeSnapshotRequest` 结构体（如有 `runner_kind` 字段则移除）

2.5 **更新 replay 校验逻辑**（`replay.rs`）
   - 将 `private["runner_kind"]` 和 `public["runner_kind"]` 的对比改为 `private["adapter_id"]` 和 `public["adapter_id"]` 的对比
   - 移除 `runner_kind` 参数的传递

**验证**：
- `cargo test -p harnesslab-core` 通过
- `cargo test -p harnesslab-cli runner::replay` 通过
- `cargo test -p harnesslab-cli tests::external_runtime_snapshot_contract` 通过（需配合阶段 4 的集成测试更新）

---

### 阶段 3：Adapter 层去 Kind 化（中等风险，注册表变更）

**目标**：从 adapter 注册和 binding 中移除 legacy runner kind。

**具体步骤**：

3.1 **修改 `protocol_registry.rs`**
   - 从 `AdapterBindingDescriptor` 中移除 `legacy_runner_kind` 字段
   - 从 `binding()` 辅助函数中移除 `legacy_runner_kind` 参数
   - 删除 `ensure_legacy_mapping()` 函数
   - 更新 `built_in_protocol_registry()` 中三个 binding 的调用：terminal-bench 和 swe-bench-pro 不再传 `Some(ExternalRunnerKind::...)`，改为不传（即 `None`，字段删除后相当于无此参数）
   - 更新 `validate_authority()` 中 legacy_runner_kind 的一致性校验（直接移除该校验逻辑）

3.2 **修改 `registry.rs`**
   - 从 `adapter_for_with_root()` 中移除 `ExternalRunnerKind` match 分发
   - 返回的 adapter 实例不再设置 `ExternalRunnerSpec.kind`（因为字段已在阶段 2 删除）

3.3 **修改 `terminal_bench.rs` 和 `swe_bench_pro.rs`**
   - 在构造 `TaskPlan` 时不再设置 `external_runner.kind`（字段已在阶段 2 删除）
   - 保留 `external_runner` 的其他字段（`dataset_path` 等）

3.4 **同步更新 adapter 层测试**（`protocol_contract_tests.rs`, `protocol_registry.rs`）
   - `protocol_contract_tests.rs`：移除对 `legacy_runner_kind.is_some()` 的断言，改为 `is_none()` 或不断言
   - `protocol_registry.rs` 中的测试：移除 `mismatched_legacy` 测试用例，或改为测试其他 registry 校验规则

**验证**：
- `cargo test -p harnesslab-adapters` 通过
- `cargo test -p xtask adapter_claims`（如有相关测试）通过

---

### 阶段 4：测试层与集成契约全面更新（低风险，工作量大）

**目标**：更新所有测试代码和集成测试契约，移除 `ExternalRunnerKind` 引用。

**具体步骤**：

4.1 **更新 `runtime_adapter_test_support.rs`**
   - 修改 `external_task(task_id, kind)` 签名：移除 `kind` 参数（已在阶段 1 完成）
   - 构造 `ExternalRunnerSpec` 时不设置 `kind`（已在阶段 1 完成）

4.2 **更新 `runtime_adapter_tests.rs`**
   - 移除所有 `ExternalRunnerKind::TerminalBench` / `ExternalRunnerKind::SweBenchPro` 引用
   - 修改 `external_task()` 调用处（已在阶段 1 完成）
   - 修改 `assert_preflight()` 等辅助函数中的 `runner_kind` 断言为 `adapter_id` 断言
   - 修改 protocol binding 测试：不再测试 legacy kind 一致性

4.3 **更新 `cleanup_tests.rs`**
   - 移除 `ExternalRunnerKind` import
   - 构造 `ExternalRunnerSpec` 时不设置 `kind`

4.4 **更新 `runtime_anchor.rs`**
   - 修改 `external_runner()` 辅助函数，移除 `kind`

4.5 **更新 `model_tests.rs`（harnesslab-core）**
   - 修改测试中断言 `external_runner` 的逻辑

4.6 **更新 data contract tests 和 swe_bench_pro_tests**
   - 移除 `ExternalRunnerKind` 引用

4.7 **更新集成测试契约**（新增，来自审查发现）
   - `tests/external_runtime_snapshot_contract.rs`：将 `public["runner_kind"] == "terminal_bench"` 改为 `public["adapter_id"] == "harnesslab.terminal-bench.runtime"`
   - `tests/swe_runtime_snapshot_contract.rs`：将 `public["runner_kind"] == "swe_bench_pro"` 改为 `public["adapter_id"] == "harnesslab.swe-bench-pro.runtime"`
   - `tests/external_smoke_contract.rs`：将事件日志中的 `runner_kind=SweBenchPro` 改为 `adapter_id=harnesslab.swe-bench-pro.runtime`
   - `tests/support/runtime_snapshot.rs`：更新指纹重写辅助函数，移除 `runner_kind`，增加 `adapter_id`

**验证**：
- `cargo test -p harnesslab-cli` 通过
- `cargo test -p harnesslab-core` 通过
- `cargo test -p harnesslab-adapters` 通过

---

### 阶段 5：最终删除 Enum 与 Guard/Claims 更新（低风险）

**目标**：从代码库中彻底删除 `ExternalRunnerKind` enum，并同步更新所有 guard 和 claims 基础设施。

**具体步骤**：

5.1 **删除 `ExternalRunnerKind` enum**（`benchmark.rs`）
   - 删除 enum 定义
   - 删除所有未使用的 import

5.2 **更新 `no_branch_guard.rs`**（新增，来自审查发现）
   - 从 `FORBIDDEN_TOKENS` 中移除 `"ExternalRunnerKind"`、`"ExternalRunnerKind::TerminalBench"`、`"ExternalRunnerKind::SweBenchPro"` 等 token（因为 enum 已不存在，禁止引用已无意义）
   - 更新 `LEGACY_SHIM_FILES` 列表：若其中文件因本次清理被重构，需确认是否仍属于 legacy shim
   - 更新 `adapt_protocol_008_allowlist_inventory_is_review_locked` 测试中的硬编码列表

5.3 **更新 `adapter_claims.rs`**（新增，来自审查发现）
   - 更新 `active_route_spec` 中的 `file_patterns`，移除因文件删除而不存在的路径，新增因重构而产生的新文件路径

5.4 **更新 `FROZEN_SELECTOR_MANIFEST.toml`**（如需要）
   - 若任何 selector 的 `file_patterns` 因文件重构而失效，同步更新

5.5 **编译清理**
   - 运行 `cargo check` 全 workspace，修复所有编译错误

**验证**：
- `cargo check --workspace` 0 errors
- `cargo test --workspace` 全部通过
- `cargo run -p xtask -- verify-no-branch-guard` 通过
- `cargo run -p xtask -- verify-forbidden-diff` 通过

## 6. 依赖关系与执行顺序

```
阶段 0（Fingerprint/Schema 决策）
    ↓
阶段 1（Runtime Adapter 分发层）
    ├─ 步骤 1.0：测试辅助函数更新（必须在 1.1 之前）
    ├─ 步骤 1.1-1.6：trait、分发、cleanup、source_ref、日志、adapter 实现
    ↓
阶段 2（Core Types + Snapshot）
    ├─ ExternalRunnerSpec / RuntimePreflightReport / AdapterProtocolAuthority
    ├─ snapshot fingerprint 计算
    ├─ replay 校验逻辑
    ↓
阶段 3（Adapter Registry）
    ├─ protocol_registry.rs binding 和校验
    ├─ registry.rs adapter_for_with_root
    ├─ terminal_bench.rs / swe_bench_pro.rs
    ├─ protocol_contract_tests.rs / protocol_registry.rs 测试（同步更新）
    ↓
阶段 4（Tests + 集成契约）
    ├─ runtime_adapter_tests.rs
    ├─ cleanup_tests.rs / runtime_anchor.rs / model_tests.rs
    ├─ data_contract_tests / swe_bench_pro_tests
    ├─ 集成测试契约（external_runtime_snapshot_contract / swe_runtime_snapshot_contract / external_smoke_contract）
    ↓
阶段 5（Delete Enum + Guard/Claims）
    ├─ 删除 ExternalRunnerKind enum
    ├─ 更新 no_branch_guard.rs
    ├─ 更新 adapter_claims.rs
    ├─ 更新 FROZEN_SELECTOR_MANIFEST.toml
```

**关键依赖**：
- 阶段 0 必须在所有代码阶段之前：fingerprint 策略决定阶段 1 和 2 的具体实现
- 阶段 1 步骤 1.0 必须在步骤 1.1 之前：测试辅助函数不更新，阶段 1 验证无法通过
- 阶段 1 必须在阶段 2 之前：runtime adapter 的 trait 移除 `kind()` 后，core types 中的 `RuntimePreflightReport` 才能安全移除 `runner_kind`
- 阶段 2 必须在阶段 3 之前：`AdapterProtocolAuthority` 移除 `legacy_runner_kind` 后，registry 才能移除相关字段
- 阶段 3 和 4 可以部分并行：但推荐顺序执行以便逐步验证
- 阶段 5 必须在最后：enum 删除后需同步更新 guard

## 7. 验证策略

### 7.1 每阶段验证
- 每阶段完成后运行 `cargo test -p <crate>` 对应模块测试
- 每阶段完成后运行 `cargo check --workspace`

### 7.2 跨阶段验证
- 阶段 3 完成后运行 `scripts/test-after-change.sh --select ADAPT-PROTOCOL-002`（registry 验证）
- 阶段 3 完成后运行 `scripts/test-after-change.sh --select ADAPT-PROTOCOL-010`（migration preservation，需更新预期）
- 阶段 4 完成后运行 `scripts/test-after-change.sh --select ADAPT-PROTOCOL-006`（replay authority）
- 阶段 5 完成后运行 `scripts/verify-planned-adapter-selectors.sh` 完整 sweep

### 7.3 端到端验证
- `cargo test -p harnesslab-cli --lib` 全部通过
- `cargo test -p harnesslab-adapters` 全部通过
- `cargo test -p harnesslab-core` 全部通过
- `cargo run -p xtask -- verify-no-branch-guard` 通过
- `cargo run -p xtask -- verify-forbidden-diff` 通过

## 8. 风险与回滚

### 8.1 风险清单

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|---------|
| 遗漏的 `ExternalRunnerKind` 引用导致编译失败 | 高 | 中 | 阶段 5 的 `cargo check --workspace` 会暴露所有遗漏；逐步执行可缩小排查范围 |
| `ExternalRunnerSpec` 序列化格式变更导致旧测试数据无法加载 | 中 | 低 | 用户确认无生产数据；测试数据可重新生成；阶段 0 已明确不保留兼容 |
| `RuntimePreflightReport.runner_kind` 被外部消费者依赖 | 中 | 中 | workspace 内已全面清理；如存在外部消费者需在 PR 中标注 breaking change |
| 测试辅助函数签名变更导致大量测试文件需要同步修改 | 高 | 低 | 阶段 1 步骤 1.0 集中处理；阶段 4 处理剩余集成测试 |
| `runtime_adapter_for_task()` 移除 fallback 后，某些测试未设置 `runtime_binding` 导致 panic | 高 | 中 | **已在阶段 1.0 前置解决**：所有测试辅助函数强制构造 `runtime_binding` |
| snapshot fingerprint 变更导致历史运行 replay 断裂 | 中 | 中 | 阶段 0 已决策采用替换策略；测试数据可重新生成 |
| Guard/Claims 更新遗漏导致阶段 5 验证失败 | 中 | 低 | 阶段 5 明确列出所有需更新的 guard/claims 文件 |

### 8.2 回滚策略

- 每阶段作为一个独立的 commit
- 若某阶段验证失败，可单独 revert 该阶段 commit，不影响已完成阶段
- 全量验证失败时，可回滚到阶段 0（当前状态）

## 9. 验收标准

- [x] `ExternalRunnerKind` enum 从 `harnesslab-core/src/benchmark.rs` 中完全删除
- [x] `cargo check --workspace` 0 errors
- [x] `cargo test --workspace` 全部通过
- [x] `scripts/verify-planned-adapter-selectors.sh` 完整 sweep 通过（active=28 planned=1）
- [x] `cargo run -p xtask -- verify-no-branch-guard` 通过
- [x] `cargo run -p xtask -- verify-forbidden-diff` 通过
- [x] terminal-bench 和 swe-bench-pro 的端到端测试全部通过
- [x] 代码审查通过（实现后 closure 审查完成）

## 10. 开放问题

1. **ExternalRunnerSpec 的最终命运**：是否完全由 `TaskRuntimeBinding` 替代？当前 `runtime_dataset_ref()` 在 protocol binding 存在时仍检查 `external_runner.dataset_path` 的一致性。若完全移除 `external_runner`，则需要将 `dataset_path`、`source_path`、`agent_timeout_sec` 迁移到 `TaskRuntimeBinding` 或新的结构中。**标记为 ADAPT-DATA-000 后续工作，本次清理不涉及。**
2. **事件名称中的 `external_runner` 前缀**：如 `external_runner_started`、`external_runner_setup_failed` 等事件名称包含 `external_runner` 字样。这些名称是否需要重命名为 `adapter_started` 等？**结论：本次清理不触及事件名称，避免扩大范围。**
3. **旧 snapshot 的完全淘汰时间**：虽然用户确认无生产数据，但旧测试数据（如 CI 缓存中的历史运行目录）可能仍存在。需在实施前确认测试环境可接受重新生成数据。

## 11. 实施工作量预估（修订后）

| 阶段 | 预估改动文件数 | 预估工作量 |
|------|--------------|---------|
| 阶段 0 | 0（纯决策） | 0.5 小时 |
| 阶段 1 | 8-12 | 4-6 小时 |
| 阶段 2 | 6-8 | 2-3 小时 |
| 阶段 3 | 6-8 | 2-3 小时 |
| 阶段 4 | 25+ | 6-8 小时 |
| 阶段 5 | 5-8 | 2-3 小时 |
| **总计** | **50-65** | **16-24 小时** |

> 注：工作量较初稿显著增加，主要因审查发现 fingerprint 兼容性、集成测试契约、guard/claims 同步等额外工作。

## 12. 与当前架构的对比

| 维度 | 当前（有 Legacy） | 目标（无 Legacy） |
|------|-----------------|-----------------|
| Adapter 分发 | `match kind` + fallback | `adapter_id` lookup only |
| Core 类型 | `ExternalRunnerKind` enum | enum 删除 |
| Registry binding | `legacy_runner_kind: Some(...)` | 字段删除 |
| Runtime adapter trait | `kind()` required | `kind()` removed |
| 测试辅助函数 | `external_task(task_id, kind)` | `external_task(task_id)` + `runtime_binding` |
| Snapshot fingerprint | 含 `"runner_kind"` | 含 `"adapter_id"` |
| Replay 校验 | 对比 `"runner_kind"` | 对比 `"adapter_id"` |
| 新增 adapter 成本 | 需理解 legacy 概念 | 无需了解 legacy 概念 |

## 13. 审查历史

- **Round 1**: 2026-06-12，对抗性审查发现 6 项 blocking findings 和 3 项非阻塞风险，全部接受并纳入本修订版计划。
- **Round 2**: 2026-06-12，实现后 closure 审查，发现 1 项 pre-task cleanup stderr 可见性缺陷（计划外），已修复并通过全 workspace 验证。其余验收项均满足。
- **Review Report**: [vs_review/2026-06-12-remove-external-runner-kind-plan-review.md](../../vs_review/2026-06-12-remove-external-runner-kind-plan-review.md)
