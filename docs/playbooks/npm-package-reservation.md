# npm Package Reservation Playbook

This playbook records the steps used to reserve the public
`@ceasarxuu/harnesslab` npm package and `harnesslab` CLI command names.

## Goal And Current Outcome

Original goal:

- npm package name: unscoped `harnesslab`
- CLI command name: `harnesslab`

Current outcome:

- npm package name achieved: scoped fallback `@ceasarxuu/harnesslab`
- CLI command name achieved after install: `harnesslab`
- npm package name not achieved: unscoped `harnesslab`

The reservation package is intentionally small. It publishes package metadata,
the license, README, and a command shim that reports the current distribution
status.

The unscoped `harnesslab` package name cannot currently be published because npm
rejects it as too similar to the existing `harness-lab` package. The scoped name
is the npm-recommended fallback and still reserves the `harnesslab` executable
when installed from the scoped package.

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
