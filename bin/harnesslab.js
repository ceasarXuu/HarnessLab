#!/usr/bin/env node

const { version: packageVersion } = require("../package.json");

const help = `HarnessLab npm CLI reservation

Usage:
  harnesslab --help
  harnesslab --version

The production HarnessLab CLI is currently distributed from the source
repository. This npm package reserves the public package and command names
while the native CLI distribution is prepared.

Repository:
  https://github.com/ceasarXuu/HarnessLab
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

console.error("The npm-distributed HarnessLab CLI is not available yet.");
console.error("Run `harnesslab --help` for the current distribution status.");
process.exit(1);
