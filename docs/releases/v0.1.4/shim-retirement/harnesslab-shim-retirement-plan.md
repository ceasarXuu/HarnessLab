# Engineering Plan: HarnessLab 兼容层与 Rust Legacy 立即退役实施计划

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set (planned `2026.06.22`); `ornnlab` npm `0.1.4` (planned); Python app `0.2.0` | 2026-06-22 | Initial Standard Plan derived from `harnesslab-shim-retirement-prd.md`. Five phases: Discovery → Python Shim Outer → Services/Settings/Tests → Rust Workspace → Verification & Close-out. Plan-to-code completeness evidence bound to every phase exit. |
| 1.1 | OrnnLab Build Set (planned `2026.06.22`); `ornnlab` npm `0.1.4` (planned); Python app `0.2.0` | 2026-06-22 | OQ-1 resolved as option A: `integrations/terminal_bench/harnesslab_tb_*.py` will be renamed to `ornnlab_tb_*.py` inside this plan. Added Phase 2.5 (terminal_bench module rename) between Phase 2 and Phase 3. Updated AC1 exemption list (terminal_bench no longer exempt). Plan moved out of Draft. |
| 1.2 | OrnnLab Build Set (planned `2026.06.22`); `ornnlab` npm `0.1.4` (planned); Python app `0.2.0` | 2026-06-22 | Phase 0 Discovery completed. Key findings: (a) `HarnessLabCommandAgent` is a real external agent contract; user approved full rename (agent name `harnesslab-command` → `ornnlab-command`, env vars `HARNESSLAB_AGENT_*` → `ORNNLAB_AGENT_*`, all 5 `verify-terminal-bench-*.sh` smoke scripts updated to the new strings rather than deleted). (b) `.github/workflows/ci.yml:72` `npm run smoke:harnesslab-transition` stays (npm Out-of-scope). (c) `test_harbor_config.py` line 23/28/43 `HARNESSLAB_TEST_ENV` is a generic env-passthrough fixture (not shim), keep but rename to `ORNNLAB_TEST_ENV` for brand consistency. (d) `test_settings_migration.py` is fully shim-only → delete file. (e) Phase 3 Rust-cleanup scope corrected: `verify-terminal-bench-*.sh` scripts are NOT Rust-related, do not delete in Phase 3. |
| 1.3 | OrnnLab Build Set `2026.06.22`; `ornnlab` npm `0.1.4` (planned); Python app `0.2.0` | 2026-06-22 | **Completed**: Phase 1 (commit 3faacfe), 2A (9832a65), 2B+2C (9da8588), 2.5 (215a61c), 3 (699e23e), and 4 (this commit). All 10 AC validated. AC1 grep returns 0 hits in active surface (excluding designed-in retention: npm transition package, GitHub repo URL, v0.1.3 release ledger, shim-retirement work-item docs, verify-ornnlab-rebrand.py shim-detection strings, README/playbook descriptions of the historical brand). AC2 grep returns 0 hits in active surface. Pytest 83 passed (51 web + 32 terminal_bench). Frontend typecheck/lint/test all pass. Verify scripts both exit 0. Rust workspace + 11 Rust verify scripts + tools.versions.toml + TEST_REGISTRY.toml + FROZEN_SELECTOR_MANIFEST.toml + verify-test-after-change-select-output.sh all removed in Phase 3 + Phase 4 cleanup. rust-legacy-fate.md status flipped to Retired in commit 699e23e. |

## Metadata

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.0
- Status: Completed (all 4 phases implemented and validated, awaiting adversarial review)
- Owner / Responsible: User (项目所有者) + AI agent
- Related Systems: OrnnLab Python (`ornnlab/`), Rust legacy workspace (`crates/`, `xtask/`), terminal_bench Python integration (`integrations/terminal_bench/`), verify scripts (`scripts/verify-*.py`).
- Related Links:
  - PRD: `./harnesslab-shim-retirement-prd.md`
  - v0.1.4 index: `./v0.1.4-docs.md`
  - Rust legacy decision (to be flipped to "Retired"): `../../archive/stubs/rust-legacy-fate.md`
  - Governance: `../v0.1.3/version-governance.md`
- Risk Level: Medium
- Plan Type: Standard

## 1. Background

OrnnLab 在 HarnessLab → OrnnLab 品牌迁移期保留了完整的"兼容 shim"，并保留
Rust legacy workspace 作为 reference。`harnesslab-shim-retirement-prd.md`
确认项目未发布、无真实旧用户、无需迁移窗口，因此本计划对所有 shim 与
Rust 残留执行立即退役。

## 2. Problem Definition

- **Current behavior**：仓库同时维护两套品牌路径的 env / SQLite / Docker label / Backup / API / Python 入口；Rust workspace 仍占用 clone/索引/搜索表面。
- **Expected behavior**：仅 OrnnLab 一套；HarnessLab 仅在 `docs/archive/`、`docs/plans/`、`vs_review/`、`coe/`、`npm/harnesslab-transition/`、`bin/harnesslab.js`、本计划与 PRD 自身（审计证据）允许出现；Rust workspace 从工作树删除。
- **Gap**：执行本计划。
- **Affected surfaces**：`harnesslab/` Python 包、`ornnlab/settings.py`、`ornnlab/services/{doctor,docker_orphan,backup,harbor_subprocess,harbor_engine}_service.py`、`tests/python/`、`scripts/verify-ornnlab-rebrand.py`、Rust workspace 全部、`pyproject.toml`、`README.md`、`rust-legacy-fate.md`。

## 3. Goals

复用 PRD 第 2 节目标。本计划负责将 PRD AC1–AC10 实施并验证通过。

## 4. Non-Goals

- npm 端（`npm/harnesslab-transition/`、`bin/harnesslab.js`、`package.json` 中 harnesslab 字段、`lib/source.js`）保持现状。
- v0.1.3 release ledger、`docs/archive/**`、`docs/plans/**`、`vs_review/**`、`coe/**` 不回改。
- 现役 OrnnLab 业务逻辑不重构（仅删除 shim 分支必需的最小修改）。

## 5. Constraints And Assumptions

| Assumption | Verification Method | If Assumption Fails |
|---|---|---|
| 本地/CI/Docker 主机无需保留 HarnessLab 残留数据 | 用户已确认 review point 2 | 退役前补迁移脚本 |
| `coverage-critical.toml` 仅服务 Rust 覆盖率 | Discovery Phase 0 已确认 (modules 全指向 `crates/harnesslab-*/src`) | n/a |
| `git rm` 直删被用户授权 (PRD 选项 B) | PRD 选项 B 文本 | n/a |
| Rust workspace 在 CI 上无活跃 job 依赖 | Phase 0 grep `.github/workflows/` | 若有 → 同步删 |

约束：

- 用户规则"禁止不可恢复删除"被用户在 PRD 选项 B 明确豁免。
- 用户规则"未经允许不得创建新分支"——本计划继续在 `main` 上推进。
- 用户规则"最小化提交"——每 phase 至少一组 commit + push。

## 6. Current State

### 6.1 Shim 清单

参见 PRD §4.A–E。本计划补充 Discovery 发现：

- `tests/python/` 中 9 个文件涉及 HARNESSLAB_*：`test_settings_migration`、`test_docker_orphan_service`、`test_backup_service`、`test_harbor_config`、`test_harbor_subprocess`、`test_system_api`、`test_real_harbor_cancel_recovery`、`test_harbor_real_smoke`、`test_cli`。
- `README.md` 命中 harnesslab/HARNESSLAB 字符串，逐处审查。

### 6.2 Rust 退役清单

- 顶层文件：`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`coverage-critical.toml` (Discovery 已确认 Rust-only)。
- 目录：`crates/`、`xtask/`。
- Verify/CI：`scripts/verify-terminal-bench-*.sh`、`scripts/scan-artifacts-for-secrets.sh` 中服务 Rust 制品的部分；`.github/workflows/` 中 Rust job (Phase 0 待确认)。
- 文档：`docs/archive/stubs/rust-legacy-fate.md` 状态翻转。

### 6.3 Out-of-scope 但需感知

- `integrations/terminal_bench/harnesslab_tb_*.py`（6 个文件）：**Discovery 已观察**到这是产品代码（非 shim），但命名仍带 `harnesslab`。处置待 OQ-1。
- `README.md`、`package.json`、`lib/source.js` 命中：按 PRD §4.G "文档" 与 npm Out-of-scope 准则区分处理。
- `npm/harnesslab-transition/`、`bin/harnesslab.js`：PRD 明确 Out-of-scope。

## 7. Plan Summary

5 阶段串行：

1. **Phase 0 — Discovery**：解决 OQ-1，扫描 CI 中的 Rust 依赖，固化最终删除清单。
2. **Phase 1 — Python Shim 上层退役**：删 `harnesslab/` 包、`pyproject.toml` console script、API 字段双发、上层相关测试。
3. **Phase 2 — Services/Settings/Tests 深度退役**：删 settings/services 中 LEGACY_* / HARNESSLAB_* 分支与对应测试 case。
4. **Phase 3 — Rust Workspace 退役**：删 `crates/`、`xtask/`、`Cargo*.toml`、`rust-toolchain.toml`、`coverage-critical.toml`、Rust verify 脚本、CI Rust 任务；翻转 `rust-legacy-fate.md`。
5. **Phase 4 — Verification & Close-out**：跑全量 AC1–AC10；闭环 fix-plan Open Decision；调用 `subagent-vs-review`。

## 8. Overall Technical Design

- **删除即设计**：每次删除前 `rg` 列出所有引用方，确保无 dangling reference。
- **保护性测试**：每个 phase 结束前必须 `uv run pytest tests/python` + frontend 4 件套全过。
- **审计驱动**：每个 phase 独立 commit message 记录"删了什么、为什么、影响哪里"。
- **测试同步退役**：不允许保留断言 legacy 行为的死测试。
- **设计豁免位**：`scripts/verify-ornnlab-rebrand.py`、PRD 与本计划自身可保留 `harnesslab` 字符串（PRD AC1 豁免清单）。

## 9. Dependencies

| Dependency | Type | Current Status | Blocking Risk | Handling Plan |
|---|---|---|---|---|
| `integrations/terminal_bench/` 处置决策 (OQ-1) | decision | Unknown | High | Phase 0 由用户拍板 |
| `.github/workflows/` Rust 任务 | system | Unknown | Medium | Phase 0 grep；如有 → Phase 3 一并删 |
| `npm/harnesslab-transition/`、`bin/harnesslab.js` | system | Ready (保留) | Low | 计入 AC1 豁免清单 |
| Frontend 已 strip `harnessLab` | code | Ready | Low | v0.1.4 fix plan Phase 3 已完成 |

## 10. Phased Execution Plan

### Phase 0: Discovery

#### Objective
解决 OQ-1，输出最终删除清单。

#### Entry Criteria
- PRD `Ready for implementation`，用户已签字 5 个 review point。

#### Entry Criteria Checks
| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| PRD ready | grep PRD Status 字段 | "Ready for implementation" 命中 | Agent |
| 用户签字 | 本对话历史 | "1 ok 2 ok 3 ok 4 ok 5 仍然在 prd 中" | User |

#### Design Approach
纯只读探索：读 `integrations/terminal_bench/` 文件头、grep `.github/workflows/` 与 `tests/python/`。

#### Implementation Tasks
1. 读 `harnesslab_tb_process.py` 与 `harnesslab_tb_agent.py` 前 40 行判定性质。
2. `rg -l "harnesslab|HARNESSLAB" .github/`。
3. `rg -n "HARNESSLAB|harnesslab" tests/python/` 列出 case 行号。
4. 把发现写入 Phase 0 结论小节。

#### Deliverables
- 本计划 Phase 0 结论小节填充。
- OQ-1 关闭或升级为产品决策。

#### Implementation Completeness Evidence
| Plan Item | Production Code Path | Integration Entry | Test Evidence | Runtime / Log Evidence | Mock / Stub Exposure | Status |
|---|---|---|---|---|---|---|
| `integrations/terminal_bench/` 性质判定 | n/a (Discovery) | Phase 0 结论 | 无新代码 | 文件头摘录 | none | planned |
| `.github/workflows/` Rust 任务清单 | n/a | Phase 0 结论 | 无 | grep 输出 | none | planned |
| `tests/python/` shim case 分布表 | n/a | Phase 0 结论 | 无 | grep 输出 | none | planned |

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| Discovery 输出完整 | 人工审查 Phase 0 结论 | OQ-1 有明确决策；CI Rust 清单覆盖所有 workflow 文件 |

#### Exit Criteria
- OQ-1 已决策。
- CI Rust 任务清单固化。
- 测试退役矩阵填好。

#### Review Plan
用户对 OQ-1 拍板。

#### Risks And Fallback
| Risk | Impact | Trigger | Mitigation | Fallback |
|---|---|---|---|---|
| OQ-1 推迟 → AC1 grep 不能 0 命中 | Medium | 用户回 "另起" | 将 `integrations/terminal_bench/` 加入 AC1 豁免 | 推迟 OQ-1 不阻塞，AC 调整 |
| CI 隐藏 Rust 任务 | Low | grep 结果非空 | 并入 Phase 3 | n/a |

#### Gate To Next Phase
OQ-1 决议 + CI Rust 任务清单合并入 Phase 3 任务列表。

---

### Phase 1: Python Shim 上层退役

#### Objective
删除最外层 shim（用户可见 import / console script / API 字段），不触碰 settings/services 内部。

#### Entry Criteria
- Phase 0 完成。

#### Entry Criteria Checks
| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Phase 0 deliverables 完整 | Phase 0 结论非空 | 文档版本升 1.1 | Agent |

#### Design Approach
- `git rm -rf harnesslab/`
- 删 `pyproject.toml` 中 `harnesslab = "ornnlab.cli:main"`。
- 删 doctor / orphan API 中 `harnesslab_orphans` 字段双发，保留 `ornnlab_orphans`。
- 删测试中"只测 harnesslab 包入口/console script"的 case。

#### Implementation Tasks
1. `git rm -rf harnesslab/`。
2. 编辑 `pyproject.toml` `[project.scripts]`。
3. 编辑 `ornnlab/services/doctor_service.py`、`docker_orphan_service.py` 删除字段写入。
4. 调整 `tests/python/test_system_api.py`、`test_cli.py` 受影响断言。
5. `uv run pytest tests/python` 确认全过。
6. Commit + push。

#### Deliverables
- 工作树中无 `harnesslab/` 目录。
- `pyproject.toml` 中无 harnesslab。
- API 响应无 `harnesslab_orphans`。
- 测试全过。

#### Implementation Completeness Evidence
| Plan Item | Production Code Path | Integration Entry | Test Evidence | Runtime / Log Evidence | Mock / Stub Exposure | Status |
|---|---|---|---|---|---|---|
| 删 `harnesslab/` 包 | `harnesslab/__init__.py`, `__main__.py`, `cli.py` | n/a | `pytest -k cli` 通过 | `python -m harnesslab` → ModuleNotFoundError | none | planned |
| 删 console script | `pyproject.toml` `[project.scripts]` | shell `harnesslab` | n/a | `which harnesslab` 未找到 | none | planned |
| 删 `harnesslab_orphans` 字段 | `doctor_service.py`, `docker_orphan_service.py` | API `/system/doctor` | `test_system_api.py` 断言无该字段 | response 无该字段 | none | planned |

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| Python 测试 | `uv run pytest tests/python` | exit 0 |
| Import smoke | `uv run python -c "import harnesslab"` | ModuleNotFoundError |
| Console smoke | `uv run harnesslab --version` | command not found |

#### Exit Criteria
- 测试全过 + commit + push。

#### Review Plan
代理自审 + 用户 Phase 4 一次性 review。

#### Risks And Fallback
| Risk | Impact | Trigger | Mitigation | Fallback |
|---|---|---|---|---|
| 前端调用 `harnesslab_orphans` | Medium | 前端 typecheck 失败 | Phase 1 提交前跑前端 4 件套 | `git revert` |
| 第三方脚本依赖 console script | Low | 用户报错 | git 历史恢复 | n/a |

#### Gate To Next Phase
验证全过 + push 完成。

---

### Phase 2: Services / Settings / Tests 深度退役

#### Objective
删除所有 `LEGACY_*`、`HARNESSLAB_*` 环境变量、`~/.harnesslab` 路径、`harnesslab.sqlite` 迁移、`harnesslab.run_id` Docker 标签、`harnesslab-backup-manifest.json` Backup 识别；同步调整/删测试。

#### Entry Criteria
- Phase 1 完成。

#### Entry Criteria Checks
| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Phase 1 完成 | `git log` 含 Phase 1 commit | hash | Agent |

#### Design Approach
按 PRD §4.B/C/D 逐文件删除。同步处理对应测试：整文件只为 shim → 整删；混合文件 → 只删 legacy case。

#### Implementation Tasks
按文件分子任务：

1. `ornnlab/settings.py` — 删 LEGACY_HOME / HARNESSLAB_HOME / harnesslab.sqlite migration。
2. `ornnlab/services/doctor_service.py` — 删 RUNTIME_ENV_PAIRS 双发，doctor 静默化。
3. `ornnlab/services/harbor_subprocess.py` + `harbor_engine.py` — 删 HARNESSLAB_HARBOR_* 回退。
4. `ornnlab/services/docker_orphan_service.py` — 删 LEGACY_RUN_LABEL / scan_harnesslab_containers。
5. `ornnlab/services/backup_service.py` — 删 LEGACY_MANIFEST_NAME / 旧 wal/shm 识别。
6. `tests/python/` 同步退役 (9 个文件)。
7. 每 1–2 个文件一次 commit + 跑测试。

#### Deliverables
- 5 services + settings.py 中无 HARNESSLAB / legacy。
- 测试套件相应调整完成，全部通过。

#### Implementation Completeness Evidence
| Plan Item | Production Code Path | Integration Entry | Test Evidence | Runtime / Log Evidence | Mock / Stub Exposure | Status |
|---|---|---|---|---|---|---|
| Settings 退役 | `ornnlab/settings.py` | OrnnLab 启动初始化 | `test_settings_migration.py` 调整或删 | `~/.harnesslab` 存在时也不读取 | none | planned |
| Doctor 静默化 | `doctor_service.py` | API `/system/doctor` | 新增反向断言 | response 无 legacy 字段 | none | planned |
| Harbor env 退役 | `harbor_subprocess.py`, `harbor_engine.py` | Harbor 子进程启动 | `test_harbor_config.py` legacy case 删 | `HARNESSLAB_HARBOR_*` 不生效 | none | planned |
| Docker orphan label 退役 | `docker_orphan_service.py` | 容器扫描 | `test_docker_orphan_service.py` 调整 | `harnesslab.run_id` 标签忽略 | none | planned |
| Backup 退役 | `backup_service.py` | 备份恢复流程 | `test_backup_service.py` 调整 | 旧 manifest 不识别 | none | planned |

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| Python 单测 | `uv run pytest tests/python` | exit 0 |
| Doctor 静默 | 设 `HARNESSLAB_HARBOR_ENGINE=fake` 调用 doctor | 输出无相关字段、无 warning |
| Orphan 静默 | `docker run -d --label harnesslab.run_id=x busybox sleep 60` 后扫描 | 不识别为 OrnnLab orphan |

#### Exit Criteria
- 上述全过。
- `rg "HARNESSLAB|harnesslab" ornnlab/` → 0 hits。

#### Review Plan
用户 Phase 4 一次性 review。

#### Risks And Fallback
| Risk | Impact | Trigger | Mitigation | Fallback |
|---|---|---|---|---|
| settings 中 LEGACY_HOME 与 init 耦合 | Medium | pytest 红 | 重构 init 路径而非"补 shim 回去" | `git revert` |
| 测试断言总数下降影响 verify 计数 | Low | rebrand verify 报 missing | 同步删 `_check_migration_tests` | n/a |

#### Gate To Next Phase
`ornnlab/` 内无 harnesslab + 测试全过 + push。

---

### Phase 3: Rust Workspace 退役

#### Objective
删除 Rust legacy 全部、Rust verify/CI、翻转 `rust-legacy-fate.md`。

#### Entry Criteria
- Phase 2 完成。

#### Entry Criteria Checks
| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Phase 2 完成 | `rg "HARNESSLAB|harnesslab" ornnlab/` | 0 hits | Agent |

#### Design Approach
- `git rm -rf crates/ xtask/`
- `git rm Cargo.toml Cargo.lock rust-toolchain.toml coverage-critical.toml`
- 按 Phase 0 输出删 `scripts/verify-terminal-bench-*.sh` / `scan-artifacts-for-secrets.sh` 中 Rust-only。
- 按 Phase 0 输出删 `.github/workflows/` 中 Rust 任务。
- 编辑 `docs/archive/stubs/rust-legacy-fate.md` 翻转为 "Retired in commit <sha> on 2026-06-22"。

#### Implementation Tasks
1. `git rm` 上述路径。
2. 同步 `scripts/verify-ornnlab-rebrand.py` 如有 Rust 巡检函数一并删。
3. 编辑 `rust-legacy-fate.md`。
4. Commit + push。

#### Deliverables
- 工作树中无 Rust 残留。
- `rust-legacy-fate.md` 状态翻转。

#### Implementation Completeness Evidence
| Plan Item | Production Code Path | Integration Entry | Test Evidence | Runtime / Log Evidence | Mock / Stub Exposure | Status |
|---|---|---|---|---|---|---|
| Rust 源码删除 | `crates/`, `xtask/` | n/a | n/a | `ls crates` → No such | none | planned |
| Rust 元数据删除 | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | n/a | n/a | `cargo check` 在仓库根 manifest 未找到 | none | planned |
| coverage 配置删除 | `coverage-critical.toml` | n/a | n/a | 文件不存在 | none | planned |
| `rust-legacy-fate.md` 翻转 | docs | n/a | grep 状态字段 | "Retired" 字样 | none | planned |
| CI Rust 任务清理 | `.github/workflows/*.yml` | CI 触发 | 下次 CI 跑 | 工作流无 Rust step | none | planned |

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| 工作树检查 | `ls crates xtask Cargo.toml 2>&1` | 全报 No such |
| AC2 grep | `rg "cargo\|crates/\|xtask\|Cargo\.toml" -g '!docs/archive/**' -g '!docs/plans/**' -g '!.git/**' -g '!docs/releases/v0.1.4/harnesslab-shim-retirement-*.md'` | 0 hits |
| Python 单测 | `uv run pytest tests/python` | exit 0 |

#### Exit Criteria
- 上述全过 + `git status` clean。

#### Review Plan
用户 Phase 4 一次性 review。

#### Risks And Fallback
| Risk | Impact | Trigger | Mitigation | Fallback |
|---|---|---|---|---|
| Python 意外 import 自 crates native module | Low | pytest 红 | Phase 0 已 grep 排除 | `git revert` |
| `.github/workflows/` 删 Rust job 影响其他 job | Low | 下次 PR | Phase 0 仅删 Rust-only step | `git revert` |

#### Gate To Next Phase
工作树/AC2 grep 全过 + push。

---

### Phase 4: Verification & Close-out

#### Objective
跑全部 AC1–AC10；闭环文档；用 `subagent-vs-review` 对抗审查；登记退役 commit。

#### Entry Criteria
- Phase 1–3 完成。

#### Entry Criteria Checks
| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| 3 个实施 phase 全 push | `git log -n 6 --oneline` | commit hash | Agent |

#### Design Approach
按 AC1–AC10 顺序验证；任一红灯 → 回 phase 修补；全绿后闭环文档 + push。

#### Implementation Tasks
1. `uv run python scripts/verify-version-governance.py`。
2. `uv run python scripts/verify-ornnlab-rebrand.py`。
3. AC1 / AC2 grep 命令。
4. `uv run pytest tests/python`。
5. `npm --prefix frontend run typecheck && lint && test && build`。
6. 关闭 `harbor-rebrand-residue-fix-plan.md` Open Decision 中 "HarnessLab 兼容 shim 退役时间表"。
7. 更新本计划状态为 `Completed`，Document Control 升 1.1。
8. 更新 `v0.1.4-docs.md` work item 状态。
9. 调用 `subagent-vs-review` skill。
10. 处置 review 发现 → 二次提交。

#### Deliverables
- 10 个 AC PASS 凭据。
- 闭环文档 commit。
- `subagent-vs-review` 报告 + 回应。

#### Implementation Completeness Evidence
| Plan Item | Production Code Path | Integration Entry | Test Evidence | Runtime / Log Evidence | Mock / Stub Exposure | Status |
|---|---|---|---|---|---|---|
| AC1–AC10 全过 | n/a | n/a | 命令输出粘贴到 Phase 4 结论 | exit 0 | none | planned |
| fix-plan Open Decision 关闭 | docs | Open Decisions 段 | grep 改后字样 | "已退役 → 参见" | none | planned |
| 本计划状态 Completed | docs | 本文件 Status 字段 | grep | "Completed" | none | planned |
| subagent-vs-review 通过 | n/a | `/vs_review/` 新文件 | 文件存在 | review report | none | planned |

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| PRD AC1 | grep harnesslab (按 PRD §10 AC1 豁免清单) | 0 hits |
| PRD AC2 | grep cargo/crates/xtask/Cargo.toml (按 AC2 豁免) | 0 hits |
| PRD AC3 | `uv run pytest tests/python` | exit 0 |
| PRD AC4 | `verify-version-governance.py` | exit 0 |
| PRD AC5 | `verify-ornnlab-rebrand.py` | exit 0 + summary 一致 |
| PRD AC6 | frontend typecheck/lint/test/build | 全 exit 0 |
| PRD AC7 | `ls harnesslab/ crates/ xtask/ Cargo.toml ...` | 全 No such |
| PRD AC8 | grep "Retired in commit" rust-legacy-fate.md | 命中 1 |
| PRD AC9 | grep fix-plan Open Decisions | 含 "已退役 → 参见" |
| PRD AC10 | `git status --porcelain` + `git log origin/main..HEAD` | 空 + 0 |

#### Exit Criteria
- 10/10 AC PASS。
- subagent-vs-review 无 P0/P1 未关闭。

#### Review Plan
用户最终签字。

#### Risks And Fallback
| Risk | Impact | Trigger | Mitigation | Fallback |
|---|---|---|---|---|
| AC1/AC2 残留遗漏 | Medium | grep 非 0 | 回 Phase 1/2/3 删干净 | n/a |
| frontend `harnessLab` 残余 | Low | typecheck 红 | 复用 v0.1.4 fix plan Phase 3 流程 | n/a |
| subagent 新风险 | Variable | review 报告 P0/P1 | Phase 4 内修复 + re-review | 延后整改为 v0.1.5 work item |

#### Gate To Next Phase
n/a (本计划结束)。

## 11. Implementation Completeness Matrix（汇总）

| Plan Item | Phase | Status |
|---|---|---|
| 删 `harnesslab/` Python 包 | Phase 1 | planned |
| 删 `pyproject.toml` console script | Phase 1 | planned |
| 删 API 字段 `harnesslab_orphans` | Phase 1 | planned |
| 删 `settings.py` LEGACY_HOME / migration | Phase 2 | planned |
| 删 doctor RUNTIME_ENV_PAIRS 双发 | Phase 2 | planned |
| 删 harbor_subprocess / harbor_engine HARNESSLAB_HARBOR_* 回退 | Phase 2 | planned |
| 删 docker_orphan_service LEGACY_RUN_LABEL / 别名 | Phase 2 | planned |
| 删 backup_service LEGACY_MANIFEST_NAME 与旧文件识别 | Phase 2 | planned |
| 调整或删 9 个 tests/python/ shim 相关测试 | Phase 2 | planned |
| 删 `crates/`、`xtask/` | Phase 3 | planned |
| 删 `Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`coverage-critical.toml` | Phase 3 | planned |
| 删 Rust verify/CI 残留 | Phase 3 | planned |
| 翻转 `rust-legacy-fate.md` Retired | Phase 3 | planned |
| 闭环 fix-plan Open Decision | Phase 4 | planned |
| 10 AC PASS | Phase 4 | planned |

## 12. Risks, Dependencies, And Mitigations

| Risk | Impact | Trigger | Mitigation |
|---|---|---|---|
| OQ-1 决策推迟 | AC1 不能 0 命中 | 用户未决 | 将 `integrations/terminal_bench/` 加入 AC1 豁免清单 |
| settings 与启动耦合 | Phase 2 红 | pytest 红 | 重构 init 路径 |
| frontend 残留 | Phase 4 红 | typecheck 红 | 复用 v0.1.4 fix plan Phase 3 修复模式 |
| CI 隐藏 Rust 任务 | Phase 3 后下次 PR 红 | CI 红 | Phase 0 grep `.github/workflows/` 全覆盖 |
| Git push 失败 | 任意 phase 不能 push | `git push` 报错 | 本地保留 commit + 重试 |

## 13. Testing And Validation Strategy

- 每个 phase 出口：`uv run pytest tests/python` 必过；Phase 3 加 AC2 grep；Phase 4 跑全套 AC1–AC10。
- 测试退役决断准则：
  - 整文件只为 shim → 整删。
  - 混合文件 → 只删 legacy case。
- 回归保护：保留所有 OrnnLab 现役行为测试不动。
- 新断言：在 doctor / orphan 测试中追加"传入 HARNESSLAB_* 后不识别"的反向断言，固化"静默"行为。

## 14. Release, Rollback, And Fallback Strategy

- Release：内部仓库变更，无对外发布；下次 v0.1.4 发布时由 release ledger 关联本计划 commit 链。
- Rollback：每个 phase 一组 commit；任意 phase 失败可 `git revert <phase commit range>`。Phase 0/4 纯文档零回滚成本。
- Fallback：若 Phase 2 settings 删除引发不可恢复破坏，回滚单 commit 后改"局部重构 + 重试"。

## 15. Observability And Success Metrics

- 过程指标：每个 phase commit 的 hash + 时间戳；每个 phase 出口的命令输出存档在 Phase 结论小节。
- 结果指标：PRD AC1–AC10 全 PASS 凭据。
- 长期度量：未来若再发现 harnesslab 残留 → 回到本计划补 phase 而非另立项。

## 16. Open Questions

- **OQ-1 (Resolved 2026-06-22, option A)**：`integrations/terminal_bench/harnesslab_tb_*.py` (6 个文件) 处置：
  - **Decision**: A — 本计划内同步 rename 为 `ornnlab_tb_*.py`，并在新 Phase 2.5 中实施。
  - Rationale: 用户决定追求最大一致性，避免后续遗留独立 work item。AC1 grep 豁免清单不再包含 `integrations/terminal_bench/`。

## 17. Change Log

| Version | Date | Change |
|---|---|---|
| 1.0 | 2026-06-22 | 初稿 |

## 18. Plan Quality Checklist

- [x] 背景与问题定义清晰。
- [x] 目标可测量，非目标控制范围。
- [x] 事实 / 假设 / 约束 / 风险 / Open Question 分离。
- [x] 复杂度与计划深度匹配 (Medium → Standard 5 phases)。
- [x] 工作分阶段递进。
- [x] 每个 phase 有 entry / checks / tasks / deliverables / validation / exit / review / risks / fallback / gate。
- [x] 高风险/高不确定性先在 Phase 0 验证。
- [x] 风险含 trigger 与 mitigation。
- [x] 测试有 passing standard。
- [x] 完成性矩阵区分 landed 与 planned/partial/mock。
- [x] 生产影响：本工作为内部仓库变更，回滚/可观测以 git commit + 命令输出代替。
- [x] 数据影响：用户已确认无需保留旧数据，无需 migration。
- [x] 安全影响：无权限/认证/敏感数据边界变化。
- [x] 不发明仓库事实（所有引用都基于 Discovery 与 PRD）。
