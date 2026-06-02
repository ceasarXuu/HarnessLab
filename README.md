# HarnessLab

HarnessLab is a benchmark harness workspace for agent evaluation workflows.

This npm package is the scoped fallback for the blocked unscoped `harnesslab`
name. It reserves `@ceasarxuu/harnesslab` on npm and publishes the
`harnesslab` command while the native CLI distribution is prepared.

The unscoped `harnesslab` package name is blocked by npm's similarity policy
because `harness-lab` already exists.

Current source repository:

```text
https://github.com/ceasarXuu/HarnessLab
```

## CLI

The npm package currently exposes a reservation command:

```bash
npx @ceasarxuu/harnesslab --help
npx @ceasarxuu/harnesslab --version
```

Run registry-backed smoke checks from a clean directory, not from the repository
root, so local package metadata cannot affect `npx` resolution.

The production CLI is built from the Rust workspace in this repository.
