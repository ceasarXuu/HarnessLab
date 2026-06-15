# Rust Legacy Workspace Fate

Decision date: 2026-06-15

The Rust workspace remains in the repository as a legacy/reference asset. It is
not the active implementation path for the Harbor WebUI rewrite.

## Decision

- Keep `Cargo.toml`, `crates/`, and `xtask/` in place for now.
- Do not extend Rust as the main product runtime.
- Do not include Rust jobs in the default WebUI CI matrix.
- Treat previous Rust tests and docs as historical evidence for requirements,
  redaction, adapter lessons, and operational discipline.
- Revisit archival only through a separate reversible migration plan.

## Rationale

The active product delegates benchmark execution to Harbor and uses
Python/FastAPI plus Vue for the local WebUI. Moving or deleting the Rust
workspace during this rewrite would create unnecessary recovery risk while the
new product is still being hardened.

Keeping the Rust workspace as legacy reference also preserves prior evidence for
adapter contracts, public/private artifact boundaries, replay lessons, and
Docker diagnostics without making those crates part of the release surface.

## Guardrails

- New runtime behavior belongs in `harnesslab/`, `frontend/`, and current WebUI
  tests unless a new architecture decision says otherwise.
- Active docs must point to the Harbor WebUI plan, not Rust CLI plans.
- Any future archival must be recoverable: move to an archive path in git or a
  backup location, never through irreversible deletion.
- If a Rust file needs modification for repository hygiene, keep the change
  scoped and run the relevant Rust gate explicitly.
