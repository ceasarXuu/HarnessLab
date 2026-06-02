# npm Package Reservation Playbook

This playbook records the steps used to reserve the public `harnesslab` npm
package and CLI command names.

## Goal

Reserve both:

- npm package name: `harnesslab`
- CLI command name: `harnesslab`

The reservation package is intentionally small. It publishes package metadata,
the license, README, and a command shim that reports the current distribution
status.

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
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/harnesslab
```

Expected signals:

- `npm whoami` returns the publishing account.
- `npm profile get --json` shows the account profile, including 2FA state.
- `.env.local` and `.npmrc` are ignored.
- The `git ls-files --error-unmatch` checks fail because local secret files
  must not be tracked.
- Registry HTTP status is `404` before the first publish.

## Local Validation

```bash
npm run smoke:npm-bin
npm pack --dry-run
tmpdir=$(mktemp -d)
npm pack --pack-destination "$tmpdir"
npm install --prefix "$tmpdir/install" "$tmpdir/harnesslab-0.1.0.tgz"
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
npm view harnesslab name version bin --json
curl -s https://api.npmjs.org/downloads/point/last-month/harnesslab
```

Expected signals:

- `npm publish` publishes `harnesslab@0.1.0`.
- `npm view` returns `name = harnesslab` and `bin.harnesslab`.
- Downloads API returns a package record instead of `package not found`.

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
npm view harnesslab name version bin --json
curl -s -o /dev/null -w "%{http_code}\n" https://registry.npmjs.org/harnesslab
```

Do not store raw tokens, OTPs, npm debug logs, or command output containing
credential material in repository artifacts.

## Reuse Notes

- Do not use the `harness` command name for this package. Existing npm packages
  and Harness platform tooling already use or reference that command.
- Keep the reservation package small until the native CLI distribution strategy
  is ready.
- When the npm CLI starts shipping the native binary, replace the shim with a
  real launcher and rerun the pack, install, and command smoke checks.
- Never commit `.env.local`, `.npmrc`, npm tokens, OTPs, screenshots containing
  credentials, or npm debug logs with credential material.
