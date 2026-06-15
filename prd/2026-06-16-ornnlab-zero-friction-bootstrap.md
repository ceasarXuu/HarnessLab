# PRD: OrnnLab Zero-Friction Bootstrap

- Status: Ready for implementation
- Created: 2026-06-16
- Updated: 2026-06-16
- Owner / requester: maintainer
- Source request: Make first-run OrnnLab initialization install required dependencies on a blank computer and avoid user setup friction.

## Requester Review Summary

- Key decisions:
  - The launcher should automatically install recommended runtime dependencies instead of only printing manual instructions.
  - The first implementation should cover macOS, Linux, and Windows.
  - Docker should be detected and recorded when present; when missing, the user gets a yes/no installation choice and may skip it.
- Important exceptions:
  - `npm install -g ornnlab` still requires a working Node/npm entrypoint before the launcher can run.
  - Docker is optional for initial WebUI startup because it is mainly needed for real container-backed evaluation flows.
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
- Ask whether to install Docker when missing.
- Allow Docker skip and retry later.
- Clone/update the source checkout.
- Install Python backend dependencies.
- Install frontend dependencies.
- Persist bootstrap state for diagnostics and recovery.
- Make setup idempotent and recoverable after partial failure.

### Out Of Scope

- Replacing npm as the initial distribution mechanism.
- Fully unattended privileged OS setup in every enterprise-managed environment.
- Guaranteeing Docker Desktop license acceptance, OS-level permission grants, or
  daemon startup.
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

## 6. Interaction And Information Design

- Phase output should be short and explicit.
- Long build output may still stream from package managers, but it must be
  preceded by a clear phase label.
- Errors should say whether the user can rerun `ornnlab` to resume.
- `ornnlab install` should force the bootstrap path and rerun missing or
  incomplete stages.
- `ornnlab setup` should remain a compatibility alias for `ornnlab install`.
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
  - frontend dependencies from `npm ci`
- Optional prerequisite:
  - Docker
- State should record:
  - platform
  - prerequisite presence and installation attempts
  - Docker status: present, installed, skipped, failed, or unknown
  - source checkout status
  - backend dependency status
  - frontend dependency status
  - last error and timestamp
- Readiness should be derived from actual files/commands, not only from the state
  file.

## 8. Edge Cases, Errors, And Recovery

- If a package manager is unavailable, show the exact missing prerequisite and
  a platform-specific manual fallback.
- If automatic install fails, stop before source/dependency setup and record the
  failure.
- If source clone succeeds but dependency installation fails, rerunning
  `ornnlab` should retry the failed dependency stage.
- If Docker install is skipped, do not block WebUI launch.
- If Docker is installed but the daemon is not running, record Docker as present
  with a runtime warning instead of blocking WebUI launch.
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
- Given Docker is skipped, when setup continues, then backend and frontend
  dependencies still install and the WebUI can start.
- Given setup is interrupted after clone but before frontend install, when the
  user reruns `ornnlab`, then setup retries the missing frontend dependency
  stage.
- Given all dependencies are already present, when the user reruns `ornnlab`,
  then setup avoids unnecessary reinstall work and starts the WebUI.
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
