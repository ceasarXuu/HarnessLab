# npm Package Reservation Playbook

This playbook records the steps used to reserve the public
`@ceasarxuu/harnesslab` npm package and `harnesslab` CLI command names, plus the
later `ornnlab` distribution package.

## Goal And Current Outcome

Original goal:

- npm package name: unscoped `harnesslab`
- CLI command name: `harnesslab`

Current outcome:

- npm package name achieved: scoped fallback `@ceasarxuu/harnesslab`
- CLI command name achieved after install: `harnesslab`
- npm package name not achieved: unscoped `harnesslab`
- active npm distribution package: `ornnlab`
- additional package names achieved: `harnessrig`, `harnessyard`, `ornnlab`
- additional CLI command names achieved after install: `harnessrig`,
  `harnessyard`, `ornnlab`

The reservation package is intentionally small. It publishes package metadata,
the license, README, and a command shim that reports the current distribution
status.

The unscoped `harnesslab` package name cannot currently be published because npm
rejects it as too similar to the existing `harness-lab` package. The scoped name
is the npm-recommended fallback and still reserves the `harnesslab` executable
when installed from the scoped package.

`harnessrig` and `harnessyard` were published as independent unscoped
reservation packages at version `0.1.0`. Each package owns its same-named CLI
command and points users back to the HarnessLab repository while the native CLI
distribution strategy was still being prepared.

`ornnlab` was later published with the same reservation-package pattern at
version `0.1.0`, owning the `ornnlab` command. `ornnlab@0.1.1` is now the
active npm launcher for the OrnnLab Harbor WebUI source workflow. The product
name for that active path is OrnnLab.

## Preflight

```bash
npm whoami
npm profile get --json
git check-ignore -q .env.local
git check-ignore -q .env
git check-ignore -q .npmrc
! git ls-files --error-unmatch .env
! git ls-files --error-unmatch .env.local
! git ls-files --error-unmatch .npmrc
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/@ceasarxuu%2Fharnesslab
```

Expected signals:

- `npm whoami` returns the publishing account.
- `npm profile get --json` shows the account profile, including 2FA state.
- `.env.local` and `.npmrc` are ignored.
- The `git ls-files --error-unmatch` checks fail because local secret files
  must not be tracked.
- For a new package name before first publish, registry HTTP status should be
  `404`.
- For the current scoped fallback after publication, registry HTTP status should
  be `200` for `@ceasarxuu/harnesslab`.
- For the blocked unscoped name, registry HTTP status should remain `404` for
  `harnesslab` unless npm support changes the similarity decision.

## Local Validation For Active `ornnlab` Package

```bash
npm run smoke:npm-bin
npm pack --dry-run
tmpdir=$(mktemp -d)
tarball=$(npm pack --pack-destination "$tmpdir" --silent)
npm install --prefix "$tmpdir/install" "$tmpdir/$tarball"
"$tmpdir/install/node_modules/.bin/ornnlab" --version
"$tmpdir/install/node_modules/.bin/ornnlab" --help
```

Expected signals:

- `ornnlab --version` prints the package version.
- `ornnlab --help` explains that plain `ornnlab` starts the local WebUI and
  prints the frontend URL.
- The tarball contents are limited by `package.json` `files`.

## Publish Active `ornnlab` Package

The publishing account uses npm WebAuthn / local machine security-key
verification for write actions. Do not assume a TOTP code exists, and do not
try to solve this path with the old ignored `.env.local` access token. The
operator must complete npm web login and approve the publish with the local
security key.

First refresh the npm web login:

```bash
npm login --auth-type=web
```

Open the generated npm login URL if the CLI does not launch the browser
automatically. The command should finish with:

```text
Logged in on https://registry.npmjs.org/.
```

Then publish with web-based write authentication:

```bash
npm publish --access public --auth-type=web
npm view ornnlab name version bin --json
curl -s https://api.npmjs.org/downloads/point/last-month/ornnlab
tmpdir=$(mktemp -d)
cd "$tmpdir"
npx --yes ornnlab --version
npx --yes ornnlab --help
```

Expected signals:

- `npm publish` publishes the current `ornnlab` version from
  `package.json`.
- `npm view` returns `name = ornnlab` and `bin.ornnlab`.
- Downloads API returns a package record instead of `package not found`.
- Clean-directory `npx ornnlab` executes the `ornnlab` bin from the registry
  package.

Observed `ornnlab@0.1.1` release behavior on 2026-06-16:

- Plain `npm publish --access public` failed with `EOTP`.
- The account did not have a usable 6-digit TOTP code; its 2FA factor was the
  local machine security key / passkey.
- An ignored `.env.local` token did not bypass publish-time 2FA.
- `npm login --auth-type=web` refreshed the npm session for user `ceasarxuu`.
- `npm publish --access public --auth-type=web` printed an npm auth URL, the
  operator approved it with the local security key, and the publish completed.
- After publish, `npm view ornnlab name version bin --json` and
  clean-directory `npx --yes ornnlab --help` confirmed the live package.

If `npm publish --access public` returns success but `npm view` and the registry
still return `404`, check npm's staged package flow:

```bash
npm view @ceasarxuu/harnesslab name version bin --json
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/@ceasarxuu%2Fharnesslab
```

When both still return not-found after a successful publish, open npmjs.com,
select **Staged Packages**, review the staged `@ceasarxuu/harnesslab` package,
and click **Approve**. Approval requires 2FA / passkey verification. After
approval, rerun the registry and `npx` checks above.

If npm returns `EOTP` or `E403` with a two-factor authentication message, do not
ask for a TOTP code unless the operator explicitly says this account has one.
Retry with web authentication so the operator can approve the write using the
local security key:

```bash
npm publish --access public --auth-type=web
```

If a future granular token with bypass 2FA is explicitly created for automation,
do not paste the token into a shell command, committed file, screenshot, or
shared log. Store it only in ignored local state or read it interactively:

```bash
read -rsp "NODE_AUTH_TOKEN: " NODE_AUTH_TOKEN
printf "\n"
export NODE_AUTH_TOKEN
npm publish --access public
unset NODE_AUTH_TOKEN
```

For this repository, `.env.local` may temporarily contain `npm_access_token`
only as a last resort and only because `.env.local` is ignored. The token seen
during the `ornnlab@0.1.1` release did not bypass publish 2FA, so prefer the
WebAuthn path above for normal publishes. If a temporary token is already
present, load it without printing the value:

```bash
set -a
source .env.local
set +a
test -n "$npm_access_token"
NODE_AUTH_TOKEN="$npm_access_token" npm publish --access public
unset npm_access_token
```

Record sanitized evidence for the active `ornnlab` release:

```bash
npm pack --dry-run --json
npm view ornnlab name version bin --json
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/ornnlab
npm run smoke:npm-registry
```

Do not store raw tokens, OTPs, npm debug logs, or command output containing
credential material in repository artifacts.

## Publish Old Scoped Transition Package

The old scoped package should be published only from the dedicated staging
manifest:

```bash
npm run smoke:harnesslab-transition
cd npm/harnesslab-transition
npm publish --access public --auth-type=web
npm view @ceasarxuu/harnesslab name version bin --json
npx --yes @ceasarxuu/harnesslab --help
npx --yes ornnlab --help
```

Expected signals:

- The packed package contains only `README.md`, `bin/harnesslab.js`, and
  `package.json`.
- The `harnesslab` command prints a transition notice pointing to `ornnlab`.
- The root `ornnlab` package still excludes `bin/harnesslab.js`.
- The transition publish must use a new version. `@ceasarxuu/harnesslab@0.1.1`
  is already live with the older reservation message, so the staging manifest
  starts at `0.1.2`.
- `@ceasarxuu/harnesslab@0.1.2` published successfully immediately after
  `ornnlab@0.1.1` using the same refreshed npm session. Keep using
  `--auth-type=web` so npm can trigger local security-key approval if required.

## Additional Brand Reservations

The following unscoped npm reservation packages were published:

| Package | Version | CLI command | Registry status |
|---|---:|---|---|
| `harnessrig` | `0.1.0` | `harnessrig` | `200` |
| `harnessyard` | `0.1.0` | `harnessyard` | `200` |
| `ornnlab` | `0.1.0` | `ornnlab` | `200` |

Preflight exact-name checks returned `404` for each name before publishing:

```bash
npm view harnessrig name version bin --json
npm view harnessyard name version bin --json
npm view ornnlab name version bin --json
```

Each reservation package should stay small:

```text
LICENSE
README.md
bin/<command>.js
package.json
```

After publication, verify both package metadata and the clean-directory command
path:

```bash
npm view harnessrig name version bin --json
npm view harnessyard name version bin --json
npm view ornnlab name version bin --json
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/harnessrig
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/harnessyard
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/ornnlab
tmpdir=$(mktemp -d)
cd "$tmpdir"
npx --yes harnessrig --version
npx --yes harnessrig --help
npx --yes harnessyard --version
npx --yes harnessyard --help
npx --yes ornnlab --version
npx --yes ornnlab --help
```

Observed publish behavior:

- `npm publish --access public --auth-type=web` for `harnessrig` returned an
  `EOTP` message after web auth, but a follow-up publish reported that version
  `0.1.0` already existed. `npm view harnessrig name version bin --json` and
  registry HTTP `200` confirmed the package was live.
- Token-based publish from ignored `.env.local` succeeded for `harnessyard`.
  `npm view` briefly returned `404` immediately after publish while clean `npx`
  could already execute the command. Waiting a few seconds and rerunning
  `npm view` plus registry HTTP checks confirmed `harnessyard` was live.
- `ornnlab` publish required a fresh `npm login --auth-type=web` because the
  local npm session and ignored token were not authorized. After login, the
  write-action web auth flow published `ornnlab@0.1.0`; `npm view`, registry
  HTTP `200`, and clean `npx ornnlab` confirmed the reservation.
- Treat `npm publish` success, clean `npx`, and registry metadata as separate
  signals. Do not assume any one signal alone proves the final package state.

## Reuse Notes

- Do not use the `harness` command name for this package. Existing npm packages
  and Harness platform tooling already use or reference that command.
- Do not retry unscoped `harnesslab` unless npm support changes the similarity
  decision; current publication attempts are rejected as too similar to
  `harness-lab`.
- Keep the reservation package small until the native CLI distribution strategy
  is ready.
- When the npm CLI starts shipping the native binary, replace the shim with a
  real launcher and rerun the pack, install, and command smoke checks.
- Never commit `.env.local`, `.npmrc`, npm tokens, OTPs, screenshots containing
  credentials, or npm debug logs with credential material.
