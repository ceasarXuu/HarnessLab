#!/usr/bin/env node

const { version: packageVersion } = require("../package.json");
const { setup, ensureReady } = require("../lib/bootstrap");
const { runDev, runBackend, runDoctor, runFrontend, frontendHost, frontendPort } = require("../lib/dev");
const { sourceDir } = require("../lib/state");
const { repoUrl } = require("../lib/source");

const help = `OrnnLab npm launcher

Usage:
  ornnlab                    Bootstrap if needed, then start the local WebUI
  ornnlab install            Install prerequisites, clone/update source, and install dependencies
  ornnlab setup              Alias for install
  ornnlab dev                Start backend and frontend development servers
  ornnlab web [args...]      Start the FastAPI backend from the managed source checkout
  ornnlab ui [args...]       Start the Vue frontend dev server from the managed source checkout
  ornnlab doctor [args...]   Run OrnnLab doctor from the managed source checkout
  ornnlab path               Print the managed source checkout path
  ornnlab --version          Print launcher version
  ornnlab --help             Show this help

Environment:
  ORNNLAB_LAUNCHER_HOME   Default: ~/.ornnlab/launcher
  ORNNLAB_SOURCE          Default: ~/.ornnlab/launcher/source
  ORNNLAB_REPO            Default: ${repoUrl}
  ORNNLAB_BACKEND_PORT    Default: 8765
  ORNNLAB_FRONTEND_PORT   Default: 5173
  ORNNLAB_INSTALL_DOCKER  1=yes, 0=no, unset=ask when missing
  ORNNLAB_AUTO_INSTALL    0=diagnose only; default installs missing required tools

Bootstrap behavior:
  Required tools are git, uv, Node.js, and npm. The launcher tries to install
  missing required tools on macOS, Linux, and Windows when a supported package
  manager is available and prints each install command before running it. Docker
  is optional for first launch; if it is missing, the launcher asks whether to
  install lightweight core tooling and records the choice. The launcher does not
  install Docker Desktop.

When the app starts, the terminal prints:
  Frontend: http://${frontendHost}:${frontendPort}/
`;

async function main() {
  const [command, ...args] = process.argv.slice(2);
  if (!command) {
    await runDev({ setupIfMissing: true });
    return;
  }
  if (command === "--help" || command === "-h" || command === "help") {
    console.log(help.trimEnd());
    return;
  }
  if (command === "--version" || command === "-V") {
    console.log(packageVersion);
    return;
  }
  if (command === "path") {
    console.log(sourceDir);
    return;
  }
  if (command === "install" || command === "setup") {
    await setup();
    return;
  }
  if (command === "web") {
    runBackend(args);
    return;
  }
  if (command === "ui") {
    runFrontend(args);
    return;
  }
  if (command === "doctor") {
    runDoctor(args);
    return;
  }
  if (command === "dev" || command === "start") {
    await runDev();
    return;
  }
  throw new Error(`Unknown command: ${command}`);
}

main().catch((error) => {
  console.error(error.message);
  console.error("Rerun `ornnlab install` after fixing the issue; bootstrap will retry incomplete stages.");
  process.exit(1);
});
