# HarnessLab Documentation

Start here for the active Harbor WebUI redesign.

- Current PRD: `../prd/2026-06-15-harnesslab-webui-prd.md`
- Current engineering plan: `plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- Legacy archive: `archive/2026-06-15-pre-harbor-webui-redesign/README.md`

## Current Direction

HarnessLab is a local WebUI over Harbor. Harbor owns benchmark execution and raw
job artifacts. HarnessLab owns agent registration, experiment/run management,
diagnostics, report summaries, and leaderboard views.

## Superseded Documents

Root-level architecture, technology, adapter protocol, original PRD, and MVP
spec files are now short stubs that point to the active plan or the archive.
The previous `prd/2026-06-07-universal-benchmark-adapter-protocol.md` file is
also a stub; its full content is preserved in the archive.
Those stubs exist to keep old references resolvable while preventing stale
Rust-runtime guidance from being read as current direction.
