# Release Ledger: OrnnLab v0.1.3

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Created the canonical release ledger inside the `v0.1.3` version document folder. |
| 1.1 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-21 | Added update/uninstall implementation, version governance guard, and publish script to user-visible changes. |
| 1.2 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-23 | Consolidated version trio into `docs/releases/v0.1.3/`; updated documentation list and historical index pointer. |

- Status: Prepared, not yet published
- Date: 2026-06-16
- Commit: `76f754f`
- Version authority: `package.json`
- Build Set: OrnnLab Build Set `2026.06.16`
- Historical index entry: `./2026-06-16-ornnlab-0.1.3.md`

## Build Set Composition

| Component | Version / Range | Authority | Role |
|---|---:|---|---|
| npm launcher | `0.1.3` | `package.json` | Public install/update entrypoint |
| Python app/backend | `0.2.0` | `pyproject.toml` | Local API, CLI, data model, diagnostics |
| Frontend private package | `0.1.0` | `frontend/package.json` | WebUI implementation package |
| Transition npm package | `0.1.2` | `npm/harnesslab-transition/package.json` | Old scoped command migration notice |
| Harbor dependency | `>=0.13,<0.14` | `pyproject.toml` | Execution engine compatibility range |
| Source commit | `76f754f` | git | Prepared bootstrap policy implementation |

## Artifacts

| Artifact | Version | Authority | Published? |
|---|---:|---|---|
| `ornnlab` npm launcher | `0.1.3` | `package.json` | no |
| Python app package | `0.2.0` | `pyproject.toml` | no |
| Frontend private package | `0.1.0` | `frontend/package.json` | no |
| `@ceasarxuu/harnesslab` transition package | `0.1.2` | `npm/harnesslab-transition/package.json` | already published |

## User-Visible Changes

- Hardened `ornnlab install` bootstrap policy.
- Required install commands are printed before execution.
- Docker handling is core-only and does not install Docker Desktop.
- Bootstrap state includes schema and launcher versions.
- Backend readiness verifies Harbor and OrnnLab imports after `uv sync`.
- Frontend readiness verifies `npm run build` after `npm ci`.
- Current release documentation is consolidated under `docs/releases/v0.1.3/`.
- Implemented `ornnlab update` command with `--dry-run` support.
- Implemented `ornnlab uninstall` command with recoverable dated backup.
- Split `bin/ornnlab.js` into `lib/` modules for maintainability.
- Added `scripts/verify-version-governance.py` version governance guard.
- Added `npm_publish.sh` automated publish script with WebAuthn flow.

## Documentation Updated

- `README.md`
- `docs/architecture/docs-index.md`
- `docs/releases/v0.1.3/version-governance.md`
- `docs/releases/v0.1.3/checklist.md`
- `docs/releases/v0.1.3/ornnlab-0.1.3-docs.md`
- `docs/releases/v0.1.3/prd.md`
- `docs/releases/v0.1.3/technical-design.md`
- `docs/releases/v0.1.3/engineering-plan.md`
- `docs/releases/v0.1.3/ornnlab-0.1.3.md`

## Local Validation

```bash
uv run python scripts/verify-ornnlab-rebrand.py
uv run python -m py_compile scripts/verify-ornnlab-rebrand.py
git diff --check
npm run smoke:npm-bin
```

## Publish Plan

Use the npm WebAuthn flow documented in
`../playbooks/npm-package-reservation.md`:

```bash
npm login --auth-type=web
npm publish --access public --auth-type=web
```

Publishing requires the maintainer to complete npm web login and approve the
local machine WebAuthn key. Do not treat this project as using TOTP or an npm
access token for publish.

After publishing, append:

```bash
npm view ornnlab name version bin --json
npx --yes ornnlab --version
npx --yes ornnlab --help
```

## Rollback Notes

- Reinstall the previous npm launcher with `npm install -g ornnlab@0.1.1` if a
  launcher rollback is required before `0.1.3` is published.
- Do not delete `~/.ornnlab/data`.
- Use `ornnlab uninstall` only after it is implemented; until then, remove
  launcher-managed files manually by moving them to a dated backup.
