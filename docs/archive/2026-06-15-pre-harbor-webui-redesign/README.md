# Pre Harbor WebUI Redesign Archive

- Archived: 2026-06-15
- Reason: HarnessLab product direction changed from a CLI-first self-owned Rust runtime to a Harbor-powered local WebUI.
- Canonical PRD: `../../../prd/2026-06-15-harnesslab-webui-prd.md`
- Canonical engineering plan: `../../plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

These documents are preserved for historical context only. They should not be
used as implementation source of truth.

## Archived Documents

| Original path | Archived path | Why archived |
|---|---|---|
| `docs/legacy/prd.md` | `prd.md` | CLI-first product direction |
| `prd/2026-06-07-universal-benchmark-adapter-protocol.md` | `prd/2026-06-07-universal-benchmark-adapter-protocol.md` | Universal adapter protocol PRD superseded by Harbor engine ownership |
| `docs/legacy/mvp-development-spec.md` | `mvp-development-spec.md` | Rust runtime MVP execution spec |
| `docs/legacy/architecture.md` | `architecture.md` | Self-owned run orchestrator architecture |
| `docs/current/technology-decisions.md` | `technology-decisions.md` | Rust single-binary technology decision |
| `docs/legacy/adapter-protocol.md` | `adapter-protocol.md` | Self-owned benchmark adapter/runtime protocol |
| `docs/legacy/agent-registration-guide.md` | `agent-registration-guide.md` | CLI-first AgentProfile v1 registration flow |
| `docs/legacy/agent-profile-reference.md` | `agent-profile-reference.md` | CLI-first AgentProfile v1 schema reference |
| `docs/current/development-operations.md` | `development-operations.md` | Rust CLI operation notes |
| `docs/current/test-engineering.md` | `test-engineering.md` | Rust/Cargo test-engineering system |
| `docs/playbooks/terminal-bench-claude-ds.md` | `playbooks/terminal-bench-claude-ds.md` | CLI Terminal-Bench user flow |
| `docs/reviews/2026-05-27-docker-runner-review-3.md` | `reviews/2026-05-27-docker-runner-review-3.md` | Legacy Docker runner review |
| `docs/plans/2026-06-03-agent-registration-gap-completion.md` | `plans/2026-06-03-agent-registration-gap-completion.md` | Rust Agent Registry closure plan |
| `docs/plans/2026-06-03-agent-registration-registry.md` | `plans/2026-06-03-agent-registration-registry.md` | Rust Agent Registry implementation plan |
| `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md` | `plans/2026-06-04-benchmark-adapter-architecture-design.md` | Rust adapter architecture plan |
| `docs/plans/2026-06-04-benchmark-adapter-phase-*.md` | `plans/2026-06-04-benchmark-adapter-phase-*.md` | Rust adapter phase artifacts |
| `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md` | `plans/2026-06-05-benchmark-adapter-phase-2-inventory.md` | Rust adapter phase artifact |
| `docs/plans/2026-06-06-benchmark-adapter-phase-*.md` | `plans/2026-06-06-benchmark-adapter-phase-*.md` | Rust adapter runtime phase artifacts |
| `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-*.md` | `plans/2026-06-08-universal-benchmark-adapter-protocol-*.md` | Universal adapter protocol artifacts |
| `docs/plans/2026-06-12-remove-external-runner-kind-plan.md` | `plans/2026-06-12-remove-external-runner-kind-plan.md` | Legacy runner-kind cleanup plan |
| `docs/plans/2026-06-15-harbor-integration-engineering-plan.md` | `plans/2026-06-15-harbor-integration-engineering-plan.md` | Rust + Python Bridge plan |
| `docs/plans/2026-06-15-harnesslab-webui-engineering-plan.md` | `plans/2026-06-15-harnesslab-webui-engineering-plan.md` | First WebUI pivot draft superseded by v3 plan |
| `docs/architecture/benchmark-compatibility-strategy.md` | `architecture/benchmark-compatibility-strategy.md` | Self-owned benchmark runtime strategy |
| `docs/architecture/harnesslab-vs-harbor.md` | `architecture/harnesslab-vs-harbor.md` | Intermediate comparison that still proposed HarnessLab runtime ownership |

Original paths now contain stubs that point to the current plan or this archive.
The repository root `README.md` and `docs/index/README.md` also point readers to the
active Harbor WebUI redesign.
