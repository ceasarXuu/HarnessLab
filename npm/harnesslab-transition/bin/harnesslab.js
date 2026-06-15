#!/usr/bin/env node

const { version: packageVersion } = require("../package.json");

const help = `HarnessLab has been renamed to OrnnLab.

Usage:
  harnesslab --help
  harnesslab --version

Install the active OrnnLab package instead:
  npm install -g ornnlab
  ornnlab --help

This scoped package only preserves the old harnesslab command as a transition
notice for users of @ceasarxuu/harnesslab.
`;

const args = process.argv.slice(2);

if (args.includes("--version") || args.includes("-V")) {
  console.log(packageVersion);
  process.exit(0);
}

if (args.length === 0 || args.includes("--help") || args.includes("-h")) {
  console.log(help.trimEnd());
  process.exit(0);
}

console.error("HarnessLab has been renamed to OrnnLab. Run `ornnlab --help`.");
process.exit(1);
