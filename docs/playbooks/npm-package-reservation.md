# npm Package Reservation Playbook

This playbook records the steps used to reserve the public
`@ceasarxuu/harnesslab` npm package and `harnesslab` CLI command names.
It also records the additional brand-name reservations `harnessrig` and
`harnessyard`.

## Goal And Current Outcome

Original goal:

- npm package name: unscoped `harnesslab`
- CLI command name: `harnesslab`

Current outcome:

- npm package name achieved: scoped fallback `@ceasarxuu/harnesslab`
- CLI command name achieved after install: `harnesslab`
- npm package name not achieved: unscoped `harnesslab`
- additional package names achieved: `harnessrig`, `harnessyard`
- additional CLI command names achieved after install: `harnessrig`,
  `harnessyard`

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
distribution strategy is prepared.

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

## Local Validation

```bash
npm run smoke:npm-bin
npm pack --dry-run
tmpdir=$(mktemp -d)
tarball=$(npm pack --pack-destination "$tmpdir" --silent)
npm install --prefix "$tmpdir/install" "$tmpdir/$tarball"
"$tmpdir/install/node_modules/.bin/harnesslab" --version
"$tmpdir/install/node_modules/.bin/harnesslab" --help
```

Expected signals:

- `harnesslab --version` prints the package version.
- `harnesslab --help` explains that the npm package is a reservation package.
- The tarball contents are limited by `package.json` `files`.

## Publish

Preferred path when npm requires 2FA:

```bash
npm publish --access public --otp=<current-otp>
npm view @ceasarxuu/harnesslab name version bin --json
curl -s https://api.npmjs.org/downloads/point/last-month/%40ceasarxuu%2Fharnesslab
tmpdir=$(mktemp -d)
cd "$tmpdir"
npx --yes @ceasarxuu/harnesslab --version
npx --yes @ceasarxuu/harnesslab --help
```

Expected signals:

- `npm publish` publishes the current `@ceasarxuu/harnesslab` version from
  `package.json`.
- `npm view` returns `name = @ceasarxuu/harnesslab` and `bin.harnesslab`.
- Downloads API returns a package record instead of `package not found`.
- Clean-directory `npx @ceasarxuu/harnesslab` executes the `harnesslab` bin from
  the registry package.

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

If npm returns `E403` with a two-factor authentication message, the current
token is authenticated but cannot bypass 2FA. Retry with one of:

```bash
npm publish --access public --otp=<current-otp>
```

If a granular token with bypass 2FA is explicitly needed, do not paste the token
into a shell command, committed file, screenshot, or shared log. Store it only
in ignored local state or read it interactively:

```bash
read -rsp "NODE_AUTH_TOKEN: " NODE_AUTH_TOKEN
printf "\n"
export NODE_AUTH_TOKEN
npm publish --access public
unset NODE_AUTH_TOKEN
```

For this repository, `.env.local` may temporarily contain `npm_access_token`
only as a last resort and only because `.env.local` is ignored. Prefer OTP for
normal publishes. If a temporary token is already present, load it without
printing the value:

```bash
set -a
source .env.local
set +a
test -n "$npm_access_token"
NODE_AUTH_TOKEN="$npm_access_token" npm publish --access public
unset npm_access_token
```

Record sanitized evidence for the release:

```bash
npm pack --dry-run --json
npm view @ceasarxuu/harnesslab name version bin --json
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/@ceasarxuu%2Fharnesslab
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/harnesslab
npm run smoke:npm-registry
```

Do not store raw tokens, OTPs, npm debug logs, or command output containing
credential material in repository artifacts.

## Additional Brand Reservations

On 2026-06-03, the following unscoped npm reservation packages were published:

| Package | Version | CLI command | Registry status |
|---|---:|---|---|
| `harnessrig` | `0.1.0` | `harnessrig` | `200` |
| `harnessyard` | `0.1.0` | `harnessyard` | `200` |

Preflight exact-name checks returned `404` for both names before publishing:

```bash
npm view harnessrig name version bin --json
npm view harnessyard name version bin --json
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
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/harnessrig
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/harnessyard
tmpdir=$(mktemp -d)
cd "$tmpdir"
npx --yes harnessrig --version
npx --yes harnessrig --help
npx --yes harnessyard --version
npx --yes harnessyard --help
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
