# PRD: HarnessLab 兼容层与 Rust Legacy 立即退役

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set (planned `2026.06.22`); `ornnlab` npm `0.1.4` (planned); Python app `0.2.0` | 2026-06-22 | Initial Ready-for-implementation PRD: retire all HarnessLab compatibility shim layers and the Rust legacy workspace immediately, given the project is pre-release with no real legacy users. |

- Status: Ready for implementation
- Created: 2026-06-22
- Updated: 2026-06-22
- Owner / requester: User (项目所有者)
- Source request: "现在就可以退役掉了，这是个未发布的早期项目，旧数据已经无用"

## Requester Review Summary

- Key decisions:
  - **范围最大化**：所有 HarnessLab 兼容 shim（Python 包/入口、运行时数据/环境变量、Docker/Backup、迁移测试与 verify 巡检）一次性退役。
  - **npm 维持现状**：保留 `npm/harnesslab-transition/`、`bin/harnesslab.js`，已发布的 `@ceasarxuu/harnesslab` 包不 deprecate。
  - **Rust legacy workspace 同步退役**：`crates/`、`xtask/`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`integrations/terminal_bench/` 的 Rust 残留、Rust 相关 verify/CI/coverage 一并清除。
  - **doctor 静默策略**：遇到 `HARNESSLAB_*` 环境变量、`~/.harnesslab` 路径、`harnesslab.run_id` 容器一律不识别、不提示、不警告。
  - **删除方式**：B 选项 — `git rm` 直接删除（用户明确放弃"禁止不可恢复删除"用户规则保护，依赖 git 历史回溯）。
- Important exceptions:
  - npm 包及 npm 仓库内的转换占位代码不在本次工作项内（独立决策推迟）。
- Must-confirm before implementation:
  - 实施期间运行 [scripts/verify-ornnlab-rebrand.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/scripts/verify-ornnlab-rebrand.py) 的 `_check_migration_tests()` 巡检会与本次退役冲突——必须同步移除该巡检函数。
  - [docs/archive/stubs/rust-legacy-fate.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/archive/stubs/rust-legacy-fate.md) 状态从"保留"改为"已退役"，并标注退役 commit。
- Status reason:
  - 所有产品边界决策已澄清，PRD 直接进入 `Ready for implementation`。

## 1. Background And Product Intent

OrnnLab 仍处于未发布的早期阶段，无外部用户。前一阶段 HarnessLab → OrnnLab
品牌迁移按"渐进迁移、保留兼容层"原则实施，引入了一整套 shim：

- 旧 Python 包入口 `harnesslab/` 仍可工作并发出 `DeprecationWarning`。
- 旧用户数据 `~/.harnesslab/`、`harnesslab.sqlite` 会被识别并迁移。
- 旧环境变量 `HARNESSLAB_HARBOR_*`、`HARNESSLAB_DOCKER_COMMAND`、`HARNESSLAB_REAL_HARBOR_*`、`HARNESSLAB_HOME` 等仍生效。
- 旧 Docker 标签 `harnesslab.run_id`、旧 backup manifest `harnesslab-backup-manifest.json` 仍被识别。
- API 字段 `harnesslab_orphans` 与 `ornnlab_orphans` 双发。
- 一批迁移测试断言这些 shim 不被无意删除。
- Rust legacy workspace（自建 runtime 时代产物）按 [rust-legacy-fate.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/archive/stubs/rust-legacy-fate.md) 决策保留为 reference。

由于"无真实用户、无需迁移窗口"，兼容层的维护成本（混淆新人、阻碍重构、
拖慢 verify 脚本、保留 dead 测试）已经超过其收益。同时 Rust legacy 在项目
切换到 Harbor 后实质上不会被再次启用，继续保留只会让仓库噪音和
clone/索引成本增加。

## 2. Goals And Success Criteria

主要目标：

1. 让仓库的"当前态"与"产品现状"完全一致：只有 OrnnLab 这一个品牌、一种数据路径、一套环境变量、一种 Docker 标签、一种 backup manifest。
2. 让 `git grep -i harnesslab` 在 active 表面（排除 `docs/archive/`、`docs/plans/`、`docs/releases/v0.1.4/*-fix-plan.md`、`vs_review/`、`coe/`、`npm/harnesslab-transition/`、`bin/harnesslab.js`）返回 0 命中。
3. 让 `cargo` 这个词在仓库中彻底消失（除 `docs/archive/` 与 git 历史外）。
4. doctor / settings / 服务层不再尝试读取任何 legacy 路径或环境变量。

成功标准：

- 所有验证脚本通过：[scripts/verify-version-governance.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/scripts/verify-version-governance.py)、[scripts/verify-ornnlab-rebrand.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/scripts/verify-ornnlab-rebrand.py) 适配后 exit 0。
- `uv run pytest tests/python` 全过（迁移测试已按 PRD 删除）。
- `npm --prefix frontend run typecheck && lint && test && build` 全过。
- 仓库内（排除豁免目录）`rg -i "harnesslab"` 与 `rg -i "cargo|crates/|xtask"` 命中均符合 PRD 第 4/8 节的"允许残留"白名单。
- [docs/archive/stubs/rust-legacy-fate.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/archive/stubs/rust-legacy-fate.md) 标注"Rust workspace retired in commit <sha>, on 2026-06-22"。

## 3. Users And Usage Context

- **本仓库开发者**（主用户）：希望仓库结构清晰、不被 legacy 干扰。
- **未来贡献者**：搜索代码时不应被 shim/legacy 误导。
- **外部 npm 安装用户**（潜在）：仍可通过老包 `@ceasarxuu/harnesslab` 触发转换提示（不变）。
- **无**：真实的 `~/.harnesslab` 数据用户、真实的 `HARNESSLAB_*` 环境变量使用者、真实的 Rust workspace 构建用户。

## 4. Scope

### In Scope

#### A. Python 包 / 入口 / 元数据

- 删除 [harnesslab/](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/harnesslab) 整个目录（`__init__.py`、`__main__.py`、`cli.py`）。
- [pyproject.toml#L16](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/pyproject.toml#L16) 删除 `harnesslab = "ornnlab.cli:main"` 条目。
- 任何 `pyproject.toml` / `tests` 配置中提到 `harnesslab` 的部分。

#### B. 运行时数据 / 环境变量 / 迁移逻辑

在 [ornnlab/settings.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/settings.py) 中：

- 删除 `LEGACY_HOME = Path.home() / ".harnesslab"` 与所有引用。
- 删除 `HARNESSLAB_HOME` 环境变量读取与所有引用。
- 删除 `harnesslab.sqlite` legacy DB 路径识别。
- 删除"首次运行迁移" `migration/ornnlab-home-migration.json` 落地逻辑。

在 [ornnlab/services/doctor_service.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/services/doctor_service.py) 中：

- 删除 `RUNTIME_ENV_PAIRS` 老/新双发表，只保留 `ORNNLAB_*` 单边。
- 删除 `harnesslab_orphans` API 字段双发；保留 `ornnlab_orphans`。
- 删除任何"读到 HARNESSLAB_* 时给 warning"的代码路径。

在 [ornnlab/services/harbor_subprocess.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/services/harbor_subprocess.py)、[ornnlab/services/harbor_engine.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/services/harbor_engine.py) 中：

- 删除 `HARNESSLAB_HARBOR_ENGINE`、`HARNESSLAB_HARBOR_SUBPROCESS_COMMAND`、`HARNESSLAB_DOCKER_COMMAND`、`HARNESSLAB_REAL_HARBOR*` 全部读取分支。
- 只保留 `ORNNLAB_*` 一套。

#### C. Docker / Backup

- [ornnlab/services/docker_orphan_service.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/services/docker_orphan_service.py)：删除 `LEGACY_RUN_LABEL = "harnesslab.run_id"` 与 `scan_harnesslab_containers` 别名方法及调用方。
- [ornnlab/services/backup_service.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/services/backup_service.py)：删除 `LEGACY_MANIFEST_NAME = "harnesslab-backup-manifest.json"`、旧 `harnesslab.sqlite-wal/-shm` 识别、旧 manifest 兼容读取。

#### D. 迁移测试

`tests/python/` 下与 shim 相关的测试整文件删除（按当前态枚举）：

- `test_settings_migration.py`（如果只测 legacy → ornnlab 迁移）
- `test_docker_orphan_service.py` 中涉及 `harnesslab.run_id` 的 case
- `test_backup_service.py` 中涉及 `harnesslab-backup-manifest.json` 的 case
- `test_harbor_config.py` 中涉及 `HARNESSLAB_HARBOR_*` 回退的 case
- 实施时按"全文件删 vs 删特定测试 case"逐文件决断；准则：测试还有其它仍有效断言 → 只删 legacy case；测试只为 shim 而存在 → 整删。

#### E. Verify 脚本

- [scripts/verify-ornnlab-rebrand.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/scripts/verify-ornnlab-rebrand.py) 删除 `_check_migration_tests()` 函数及其调用；将 10/10 降为对应数字（实施时确定）。
- 移除 `MIGRATION_TEST_TARGETS` 之类与 shim 相关的常量。
- 顶层 `rg "harnesslab"` 改为禁止策略（除豁免目录），由 verify 脚本守卫。

#### F. Rust legacy workspace

整目录/整文件 `git rm`：

- [crates/](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/crates) 全部
- [xtask/](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/xtask) 全部
- [Cargo.toml](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/Cargo.toml)
- `Cargo.lock`
- `rust-toolchain.toml`
- `coverage-critical.toml`（若仅服务 Rust 测试覆盖）
- [integrations/terminal_bench/](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/integrations/terminal_bench) 中纯 Rust 部分；如果该目录是 Python 测试基准且有非 Rust 内容，则只删 Rust 部分。
- 所有 `scripts/verify-terminal-bench-*.sh`、`scripts/scan-artifacts-for-secrets.sh` 中如果服务于 Rust 制品扫描 → 一并删；如果通用 → 保留。

更新 [docs/archive/stubs/rust-legacy-fate.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/archive/stubs/rust-legacy-fate.md)：
状态改为 "Retired"，并加退役 commit/date。

#### G. 文档

- 关闭 [docs/releases/v0.1.4/shim-retirement/harbor-rebrand-residue-fix-plan.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/shim-retirement/harbor-rebrand-residue-fix-plan.md) 中 "HarnessLab 兼容 shim 退役时间表" 这条 open item（在 Open Decisions 章节追加退役决议链接）。
- 更新 [docs/releases/v0.1.4/v0.1.4-docs.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/v0.1.4-docs.md)：登记本 PRD + 即将产出的实施计划。
- 同步更新 [docs/releases/v0.1.3/version-governance.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.3/version-governance.md) Document Control 与 Active Index，新增本 PRD。

### Out Of Scope

- **npm**：仓库内 `npm/harnesslab-transition/` 与 `bin/harnesslab.js` 不动；npm 上 `@ceasarxuu/harnesslab` 已发布版本不 `deprecate`。
- **OrnnLab 自身核心功能**：本工作项不引入新功能、不重构现有 OrnnLab 业务代码（除删除 shim 分支必需的最小改动外）。
- **历史归档**：`docs/archive/**`、`docs/plans/**`、`vs_review/**`、`coe/**` 中的 HarnessLab/Rust 词条保留作为审计证据。
- **v0.1.3 release ledger**：已发布版本快照不回改。

## 5. Core User Journey

实施后（开发者视角）：

1. 新开发者 clone 仓库，看不到 `harnesslab/` 包目录、Rust workspace、`Cargo.toml`。
2. 设置 `HARNESSLAB_HARBOR_ENGINE=fake` 跑测试 → 测试**不识别**该变量，会因 `ORNNLAB_HARBOR_ENGINE` 缺失而走默认路径或报真实错（按当前默认行为）。
3. 把 `~/.harnesslab/harnesslab.sqlite` 拷贝到本机 → OrnnLab 启动时**完全不读取**，从 `~/.ornnlab/ornnlab.sqlite` 全新创建。
4. 运行 `harnesslab --version` → `command not found`（因为 console script 已删）。
5. 运行 `python -m harnesslab` → `No module named harnesslab`。
6. doctor 命令对 `HARNESSLAB_*` 变量**完全静默**，不输出任何相关字段。

## 6. Interaction And Information Design

无 UI 改动。CLI/API 层面：

- 删除 API 响应中的 `harnesslab_orphans` 字段。前端代码已扫描确认未使用该字段。
- doctor 命令的输出字段表只包含 OrnnLab 名称的字段。

## 7. Product Rules And State Logic

- **不识别原则**：所有 HarnessLab 命名的环境变量、路径、文件、标签、API 字段在代码层面"不存在"，等同于一个不认识的字符串。
- **静默原则**：不警告、不报错（除非该字符串恰好出现在用户**显式输入**的位置导致正常代码路径的语义错误，例如 `--config=HARNESSLAB_HOME` 这种）。
- **彻底删除原则**：相关代码路径整段删除，不留 `# legacy:` 注释、不留 `if False:` 死分支、不留 deprecated 函数空 stub。
- **审计可回溯**：所有删除走 git 历史，保留 PRD + 实施计划文档作为决策证据。

## 8. Edge Cases, Errors, And Recovery

| 场景 | 行为 |
|---|---|
| 用户机器还有 `~/.harnesslab/` 残留 | 静默无视，新跑 OrnnLab 在 `~/.ornnlab/` 全新初始化 |
| CI 环境变量仍有 `HARNESSLAB_HARBOR_ENGINE=fake` | 不识别，相当于未设置；如果 CI 依赖该变量驱动 fake harbor，需用户/我同步改 CI 为 `ORNNLAB_HARBOR_ENGINE` |
| Docker 主机上仍有 `harnesslab.run_id` 标签的孤儿容器 | 不识别，不清理；用户需手动 `docker ps --filter "label=harnesslab.run_id" -aq \| xargs docker rm -f` |
| 旧 backup 文件 `harnesslab-backup-manifest.json` | 不识别，无法恢复；用户须接受 |
| `harnesslab --version` / `python -m harnesslab` | `command not found` / `No module named harnesslab` — 期望行为，不修复 |
| 第三方 Python 代码 `import harnesslab` | `ModuleNotFoundError` — 期望行为 |
| Rust 构建工具调用 `cargo build` 等 | 命令仍存在（来自系统 toolchain），但仓库内无 `Cargo.toml` → 报 manifest 未找到错误 |

## 9. Content And Terminology

- 文档中"HarnessLab 兼容 shim"统一改用过去时表述："曾经存在的 HarnessLab 兼容层（已于 2026-06-22 退役）"。
- [rust-legacy-fate.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/archive/stubs/rust-legacy-fate.md) 状态字段从 `Kept as reference` 改为 `Retired`。
- v0.1.4 文档索引中新增 work item 行："HarnessLab Shim + Rust Legacy Retirement"。

## 10. Acceptance Criteria

- **AC1**：`rg -i "harnesslab" -g '!docs/archive/**' -g '!docs/plans/**' -g '!docs/releases/v0.1.3/**' -g '!docs/releases/v0.1.4/**' -g '!vs_review/**' -g '!coe/**' -g '!npm/harnesslab-transition/**' -g '!bin/harnesslab.js' -g '!docs/architecture/harnesslab-vs-harbor.md' -g '!docs/playbooks/npm-package-reservation.md' -g '!README.md' -g '!package.json' -g '!lib/source.js' -g '!scripts/verify-harnesslab-transition-package.sh' -g '!scripts/test-after-change-web.sh' -g '!scripts/verify-ornnlab-rebrand.py' -g '!tests/python/test_harbor_subprocess.py' -g '!.git/**'` 返回 0 命中。豁免说明：(1) npm transition / GitHub repo URL / 设计内巡检字符串属于 Out-of-scope；(2) v0.1.3 ledger 是已发布历史；(3) v0.1.4 整目录在豁免内（含 shim-retirement work-item 文档与 v0.1.4-docs.md 索引）；(4) `tests/python/test_harbor_subprocess.py` 内 SC-5 跟进保留 `HARNESSLAB_HARBOR_SUBPROCESS_COMMAND` 字符串作为回归守卫（生产代码不读该变量，仅测试集合中设置以验证忽略行为）；(5) `docs/architecture/harnesslab-vs-harbor.md` 是 historical stub，文件名与内文均描述历史品牌；(6) README/playbook 描述 npm transition 历史。
- **AC2**：`rg -n "cargo|crates/|xtask|Cargo\.toml" -g '!docs/archive/**' -g '!docs/plans/**' -g '!.git/**'` 返回 0 命中（除 PRD/计划文档中描述性提及外）。
- **AC3**：`uv run pytest tests/python` exit 0；删除的测试 case 不再存在。
- **AC4**：`uv run python scripts/verify-version-governance.py` exit 0。
- **AC5**：`uv run python scripts/verify-ornnlab-rebrand.py` exit 0；`_check_migration_tests` 已移除；总检查数与脚本内 summary 一致。
- **AC6**：`npm --prefix frontend run typecheck && npm --prefix frontend run lint && npm --prefix frontend run test && npm --prefix frontend run build` 全 exit 0。
- **AC7**：`harnesslab/` 目录、`crates/`、`xtask/`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml` 在仓库工作树中均不存在（`ls` 报 No such file or directory）。
- **AC8**：[rust-legacy-fate.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/archive/stubs/rust-legacy-fate.md) 含 "Retired in commit <sha> on 2026-06-22" 字样。
- **AC9**：[docs/releases/v0.1.4/shim-retirement/harbor-rebrand-residue-fix-plan.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/shim-retirement/harbor-rebrand-residue-fix-plan.md) Open Decisions 章节中"HarnessLab 兼容 shim 退役时间表"条目改为"已退役 → 参见 harnesslab-shim-retirement-prd.md"。
- **AC10**：所有改动以最小化提交原则分多个 commit，且 push 到 `origin/main`，工作树 clean。

## 11. Review Checklist And Sign-off Questions

- [ ] 你确认放弃"禁止不可恢复删除"用户规则（已通过 B 选项确认，记入决策日志）。
- [ ] 你确认本机 / CI / Docker 主机上没有 HarnessLab 残留数据需要保留。
- [ ] 你确认 `terminal_bench` 集成中如果有非 Rust 内容（Python/Markdown 等），我可以保留这些非 Rust 部分；如果全是 Rust，则整目录删。
- [ ] 你确认 `coverage-critical.toml` 如果不仅服务 Rust，可以保留（实施时检查后决定）。
- [ ] 你确认本 PRD 编号为 v0.1.4 的第二个 work item，与 `harbor-rebrand-residue-fix-plan.md` 并列。

## 12. Clarification Decision Log

| Topic | Decision | Rationale | Source Round |
|---|---|---|---|
| 退役时机 | 立即退役 | 项目未发布、无用户、无旧数据 | 用户首轮指令 |
| 退役范围 | 最大范围 | 一次性切干净避免后续遗留 | 选项 1 |
| npm 处置 | 维持现状 | npm 端独立决策，与代码退役解耦 | 选项 2 |
| Rust workspace | 同步退役 | "用不上的话"——切到 Harbor 后无需 Rust runtime | 选项 3 |
| doctor 提示策略 | 完全静默 | "彻底干净"——避免成为半退役状态 | 选项 4 |
| 删除方式 | `git rm` 直删 | 用户明确允许，git 历史可恢复 | 选项 B |

## 13. Open Questions And Risks

- **R1**：实施过程中如果发现某段 shim 与 OrnnLab 现役代码耦合很深，删除会带连锁影响（例如 settings 中的 migration 调用是 `init_db()` 的前置 step），需要先重构后再删。此时按"长期主义"原则，正确重构 + 退役一并提交，而不是"先留 shim"。
- **R2**：[integrations/terminal_bench/](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/integrations/terminal_bench) 目录可能混合 Rust + 非 Rust 内容，实施前需要 `ls` 一次后逐项决断。
- **R3**：`coverage-critical.toml` 用途不明，需要先 grep 确认它服务于哪个 toolchain；如果是 Rust-only → 删，否则保留。
- **R4**：删除 `harnesslab_orphans` API 字段是 break API change。前端已扫描确认未用，但如果有任何 OpenAPI 文档/Schema 文件，需要同步更新。
- **R5**：[scripts/verify-ornnlab-rebrand.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/scripts/verify-ornnlab-rebrand.py) 自身在 DOC_INVENTORY 等地方包含 "harnesslab" 字符串（作为兼容验证的对照），这些是设计内的，**不需要也不能删**，需要 AC1 的 grep 显式豁免该文件。

## 14. Implementation Notes

- **审计驱动**：每一处删除前先用 `rg` 列出所有引用方，确保不会留下 dangling reference。
- **最小化提交**：按 ABCDEFG 分组的边界尽量分 commit，每个 commit 后跑相关测试。
- **测试驱动**：删除 shim 同时删除/调整其对应测试，确保 `pytest` 在每个 commit 后通过。
- **审查驱动**：实施后用 `subagent-vs-review` 跑一轮对抗性审查。
- **日志驱动**：删除后用 [rust-legacy-fate.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/archive/stubs/rust-legacy-fate.md) + v0.1.4 文档索引登记本次退役决议，未来回查有迹可循。
- **工程计划文档**：本 PRD 完成后再产出 `docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-plan.md` 作为分阶段实施计划（参考 [harbor-rebrand-residue-fix-plan.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/shim-retirement/harbor-rebrand-residue-fix-plan.md) 的结构）。
