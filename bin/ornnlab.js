#!/usr/bin/env node

const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { version: packageVersion } = require("../package.json");

const repoUrl = process.env.ORNNLAB_REPO || "https://github.com/ceasarXuu/HarnessLab.git";
const homeDir = process.env.ORNNLAB_HOME || path.join(os.homedir(), ".ornnlab");
const sourceDir = process.env.ORNNLAB_SOURCE || path.join(homeDir, "HarnessLab");

const help = `OrnnLab npm launcher

Usage:
  ornnlab setup              Clone/update the HarnessLab source and install dependencies
  ornnlab dev                Start backend and frontend development servers
  ornnlab web [args...]      Start the FastAPI backend from the managed source checkout
  ornnlab ui [args...]       Start the Vue frontend dev server from the managed source checkout
  ornnlab doctor [args...]   Run HarnessLab doctor from the managed source checkout
  ornnlab path               Print the managed source checkout path
  ornnlab --version          Print launcher version
  ornnlab --help             Show this help

Environment:
  ORNNLAB_HOME     Default: ~/.ornnlab
  ORNNLAB_SOURCE   Default: ~/.ornnlab/HarnessLab
  ORNNLAB_REPO     Default: ${repoUrl}

Prerequisites:
  git, uv, Node.js, and npm must be available on PATH.
`;

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    env: process.env,
    stdio: options.stdio || "inherit",
  });
  if (result.error) {
    throw new Error(`${command} failed to start: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
}

function spawnAttached(command, args, options = {}) {
  const child = spawn(command, args, {
    cwd: options.cwd,
    env: process.env,
    stdio: "inherit",
  });
  child.on("error", (error) => {
    console.error(`${command} failed to start: ${error.message}`);
    process.exitCode = 1;
  });
  return child;
}

function ensureCommand(command) {
  const probe = spawnSync(command, ["--version"], { stdio: "ignore" });
  if (probe.error || probe.status !== 0) {
    throw new Error(`Required command not available on PATH: ${command}`);
  }
}

function ensureSource() {
  if (!fs.existsSync(sourceDir)) {
    throw new Error(`Source checkout not found. Run: ornnlab setup`);
  }
  const gitDir = path.join(sourceDir, ".git");
  if (!fs.existsSync(gitDir)) {
    throw new Error(`Source path exists but is not a git checkout: ${sourceDir}`);
  }
}

function setup() {
  ensureCommand("git");
  ensureCommand("uv");
  ensureCommand("npm");
  fs.mkdirSync(homeDir, { recursive: true });

  if (!fs.existsSync(sourceDir)) {
    run("git", ["clone", repoUrl, sourceDir]);
  } else {
    ensureSource();
    run("git", ["pull", "--ff-only"], { cwd: sourceDir });
  }

  run("uv", ["sync", "--group", "dev"], { cwd: sourceDir });
  run("npm", ["ci"], { cwd: path.join(sourceDir, "frontend") });
  console.log(`OrnnLab source is ready at ${sourceDir}`);
}

function runBackend(args) {
  ensureSource();
  run("uv", ["run", "harnesslab", "web", ...args], { cwd: sourceDir });
}

function runDoctor(args) {
  ensureSource();
  run("uv", ["run", "harnesslab", "doctor", ...args], { cwd: sourceDir });
}

function runFrontend(args) {
  ensureSource();
  run("npm", ["run", "dev", "--", "--host", "127.0.0.1", ...args], {
    cwd: path.join(sourceDir, "frontend"),
  });
}

function runDev() {
  ensureSource();
  const backend = spawnAttached("uv", ["run", "harnesslab", "web"], { cwd: sourceDir });
  const frontend = spawnAttached(
    "npm",
    ["run", "dev", "--", "--host", "127.0.0.1"],
    { cwd: path.join(sourceDir, "frontend") },
  );

  const shutdown = () => {
    backend.kill("SIGTERM");
    frontend.kill("SIGTERM");
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

function main() {
  const [command, ...args] = process.argv.slice(2);
  if (!command || command === "--help" || command === "-h") {
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
  if (command === "setup") {
    setup();
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
  if (command === "dev") {
    runDev();
    return;
  }
  throw new Error(`Unknown command: ${command}`);
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
