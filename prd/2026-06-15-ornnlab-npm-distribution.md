# PRD: OrnnLab npm Distribution

- Status: Ready for implementation
- Created: 2026-06-15
- Updated: 2026-06-15
- Owner / requester: unknown
- Source request: Publish the current HarnessLab product under the occupied
  `ornnlab` npm package name.

## Requester Review Summary

- Key decisions:
  - `ornnlab` is the public npm package and command name for the current product
    distribution path.
  - The first npm release is a launcher over the GitHub source checkout, not a
    bundled native binary.
- `ornnlab setup` performs safe clone/update and dependency installation.
- Plain `ornnlab` performs setup if needed, starts the current WebUI, and prints
  the frontend URL for the user.
- `ornnlab dev`, `ornnlab web`, `ornnlab ui`, and `ornnlab doctor` remain
  explicit workflow commands.
- Important exceptions:
  - The launcher does not delete or overwrite user data.
  - The launcher requires `git`, `uv`, Node.js, and npm on `PATH`.
- Must-confirm before implementation:
  - None for the current source-launcher release.
- Status reason:
  - The user explicitly chose the occupied `ornnlab` package name and the
    current product already has a source-based launch path.

## 1. Background And Product Intent

Users should be able to install a stable command from npm instead of cloning the
repository manually. The command should make the current Harbor WebUI build
usable while the project still lacks a fully bundled desktop/native package.

## 2. Goals And Success Criteria

- `npm install -g ornnlab` installs an `ornnlab` command.
- `ornnlab --version` prints the npm launcher version.
- `ornnlab --help` explains setup and run commands.
- `ornnlab setup` clones or fast-forwards the HarnessLab source checkout and
  installs backend/frontend dependencies.
- `ornnlab` starts backend and frontend dev servers for the current MVP.
- The terminal prints `Frontend: http://127.0.0.1:5173/` before server logs.
- Local package smoke and registry smoke can verify the release.

## 3. Users And Usage Context

Primary user: local developer/operator evaluating HarnessLab as a Harbor WebUI.

Usage context: single-user local machine with npm, Node.js, `uv`, and git
available.

## 4. Scope

### In Scope

- Unscoped npm package name: `ornnlab`.
- CLI command name: `ornnlab`.
- Source checkout management under `~/.ornnlab/HarnessLab` by default.
- Backend, frontend, doctor, and path commands.

### Out Of Scope

- Bundled Python runtime.
- Bundled frontend static serving from FastAPI.
- Native app packaging.
- Automatic Docker installation.
- Automatic deletion of old checkouts or user data.

## 5. Core User Journey

1. User runs `npm install -g ornnlab`.
2. User runs `ornnlab`.
3. Launcher clones or updates the source checkout and installs dependencies if
   needed.
4. Launcher prints the frontend URL.
5. Launcher starts backend and frontend development servers.
6. User opens the frontend URL printed in the terminal.

## 6. Interaction And Information Design

The command help must list setup, dev, backend, frontend, doctor, and path
commands. Errors must be direct and actionable, especially missing prerequisite
commands or missing source checkout.

## 7. Product Rules And State Logic

- Default state path is `~/.ornnlab/HarnessLab`.
- `ORNNLAB_HOME`, `ORNNLAB_SOURCE`, and `ORNNLAB_REPO` may override defaults.
- Existing non-git source paths are not overwritten.
- Existing git checkouts are updated only with `git pull --ff-only`.
- Runtime product data remains under the current HarnessLab default
  `~/.harnesslab`.

## 8. Edge Cases, Errors, And Recovery

- Missing `git`, `uv`, or `npm`: fail with the missing command name.
- Source checkout absent for run commands: tell the user to run `ornnlab setup`.
- Existing non-git source path: fail without modifying it.
- Dependency install failure: leave the checkout in place for manual inspection.
- Frontend/backend process termination: forward `SIGTERM` on launcher shutdown.

## 9. Content And Terminology

- Public install name: `ornnlab`.
- Product name remains HarnessLab/OrnnLab during transition.
- Help text should call this an npm launcher, not a complete native bundle.

## 10. Acceptance Criteria

- `npm run smoke:npm-bin` passes.
- `npm pack --dry-run --json` contains only `LICENSE`, `README.md`,
  `bin/ornnlab.js`, and `package.json`.
- Clean local tarball install exposes `ornnlab --version` and `ornnlab --help`.
- Launcher help documents that plain `ornnlab` starts the local WebUI.
- Launcher help documents the printed frontend URL behavior.
- After publish, `npm view ornnlab name version bin --json` returns the new
  version and `bin.ornnlab`.
- After publish, clean `npx --yes ornnlab --version` returns the new version.

## 11. Review Checklist And Sign-off Questions

- Does `ornnlab` remain the desired public package and command name?
- Is a source-launcher npm package acceptable until a bundled native package is
  built?
- Are `git`, `uv`, Node.js, and npm acceptable prerequisites for this release?

## 12. Clarification Decision Log

| Topic | Decision | Rationale | Source Round |
|---|---|---|---|
| Public package | `ornnlab` | User said the name is occupied and should be used. | Initial request |
| Distribution shape | source launcher | Matches current product state without overclaiming bundled packaging. | Implementation review |
| Safety | no deletion or overwrite | Repository rules require recoverable operations. | AGENTS.md |
