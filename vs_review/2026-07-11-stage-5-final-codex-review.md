# Subagent VS Review: v1.0.5 Stage 5 最终 Codex 审查

- Created: 2026-07-11T18:20:00+08:00
- Updated: 2026-07-11T18:31:00+08:00
- Report schema: adversarial-v1
- Task: 验证 v1.0.5 Stage 5 发布前硬化是否可以完成。
- Report path: `vs_review/2026-07-11-stage-5-final-codex-review.md`
- Review mode: fresh internal subagents
- Source session policy: 不继承主代理上下文；审查员只读取目标文件。
- Status: open

## Round 1: 启动器与发布证据完整性

### Review Input

#### Objective
对当前 `f9d79f8` 的 Stage 5 实现进行只读对抗性审查，发现会阻止发布准备的实现或证据缺口。

#### Review Target
跨平台启动器、取消清理、真实 Harbor 条件回归、CI 证据和 Stage 5 工程计划。

#### Target Locations
- `run_dev.sh`
- `scripts/test-run-dev-api.sh`
- `lib/dev.js`
- `tests/python/test_harbor_real_smoke.py`
- `tests/python/test_real_harbor_cancel_recovery.py`
- `docs/releases/v1.0.5/engineering-plan.md`

#### Change Introduction
Stage 5 已补充 API 模式启动、跨平台 CI 和真实 Harbor 条件测试，主张可以进入最终发布准备。

#### Risk Focus
- 包装进程退出后遗留 Uvicorn 或 Vite 子进程。
- 文档声称的真实 Harbor 跳过条件与测试实际条件不一致。
- Windows 进程树关闭失败时缺少可观测性。

#### User-Perspective Review Focus
本地开发者按 Ctrl-C 后，端口必须释放，不能留下不可见的服务进程。

#### Implementation Completeness Focus
S5-02、S5-03、S5-05 和 S5-07 的生产路径、测试路径与工程计划必须相互一致。

#### Target Benefit Focus
验证“本地启动可可靠收敛”和“发布证据可复现”的主张，不将 mock 或手工说明当作实现。

#### Assumptions To Attack
- 终止直接父 PID 等同于终止整棵服务进程树。
- 真实 Harbor 回归缺少凭证时会自动 skip。
- 手动 CI 证据可以替代被注释的自动触发。

#### Adversarial Lenses
- implementation-completeness
- failure
- release-ops
- testing
- observability

#### Verification Status
审查前的最终跨平台 CI `#29147812986` 为绿色；真实 Harbor Docker 回归为 opt-in。

#### Reviewer Instructions
- Fresh internal subagent session.
- 不修改文件。
- 读取目标文件直接给出 blocker、warning、verdict 和路径证据。

### Internal Subagent Unavailable Fallback

- Required only when fresh internal subagents are unavailable.
- Internal subagent unavailable reason: n/a
- Fallback outcome: n/a

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 120s | 120s | 2 | 未获得结论不得通过 |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| `code-reviewer` | 独立代码审查，优先攻击生产路径与验证证据是否真正闭环 | 启动器清理、条件回归和跨平台发布 |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| `code-reviewer` | `multi_agent_v1__spawn_agent` | `019f50ac-66d4-7110-b87e-7a3e0f3734f3` | Codex subagent transcript | `fork_context=false` | Round 1 Review Input | 主代理历史、结论和未提交 diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| R1 | `code-reviewer` | 1 | `019f50ac-66d4-7110-b87e-7a3e0f3734f3` | 180s | completed_after_extension | 初始等待超时后要求返回最终结论 | completed |

### Reviewer Outputs

#### R1

##### Summary
发现两个 HIGH 阻断项：shell 启动器只终止包装进程，未验证后代服务进程和端口释放；工程计划错误声称真实 Harbor 测试在缺失凭证时跳过。结论为 `BLOCKED`。

##### Blocking Findings
- B1: `run_dev.sh` 只对直接 PID 调用 `kill`，而 `uv run`、`npm run dev` 可能派生 Uvicorn/Vite 后代。`scripts/test-run-dev-api.sh` 仅确认启动脚本退出，没有确认两个服务端口释放。
  - Broken assumption: 父进程退出即服务树退出。
  - Failure scenario: Ctrl-C 后端口仍被遗留服务占用。
  - Impact: 再次启动失败、开发环境状态不可预期。
  - Evidence: `run_dev.sh:54`, `run_dev.sh:58`, `scripts/test-run-dev-api.sh:28`, `scripts/test-run-dev-api.sh:68`（审查时版本）。
- B2: S5-05 声称“缺失凭证时明确 skip”，但两个真实 Harbor 测试只检查 `ORNNLAB_REAL_HARBOR=1` 和 Docker 可用性。
  - Broken assumption: 测试具有凭证 skip 前置条件。
  - Failure scenario: 文档承诺与实际测试条件不一致。
  - Impact: 发布验收证据无效。
  - Evidence: `docs/releases/v1.0.5/engineering-plan.md:62`, `tests/python/test_harbor_real_smoke.py:12`, `tests/python/test_real_harbor_cancel_recovery.py:15`。

##### Non-blocking Risks
- W1: `.github/workflows/ci.yml` 仅保留 `workflow_dispatch`。这是用户明确要求关闭自动 CI 的既有策略，记录为接受风险，不在本轮擅自改回。
- W2: `lib/dev.js` 的 Windows `taskkill` 返回状态未记录。已作为可靠性观察问题修复。

##### User-Perspective Checks
- Usability: B1。开发者无法确认 Ctrl-C 是否真正停止所有服务。
- Ease of use: B1。遗留端口会让下一次启动出现难以诊断的占用错误。
- Ease of understanding: B2。工程计划描述的 skip 条件不真实。

##### Implementation Completeness Checks

| Plan Item | Expected Behavior | Production Code Path | Integration Entry | Test Evidence | Runtime / Log Evidence | Mock / Stub Exposure | Status | Finding Link |
|---|---|---|---|---|---|---|---|---|
| S5-02 | 退出时收敛服务树 | `run_dev.sh` | Ctrl-C / EXIT | 原测试未验证端口释放 | 缺失 | none | partial | B1 |
| S5-03 | 全栈退出后无残留端口 | `scripts/test-run-dev-api.sh` | 发布门禁 | 原测试仅等待启动器 PID | 缺失 | none | partial | B1 |
| S5-05 | 条件回归描述准确 | 真实 Harbor pytest | `ORNNLAB_REAL_HARBOR=1` | skip 条件不含凭证 | 文档不一致 | none | partial | B2 |

##### Target Benefit Checks

| Claimed Benefit | Baseline | Target | Measurement Method | Comparison Evidence | Result | Regression / Side Effect | Status | Finding Link |
|---|---|---|---|---|---|---|---|---|
| 本地服务可可靠停止 | 仅脚本 PID 退出 | 服务端口释放 | 退出后健康检查与端口 bind | 原证据不足 | unmeasured | 遗留监听端口 | weak-evidence | B1 |

##### Required Fixes
- B1: 递归终止服务后代；测试验证健康端点不可达且两个端口可重新 bind。
- B2: 以实际条件修正文档，或实现真实凭证 skip；不得保留不实表述。

##### Missing Tests
- B1: run_dev 退出后的端口释放测试。

##### Missing Logs / Observability
- W2: Windows `taskkill` 失败需留下可诊断日志。

##### Evidence
- `run_dev.sh` - 审查时只终止 `FRONTEND_PID` 和 `BACKEND_PID`。
- `scripts/test-run-dev-api.sh` - 审查时未验证端口释放。
- `tests/python/test_harbor_real_smoke.py` - 条件仅包含 opt-in 与 Docker。

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| R1 | B1 | 包装进程退出不保证 Uvicorn/Vite 后代退出 | blocking | accept | 代码路径和测试均无法反证残留端口 | `run_dev.sh` 新增递归 `pgrep -P` 树终止与等待；`test-run-dev-api.sh` 新增健康端点不可达和可重新 bind 验证 | Round 2 独立复审 |
| R1 | B2 | 文档错误宣称凭证缺失会 skip | blocking | accept | 公共基准回归没有认证前置，真实测试也不检查凭证 | S5-05 改为准确的 opt-in + Docker skip 语义 | Round 2 独立复审 |
| R1 | W1 | 自动 CI 已关闭 | non-blocking | accept-risk | `.github/workflows/ci.yml` 注释明确写有用户关闭自动 CI 的决策 | 保持手动 `workflow_dispatch`，不逆转用户决定 | 发布时继续手动触发 |
| R1 | W2 | Windows taskkill 失败不可观察 | non-blocking | accept | 失败时可能无法定位服务树问题 | `lib/dev.js` 记录失败原因并回退 `SIGTERM` | Windows CI 复验 |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 2 pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Implementation completeness gaps resolved or accepted by user: yes, pending independent verification
- Target benefit warnings recorded: yes
- Blocked reason: 等待修复后的独立 Round 2 复审。
- Allowed to proceed: no

## Final Conclusion

Round 1 不允许完成 Stage 5。阻断项已修复并通过本地全量门禁，但必须由新的独立 Codex 审查员复审后才能关闭。
