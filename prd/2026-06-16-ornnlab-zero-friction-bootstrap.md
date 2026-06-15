# PRD: OrnnLab Zero-Friction Bootstrap

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | `ornnlab` npm `0.1.2` | 2026-06-16 | Defined blank-machine bootstrap, install, update, and uninstall product behavior. |
| 1.1 | `ornnlab` npm `0.1.3` | 2026-06-16 | Added core-only Docker policy, state versioning, and dependency integrity checks. |
| 1.2 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Linked bootstrap requirements to document version governance. |

- Status: Ready for implementation
- Created: 2026-06-16
- Updated: 2026-06-16
- Owner / requester: xuzhang
- Source request: Make first-run OrnnLab initialization install required dependencies on a blank computer and avoid user setup friction.

## Requester Review Summary

- Key decisions:
  - The launcher should automatically install recommended runtime dependencies instead of only printing manual instructions.
  - The first implementation should cover macOS, Linux, and Windows.
  - Docker should be detected and recorded when present; when missing, the user gets a yes/no installation choice and may skip it.
  - Automatic installation may request interactive privilege escalation, but the launcher must show the command before running it.
  - Docker installation must stay lightweight and core-only; OrnnLab must not install Docker Desktop.
- Important exceptions:
  - `npm install -g ornnlab` still requires a working Node/npm entrypoint before the launcher can run.
  - Docker is optional for initial WebUI startup because it is mainly needed for real container-backed evaluation flows.
  - Installing Docker tooling is not the same as guaranteeing a running Docker daemon.
- Must-confirm before implementation:
  - None. Docker is explicitly optional and retryable later.
- Status reason:
  - The product behavior and acceptance criteria are defined enough to implement.

## 1. Background And Product Intent

OrnnLab is distributed through npm as a lightweight launcher. The current launcher
can clone the source checkout and install Python/frontend project dependencies,
but it assumes `git`, `uv`, Node.js, and npm already exist on the user's PATH.
That creates first-use friction and makes a blank-machine install feel broken.

The product intent is to make `ornnlab` behave like a self-guided local app
bootstrapper: it should detect missing prerequisites, install what is needed
where safe, explain what is happening, recover from interrupted setup, and start
the WebUI when the machine is ready.

## 2. Goals And Success Criteria

- A user who can install the npm package can run `ornnlab` and be guided through
  the rest of the runtime setup.
- Missing recommended dependencies are automatically installed on supported
  platforms when a known package manager or installer path exists.
- The launcher does not silently skip half-installed states; rerunning `ornnlab`
  resumes or repairs incomplete setup.
- Docker is handled as an optional capability with a clear yes/no choice.
- Docker install uses lightweight core tooling only; Docker Desktop is never installed by OrnnLab.
- Bootstrap state is versioned so future launcher releases can migrate or rerun stale setup stages.
- Dependency readiness is verified by real backend/frontend checks, not only by directory existence.
- The user sees phase-oriented output instead of an unexplained wall of build
  logs.

## 3. Users And Usage Context

Primary user:

- A developer or evaluator installing OrnnLab locally from npm.

Usage context:

- Fresh macOS, Linux, or Windows machine.
- The user may not know which developer tools are required.
- The user wants the WebUI to start, not to read a multi-step setup guide.

## 4. Scope

### In Scope

- Detect `git`, `uv`, Node.js/npm, and Docker.
- Automatically install required non-Docker prerequisites when possible.
- Detect and record Docker capability.
- Ask whether to install lightweight Docker tooling when missing.
- Allow Docker skip and retry later.
- Clone/update the source checkout.
- Install Python backend dependencies.
- Install frontend dependencies.
- Provide `ornnlab update` to refresh the managed source checkout and
  dependencies after installation.
- Provide `ornnlab uninstall` to remove launcher-managed files while preserving
  user data by default.
- Persist bootstrap state for diagnostics and recovery.
- Make setup idempotent and recoverable after partial failure.

### Out Of Scope

- Replacing npm as the initial distribution mechanism.
- Fully unattended privileged OS setup in every enterprise-managed environment.
- Installing Docker Desktop.
- Guaranteeing Docker daemon startup, OS-level permission grants, runtime
  initialization, or Windows WSL Docker Engine provisioning.
- Changing Harbor execution ownership.

## 5. Core User Journey

1. User installs the package:
   ```bash
   npm install -g ornnlab
   ```
2. User runs:
   ```bash
   ornnlab
   ```
3. The launcher prints bootstrap phases:
   - checking prerequisites
   - installing missing prerequisites
   - checking Docker capability
   - preparing source checkout
   - syncing Python dependencies
   - installing frontend dependencies
   - starting WebUI
4. If Docker is missing, the launcher asks:
   ```text
   Docker is optional for first launch. Install Docker now? [y/N]
   ```
5. If the user skips Docker, WebUI startup continues and the state records
   Docker as skipped.
6. If a later workflow requires Docker, the product can offer the installation
   choice again.

Update journey:

1. User runs:
   ```bash
   ornnlab update
   ```
2. Launcher checks the published npm launcher version and the installed launcher
   version.
3. If the global npm launcher is behind, the launcher prints the explicit command:
   ```bash
   npm install -g ornnlab@latest
   ```
4. Launcher updates the managed source checkout, reruns Python dependency sync,
   reruns frontend dependency install, records update state, and exits without
   starting the WebUI.

Uninstall journey:

1. User runs:
   ```bash
   ornnlab uninstall
   ```
2. Launcher lists launcher-managed files that can be removed.
3. Launcher preserves `~/.ornnlab/data` unless the user explicitly requests a
   full data cleanup.
4. Launcher removes or archives recoverably:
   - source checkout under `~/.ornnlab/launcher/source`
   - bootstrap state under `~/.ornnlab/launcher/bootstrap-state.json`
   - launcher cache files under `~/.ornnlab/launcher`
5. Launcher prints:
   ```bash
   npm uninstall -g ornnlab
   ```
   for global npm package removal.

## 6. Interaction And Information Design

- Phase output should be short and explicit.
- Long build output may still stream from package managers, but it must be
  preceded by a clear phase label.
- Any system-level install command must be printed before execution, including
  commands that may request `sudo`, Homebrew installation, package-manager
  prompts, or Windows installer confirmation.
- Errors should say whether the user can rerun `ornnlab` to resume.
- `ornnlab install` should force the bootstrap path and rerun missing or
  incomplete stages.
- `ornnlab setup` should remain a compatibility alias for `ornnlab install`.
- `ornnlab update` should refresh the managed source checkout and dependencies
  without starting the WebUI.
- `ornnlab uninstall` should remove launcher-managed runtime files without
  deleting product data by default.
- `ornnlab doctor` should remain available for deeper application diagnostics
  after bootstrap succeeds.

## 7. Product Rules And State Logic

- Required prerequisites for normal WebUI startup:
  - `git`
  - `uv`
  - Node.js runtime
  - `npm`
- Required project dependencies:
  - Python environment from `uv sync --group dev`
  - Harbor import and OrnnLab CLI version checks after Python sync
  - frontend dependencies from `npm ci`
  - frontend build check after dependency install
- Optional prerequisite:
  - Docker core tooling, not Docker Desktop
- State should record:
  - `schemaVersion`
  - launcher version
  - platform
  - prerequisite presence and installation attempts
  - Docker status: present, installed, skipped, failed, or unknown
  - source checkout status
  - backend dependency status
  - frontend dependency status
  - last error and timestamp
- Uninstall state should record:
  - removed launcher-managed paths
  - preserved data paths
  - backup/trash paths when data cleanup is explicitly requested
- Readiness should be derived from actual files/commands, not only from the state
  file.

## 7.1 Bootstrap Policy Decisions

- Privilege and transparency:
  - Automatic required-tool installation is allowed to use interactive privilege
    escalation when the platform requires it.
  - The launcher must print the exact command before running it.
  - If the user cancels the OS prompt or `sudo`, setup stops before project
    dependency sync and records the failure.
- Node/npm boundary:
  - OrnnLab does not solve the pre-npm bootstrap problem. The initial
    `npm install -g ornnlab` still requires an existing Node/npm entrypoint.
  - After installation, the launcher still detects Node/npm because PATH or
    package-manager state may drift.
- Docker boundary:
  - Docker is optional for first WebUI launch.
  - OrnnLab detects Docker and records capability when present.
  - If Docker is missing, OrnnLab may install lightweight core tooling only.
  - macOS lightweight path is Docker CLI plus Colima, not Docker Desktop.
  - Linux lightweight path is the distribution's Docker engine package when
    available.
  - Windows must not install Docker Desktop; if no safe core-only path is
    available, OrnnLab explains the WSL/Docker Engine path and continues.
  - OrnnLab does not force-start Docker, log users in, accept license prompts, or
    guarantee the daemon is immediately running.
- State and integrity:
  - `bootstrap-state.json` must include `schemaVersion` and `launcherVersion`.
  - State is diagnostic evidence; actual command/file checks remain the source
    of truth.
  - Backend sync is not complete until Harbor and OrnnLab can be imported/run.
  - Frontend sync is not complete until the frontend build check passes.
- Network and retry:
  - Network failures during git, uv, npm, or npm-registry version checks must
    identify the failed phase.
  - Rerunning `ornnlab install` retries incomplete phases without requiring the
    user to manually delete partial files.

## 8. Edge Cases, Errors, And Recovery

- If a package manager is unavailable, show the exact missing prerequisite and
  a platform-specific manual fallback.
- If a required-tool install needs elevated permissions, print the command first
  and let the OS/package manager prompt the user.
- If automatic install fails, stop before source/dependency setup and record the
  failure.
- If network access fails during git, uv, npm, or registry checks, show the
  failed phase and tell the user that rerunning `ornnlab install` retries it.
- If source clone succeeds but dependency installation fails, rerunning
  `ornnlab` should retry the failed dependency stage.
- If update detects an outdated npm launcher, it should print the npm update
  command instead of silently modifying the global install.
- If update cannot reach the npm registry, it should continue source/dependency
  update and record the version-check failure as a warning.
- If uninstall finds no launcher-managed files, it should exit successfully and
  report that the install is already clean.
- If uninstall is asked to remove user data, it should require explicit
  confirmation and use a recoverable move to backup/trash, not permanent delete.
- If OrnnLab appears to be running, uninstall should ask the user to stop it
  before removing managed files.
- If Docker install is skipped, do not block WebUI launch.
- If lightweight Docker tooling is installed but the daemon/runtime is not
  running, record Docker as installed but not running and continue.
- If the only automatic Docker path is Docker Desktop, do not install it; explain
  the lightweight/manual path and continue.
- If the platform is unsupported or locked down by policy, the launcher should
  explain the manual path and keep retry behavior safe.

## 9. Content And Terminology

- Use "bootstrap" for machine and project setup.
- Use "required prerequisites" for `git`, `uv`, Node.js, and npm.
- Use "optional Docker capability" for Docker.
- Avoid saying "build failed" when the actual issue is a missing system tool.

## 10. Acceptance Criteria

- Given `git` or `uv` is missing on a supported platform, when the user runs
  `ornnlab install`, then the launcher attempts to install the missing tool using
  the supported platform installer.
- Given Docker is missing, when setup reaches Docker detection, then the user is
  prompted with a yes/no choice and may skip it.
- Given Docker is missing on macOS and the user chooses install, then the
  launcher installs Docker CLI plus Colima rather than Docker Desktop.
- Given Docker is missing on Windows and no core-only path is available, then the
  launcher does not install Docker Desktop and instead gives WSL/Docker Engine
  guidance.
- Given Docker is skipped, when setup continues, then backend and frontend
  dependencies still install and the WebUI can start.
- Given backend dependency sync completes, then `import harbor`, `import ornnlab`,
  and `ornnlab --version` checks pass before backend is marked ready.
- Given frontend dependency install completes, then the frontend build check
  passes before frontend is marked ready.
- Given bootstrap writes state, then the state includes `schemaVersion` and
  `launcherVersion`.
- Given setup is interrupted after clone but before frontend install, when the
  user reruns `ornnlab`, then setup retries the missing frontend dependency
  stage.
- Given all dependencies are already present, when the user reruns `ornnlab`,
  then setup avoids unnecessary reinstall work and starts the WebUI.
- Given the user runs `ornnlab update`, when the managed source checkout is a
  git repository, then the launcher runs a fast-forward update and reruns backend
  and frontend dependency sync without starting servers.
- Given the installed launcher version is behind npm latest, when the user runs
  `ornnlab update`, then the launcher prints `npm install -g ornnlab@latest`.
- Given the user runs `ornnlab uninstall`, then launcher-managed files are
  removed or archived and `~/.ornnlab/data` is preserved by default.
- Given the user requests full data cleanup during uninstall, then the launcher
  requires explicit confirmation and moves data to a recoverable backup/trash
  location.
- Given the user runs `ornnlab uninstall`, then the launcher prints
  `npm uninstall -g ornnlab` for removing the global npm package.
- Given automatic install fails, when the launcher exits, then the user sees the
  failed prerequisite, attempted command, and a safe rerun path.

## 11. Review Checklist And Sign-off Questions

- Does the first-run output make it obvious which phase is running?
- Does Docker feel optional rather than a hard first-launch blocker?
- Does rerun behavior recover from partial setup?
- Are platform-specific install paths transparent enough for users to trust?

## 12. Clarification Decision Log

| Topic | Decision | Rationale | Source Round |
|---|---|---|---|
| Dependency installation | Automatically install recommended dependencies | Minimize blank-machine friction | User answer `1. A` |
| Platform scope | macOS, Linux, and Windows | First-run bootstrap should not be macOS-only | User answer `2. C` |
| Docker behavior | Detect and record if present; prompt yes/no if missing; allow skip and retry later | Docker is important but should not block first WebUI launch | User answer `3` |

## 13. Open Questions And Risks

- Some machines may lack a supported package manager or block privileged
  installers.
- Docker installation may require UI steps, license acceptance, or daemon start
  outside the launcher.
- The npm launcher cannot bootstrap the initial Node/npm used to install itself;
  this must remain explicit in install docs.

## 14. Implementation Notes

- Keep the npm package small; implement bootstrap logic in the launcher.
- Prefer transparent commands and phase logs over silent background setup.
- Treat actual filesystem/command checks as source of truth; state files are
  diagnostic aids, not readiness authority.
