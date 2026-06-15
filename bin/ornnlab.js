#!/usr/bin/env node

const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { version: packageVersion } = require("../package.json");

const repoUrl = process.env.ORNNLAB_REPO || "https://github.com/ceasarXuu/HarnessLab.git";
const homeDir = process.env.ORNNLAB_HOME || path.join(os.homedir(), ".ornnlab");
const sourceDir = process.env.ORNNLAB_SOURCE || path.join(homeDir, "HarnessLab");
const backendHost = process.env.ORNNLAB_BACKEND_HOST || "127.0.0.1";
const backendPort = process.env.ORNNLAB_BACKEND_PORT || "8765";
const frontendHost = process.env.ORNNLAB_FRONTEND_HOST || "127.0.0.1";
const frontendPort = process.env.ORNNLAB_FRONTEND_PORT || "5173";

const help = `OrnnLab npm launcher

Usage:
  ornnlab                    Set up if needed, then start the local WebUI
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
  ORNNLAB_BACKEND_PORT   Default: 8765
  ORNNLAB_FRONTEND_PORT  Default: 5173

Prerequisites:
  git, uv, Node.js, and npm must be available on PATH.

When the app starts, the terminal prints:
  Frontend: http://${frontendHost}:${frontendPort}/
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
    throw new Error("Source checkout not found. Run: ornnlab setup");
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
  run("uv", ["run", "harnesslab", "web", "--host", backendHost, "--port", backendPort, ...args], {
    cwd: sourceDir,
  });
}

function runDoctor(args) {
  ensureSource();
  run("uv", ["run", "harnesslab", "doctor", ...args], { cwd: sourceDir });
}

function runFrontend(args) {
  ensureSource();
  run("npm", ["run", "dev", "--", "--host", frontendHost, "--port", frontendPort, "--strictPort", ...args], {
    cwd: path.join(sourceDir, "frontend"),
  });
}

function printLaunchUrls() {
  console.log("");
  console.log("OrnnLab is starting.");
  console.log(`Frontend: http://${frontendHost}:${frontendPort}/`);
  console.log(`Backend API: http://${backendHost}:${backendPort}/`);
  console.log("Press Ctrl+C to stop both servers.");
  console.log("");
}

function runDev({ setupIfMissing = false } = {}) {
  if (!fs.existsSync(sourceDir) && setupIfMissing) {
    console.log("OrnnLab source checkout not found; running setup first.");
    setup();
  }
  ensureSource();
  printLaunchUrls();
  const backend = spawnAttached(
    "uv",
    ["run", "harnesslab", "web", "--host", backendHost, "--port", backendPort],
    { cwd: sourceDir },
  );
  const frontend = spawnAttached(
    "npm",
    ["run", "dev", "--", "--host", frontendHost, "--port", frontendPort, "--strictPort"],
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
  if (!command) {
    runDev({ setupIfMissing: true });
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
  if (command === "dev" || command === "start") {
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
